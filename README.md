# taixin-halow-bridge

Modifications to the Taixin TXW8301 (TXW4002A) 802.11ah HaLow wireless-bridge (WNB) firmware: an embedded web admin UI, HaLow channel planning, AT command fixes and a DHCP fallback address.

This repository does not contain the vendor SDK. It holds:

| Path | What |
|---|---|
| `patches/` | `git format-patch` series against [Taixin-Semiconductor/TXW8301_WNB](https://github.com/Taixin-Semiconductor/TXW8301_WNB) at `0e273c8` (SDK v2.4.1.3-43933) |
| `web/` | The web admin UI (Vue 3 + Vite). It started from [Taixin-Semiconductor/hgic-wnb-web](https://github.com/Taixin-Semiconductor/hgic-wnb-web) (`2e0384c`) but has diverged too far to keep as a patch, so the full source is here. |
| `scripts/` | `apply.sh` builds the patched firmware tree; `export-patches.sh` writes the patches back out. `upstream.env` pins the vendor commit. |

## Building

Requirements: Git (Git Bash on Windows), XuanTie CDK with the CK803 toolchain, and Node.js 20 only if you change the UI.

```sh
scripts/apply.sh            # clones the vendor tree into TXW8301_WNB/ and applies patches/
```

Open `TXW8301_WNB/project/txw4002a.cdkproj` in CDK, configuration `BuildSet`, and build. Flash or upload `project/txw8301_v<ver>-<svn>_<date>_.bin` (the makecode output; plain `txw8301.bin` lacks the OTA header).

The patches already contain the built UI (`project/webui_page.h`), so Node is not needed for a firmware-only build.

## Changing the UI

```sh
cd web && npm ci && npm run build
powershell -ExecutionPolicy Bypass -File TXW8301_WNB/project/utilities/gen_webui_page.ps1
```

This regenerates `TXW8301_WNB/project/webui_page.h`. The page is copied into SRAM with the rest of the image at boot, so keep it small (see `NOTES.md` in the patched tree).

## Changing the firmware

`TXW8301_WNB/` is a normal git checkout (branch `bridge`, ignored by this repo). Commit there, then:

```sh
scripts/export-patches.sh   # rewrites patches/ from the commits after the vendor base
git add -A patches && git commit
```

CDK rewrites `txw4002a.mk`, `cdkws.mk`, `*.cdkws` and `*.modify.bat` on every open; the patched tree ignores them.

## Releases and online update

The firmware reports its version as `v2.4.1.3-43933 app-1 r<n>`: SDK version and build, the vendor's app version, and this project's release number `BRIDGE_RELEASE` (`TXW8301_WNB/project/project_config.h`). Released images live in `releases/`, and `releases/latest.json` names the current one. The Firmware page's **Check for updates** reads that file from GitHub in the browser (the device itself never goes online), downloads the image, checks its size and CRC-32, and uploads it to the device.

To publish a release:

1. Bump `BRIDGE_RELEASE`, commit in `TXW8301_WNB`, run `scripts/export-patches.sh`, commit `patches/`.
2. Build in CDK and test the image on a device.
3. `scripts/release.sh TXW8301_WNB/project/txw8301_v2.4.1.3-43933_app-1_<date>_.bin "What changed"`
4. Commit `releases/` and push. Devices see it within about 5 minutes (GitHub's cache).

On a link, update the station end before the AP.

## Changes from the vendor firmware

- Web admin UI served from the device (`project/webui.c`): status, WiFi, radio, LAN, device management, firmware upload, AT console.
- Radio page: pick region (US 902–928, EU 863–868, China 760–784, custom) and bandwidth; the 802.11ah channel list follows the plan and the module's tuning range.
- WiFi page: WNB/standard mode selection, pairing button, raw 64-hex-digit PSK or passphrase.
- Status: RSSI, link (RX) rate and traffic per peer, Ethernet link and counters, heap and packet-buffer usage, firmware build time, DHCP host name.
- DHCP host name `<device name>-<last 3 MAC bytes>`; station IPs in the client list learned from ARP.
- Online update from this repo's `releases/` (Firmware page).
- DHCP client falls back to 192.168.123.1/24 after 60 s without a lease and keeps asking for one.
- `AT+SYSCFG` and new `AT+SCAN?` reply through the AT channel; `AT+PING`/`AT+IPERF2` dropped to save ~6 KB of SRAM.

Defaults to change before shipping a product: WiFi passphrase `12345678`, web password `admin`.

## License

Apache License 2.0, like both upstream projects. See `LICENSE` and `NOTICE`.
