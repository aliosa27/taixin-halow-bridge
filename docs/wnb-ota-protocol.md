# WNB Ethernet OTA protocol

This is the raw-Ethernet protocol that Taixin's Windows OTA program uses to find TXW8301 bridges and write firmware to them. It is enabled by `SYS_APP_WNBOTA` (`wnb_ota_init()` in `main.c`).

The SDK ships only the header (`sdk/include/lib/ota/libota.h`). The rest of this page comes from disassembling the closed objects:

- `libnetutils.a`: `utils_wnbota.o` (`wnb_ota_pkthook_rx`, `wnb_ota_pkthook_tx`)
- `libcommon.a`: `ota_eth_ota.o` (`libota_proc`, `libota_check_sum`) and `ota_fw.o` (`libota_write_fw_ah`)

Disassembly was against SDK v2.4.1.3-43933. Points marked *unverified* have not been tested on hardware.

## Transport

- Plain Ethernet II frames, EtherType **0x4847** ("HG"). No IP is involved, so the PC needs no address on the device's subnet, only a layer-2 path to it.
- `wnb_ota_init()` registers two hooks on the WiFi bridge:
  - The **tx hook** sees frames the bridge is about to send over HaLow, which are the frames arriving from the Ethernet port. The device answers through `ieee80211_input`, so the reply goes back out the Ethernet port.
  - The **rx hook** sees frames received over HaLow. The device answers through `ieee80211_tx`, back over the air.
- A frame is handled when its destination is **broadcast/multicast**, or **equal to the device's WiFi MAC** (`ieee80211_conf_get_mac`, the MAC shown on Status).
  - **Unicast to the device:** the frame is consumed (not forwarded).
  - **Broadcast:** the frame is answered *and* still bridged. A broadcast scan therefore also reaches the device at the other end of the HaLow link, and its reply comes back through the local device.
  - **Unicast to the far device's MAC:** the frame passes through the local device unchanged.
  - Verified 2026-11-21: a broadcast SCAN from the PC returned the local WNB AP and both WNB stations behind it over HaLow, each with its own MAC. Remote *upgrade* through the partner is not tested yet.
