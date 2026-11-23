# Taixin toolbox

A desktop tool for Taixin TXW8301 HaLow bridges, part of [taixin-halow-bridge](https://github.com/aliosa27/taixin-halow-bridge). It talks to devices on the Ethernet side, so the PC needs a wired connection to the bridge network but no IP address on the device's subnet.

- **Firmware (OTA):** find bridges (including far-end devices across the HaLow link), check a firmware `.bin`, flash it, reboot. "Reset settings" reboots with factory defaults, for moving between major versions (for example v1.6.x to v2.x) or recovering a device with broken settings.
- **NETAT console:** the vendor's AT-command console over UDP broadcast. The command line stays at the bottom of the window while replies scroll above it.
- **NETLOG:** the vendor's live log stream, with save to file. **Pop out** opens it in its own window, so you can watch the log while sending commands in the NETAT console; close that window (or press **Dock**) to put it back.
- **About:** version and build, source links, where to report issues.

NETAT and NETLOG only answer on stock Taixin firmware; this project's firmware has them off (use the web UI's Debug page instead).

## Download

Builds for Windows, macOS and Linux are on the [releases page](https://github.com/aliosa27/taixin-halow-bridge/releases) (`taixin-toolbox-*`).

## Running

| OS | Needs | Run |
|---|---|---|
| Windows 10/11 | [Npcap](https://npcap.com) (installed with Wireshark, or on its own) | `taixin-toolbox.exe` |
| macOS 11+ (Apple Silicon or Intel) | nothing extra (system libpcap) | `sudo ./taixin-toolbox`, or give your user access to `/dev/bpf*` (Wireshark's ChmodBPF) and run it normally |
| Linux x86_64 | libpcap (package `libpcap0.8` on Debian/Ubuntu, `libpcap` elsewhere) | `sudo ./taixin-toolbox`, or `sudo setcap cap_net_raw,cap_net_admin+ep ./taixin-toolbox` once |

macOS: the binary is not signed. If it is blocked, remove the download quarantine with `xattr -d com.apple.quarantine taixin-toolbox`.

Pick the adapter that is cabled to the bridge network at the top of the window, then use Scan on any tab.

The window follows the operating system's light or dark setting. The **Light / Dark / System** buttons at the top right override it until the tool is closed.

## Console mode

```sh
taixin-toolbox --version
taixin-toolbox --list                       # capture adapters
taixin-toolbox --scan <n>                   # OTA scan on adapter n
taixin-toolbox --netat-scan <n>
taixin-toolbox --netat <n> <mac> AT+VERSION
```

On Windows the release build has no console window, so console output only shows from a debug build (`cargo run -- ...`).

## Building

```sh
cargo build --release    # target/release/taixin-toolbox(.exe)
cargo test
```

The protocol, the flashing rules and build notes are in [docs/wnb-ota-protocol.md](../../docs/wnb-ota-protocol.md).

## License

Apache-2.0.