- **Replies** are sent to the request's source MAC, with the device's WiFi MAC as the source. Any source MAC works: the tool uses a random locally administered one (Npcap does not report the adapter's own MAC) and captures in promiscuous mode.
- *Unverified:* whether the tx hook runs while the HaLow link is down, which matters for a first install with no partner.

## Frame layout

All multi-byte fields are **big-endian**, except the reboot flags.

```
offset  size  field
0       6     dest MAC
6       6     src MAC
12      2     EtherType 0x4847
14      1     stype
15      1     status
16      ...   body (per stype)
```

| stype | Name | Direction | Body |
|---|---|---|---|
| 1 | REBOOT | PC → device | `flags u32` at 16, **little-endian**. Bit 0: load factory defaults before rebooting. |
| 2 | SCAN | PC → device (usually broadcast) | none |
| 3 | SCAN_REPORT | device → PC | see below; 32-byte frame |
| 4 | FW_DATA | PC → device | see below |
| 5 | FW_DATA_RESP | device → PC | the request frame cut to 36 bytes, with `stype` = 5 and `status` set |
| 6–9 | GET/SET_PARAM(_RESP) | | defined in the header; this firmware returns nothing |

Frames shorter than 60 bytes are padded on the wire; the device ignores the padding.

### SCAN_REPORT (stype 3)

| Offset | Field | Notes |
|---|---|---|
| 16 | `version u32` | SDK version, e.g. `0x02040103` = 2.4.1.3 |
| 20 | `chipid u16` | `sysctrl_get_chip_id()`; FW_DATA must carry the same value |
| 22 | `mode u8` | the `libota_proc` mode argument, which is the WiFi interface index. In this firmware that equals the mode number: 1 STA, 2 WNB STA, 3 AP, 4 WNB AP (2 and 4 seen on hardware) |
| 23 | `rev u8` | 0 |
| 24 | `svn_version u32` | e.g. 43933 |
| 28 | `app_version u32` | e.g. 1 |

`status` is 0.

### FW_DATA (stype 4)

| Offset | Field | Notes |
|---|---|---|
| 16 | `version u32` | the device only checks that the low byte is **3**; send `3` |
| 20 | `off u32` | byte offset of this chunk in the image |
| 24 | `tot_len u32` | size of the `.bin` file |
| 28 | `len u16` | chunk length |
| 30 | `checksum u16` | see below |
| 32 | `chipid u16` | must equal the device's chip ID (from SCAN_REPORT) |
| 34 | `data[len]` | |

Data per frame is at most 1466 bytes (1500 MTU minus the 34-byte header). Use 1024 to leave headroom on the HaLow hop.

**Checksum** (`libota_check_sum`) is the Internet checksum (RFC 1071) over `data`, summing **little-endian** 16-bit words; with an odd length, the last byte is added on its own. Fold carries until the sum fits in 16 bits, then invert. The device compares the result with the big-endian `checksum` field.

```text
sum = 0
for each 2-byte pair (lo, hi): sum += lo | hi << 8
if odd: sum += last byte
while sum >> 16: sum = (sum & 0xffff) + (sum >> 16)
checksum = ~sum & 0xffff        // stored big-endian at offset 30
```

**Response:** FW_DATA_RESP echoes `off`, `tot_len`, `len` and `checksum`.

- `status` = 0: the chunk is written and verified.
- `status` = 0xFF: rejected. The cause is a wrong version byte, wrong chip ID, a bad checksum, or a failed write.

## Flash write rules (`libota_write_fw_ah`)

- **Starting:** `off` = 0 starts an upgrade. The first chunk must begin with the boot marker `69 5A` (`FWINFO_BOOT_HDR` 0x5A69), and the image's customer ID must match the chip's. The image goes to the OTA slot that is not running (slots at `0x00000` and `0x80000`).
- **Marker last:** the 2-byte marker is held back and written last, so an interrupted upgrade leaves the old image bootable.
- **Strictly sequential.** The device tracks the number of bytes written.
  - `off` **below** that count: returns OK and does nothing, so retransmitting a chunk whose response was lost is safe.
  - `off` **equal** to it: the chunk is erased as needed, written, read back and compared.
  - `off` **above** it (a gap): the session is aborted, and the upgrade must restart from 0.
- **Commit:** when `off + len == tot_len`, the device writes the marker, verifies it, and switches the boot slot. Do not send an extra zero-length frame afterwards, because the session is already closed and it fails.
- **No reboot:** the device does not reboot by itself. Send REBOOT with flags 0.
- **Size limit:** this firmware keeps its config in the last flash sectors, so images must be at most `0x7C000` bytes (496 KB).

## Upgrade sequence

1. Open the Ethernet adapter in promiscuous mode with the capture filter `ether proto 0x4847`.
2. Broadcast SCAN; collect SCAN_REPORTs for about 1 s.
3. Check the file:
   - it starts with `69 5A`
   - its size is at most `0x7C000`
   - the chip ID in its firmware-info header matches the report
4. For each chunk, send FW_DATA and wait for the FW_DATA_RESP with the same `off` (stop-and-wait). The tool uses 1400-byte chunks, waits 2.5 s per send, pauses 0.25 s before a re-send and 0.1 s after a reply slower than 0.5 s, and sends each chunk up to 10 times; a reply can take a while when the device erases a sector or answers over HaLow. On status 0xFF or no reply the device has dropped the session: restart from `off` 0 (the tool does this up to 3 times, 3 s apart), except when the first chunk is rejected, which means the image itself is refused.
5. Send REBOOT (flags 0). Re-scan after about 10 s to confirm the new version.
   - **Between major versions** (for example v1.6.x to v2.x), send flags 1 instead. The old firmware is still running at this point, so it clears its own settings (`syscfg_loaddef("syscfg")`: zeroes the magic of both copies in the last two flash sectors) and the new firmware boots on its defaults. This only helps when both versions keep their settings in the same place; this firmware also rejects an unusable saved config at boot (see `syscfg_check` in `project/syscfg.c`).

## Tool

`tools/taixin-toolbox` (Taixin toolbox) is a cross-platform (Windows, macOS, Linux) Rust implementation; see its README. It loads Npcap (`wpcap.dll`) or the system libpcap at run time.

The tool has four tabs:

- **Firmware (OTA):** scan, check a `.bin` (boot marker, size, chip ID against the selected device), flash with progress and cancel, reboot. "Reset settings" makes the reboot after flashing use flags 1, and the tool suggests it when the major version changes. "Reset settings and reboot" does the same without flashing.
- **NETAT console:** the vendor's AT-over-UDP broadcast (port 56789).
- **NETLOG:** the vendor's log stream (port 64320, heartbeat every 500 ms).
- **About:** version, commit and build date, source links, credits, and where to report issues (the same links as the firmware's About page).

NETAT and NETLOG frame layouts come from [taixin_tools](https://github.com/aliosa27/taixin_tools) (`docs/netat_protocol.md`, `docs/netlog_protocol.md`). They are sent as raw broadcast frames from the adapter's IPv4 address (or 0.0.0.0). Both services are off in this project's firmware, so they only answer on stock firmware.

Verified on hardware (2026-11-21): OTA discovery, flashing, reset settings. Not yet tested on hardware: NETAT, NETLOG (they need a device on stock firmware), flashing over the HaLow link.

### Building

```sh
cd tools/taixin-toolbox
cargo build --release          # GUI: target/release/taixin-toolbox(.exe)
cargo run -- --version         # console: version, commit, build date
cargo run -- --list            # console: list capture adapters
cargo run -- --scan <n>        # console: OTA scan on adapter n
cargo run -- --netat-scan <n>  # console: NETAT scan
cargo run -- --netat <n> <mac> AT+VERSION
```

- **Windows:** needs [Npcap](https://npcap.com) at run time.
  - With the `x86_64-pc-windows-gnu` toolchain, the build also needs MinGW binutils (`dlltool` and `as`) on `PATH`, because several dependencies use raw-dylib imports. The MSVC toolchain does not need them.
- **macOS:** uses the system libpcap. Run with `sudo`, or give your user access to `/dev/bpf*` (e.g. Wireshark's ChmodBPF).
- **Linux:** needs `libpcap.so.1` (package `libpcap0.8` or `libpcap`) and raw-socket access: run with `sudo`, or `sudo setcap cap_net_raw,cap_net_admin+ep taixin-toolbox`.
- **CI:** `.github/workflows/taixin-toolbox.yml` builds Windows x86_64, macOS universal (Apple Silicon + Intel) and Linux x86_64 on every push that touches the tool. Pushing a tag `toolbox-v*` (e.g. `toolbox-v0.2.0`) also publishes the three archives as a GitHub release.
