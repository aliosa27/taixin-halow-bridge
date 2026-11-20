# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project Overview

Wireless network bridge device admin web UI (hgic-wnb-web-ng). Vue 3 SPA that communicates with a WNB device at `192.168.123.1` via a custom HTTP API. The final build output is a **single HTML file** embedded in device firmware.

## Build & Development Commands

```bash
npm run dev        # Dev server with HMR (proxies API to 192.168.123.1)
npm run build      # TypeScript check + Vite production build
npm run preview    # Preview production build locally
```

There are no test or lint commands configured.

## Architecture

**Stack**: Vue 3 (Composition API + `<script setup>`), TypeScript 5, Pinia, Vue Router, Vue I18n, Vite

### Key Source Files

- `src/wnbCom/index.ts` — Device API communication layer (~676 lines). All device interaction goes through here. Uses custom binary-delimited protocol over HTTP POST (US `0x1F` field separator, GS `0x1D` record separator). Endpoints: `/post.action`, `/auth.action`, `/fwupg.data`.
- `src/store/index.ts` — Three Pinia stores: `useDashboardStore` (auth token, device config), `useFormDataStore` (cached form data per page), `useSetupHelperStore` (wizard state).
- `src/router/index.ts` — Routes with auth guards. Redirects to `/login` when unauthenticated. Default post-login page is `/status/overview`.
- `src/utils/index.ts` — Input validation functions (IP, MAC, SSID, PSK, netmask, hostname) and enums (`WIFI_MODE`, `KEY_MGMT`, `SELECT`, `INPUT_CHECK`).
- `src/i18n/index.ts` — English and Simplified Chinese. Per-component translations via `<i18n>` blocks plus global fallback messages.

### Conditional Features

Device firmware reports capabilities via `deviceConfig`. Components conditionally render based on flags:
- `fixWiFiMode` — locks WiFi mode selection
- `noAdvWebCfg` — hides advanced WiFi settings page
- `noAPSTAmode` — disables repeater mode option
- `noWebAuth` — disables login/logout

### Dev Proxy

Vite proxies `/post.action`, `/auth.action`, and `/fwupg.data` to `http://192.168.123.1`. A physical or emulated device must be reachable at that address during development.

### Build Constraints

The output must be a single HTML file (via `vite-plugin-singlefile`) with legacy ES2015 support (via `@vitejs/plugin-legacy`). Bundle size matters — it ships in device firmware. Compression plugins (gzip/brotli) are enabled.

### Path Alias

`@/*` maps to `./src/*` (configured in tsconfig.json and vite.config.ts).

### LSP Integration

LSP is available and working for this project. Use the LSP tool for code intelligence: `goToDefinition`, `findReferences`, `hover`, `documentSymbol`, `workspaceSymbol`, `goToImplementation`, `incomingCalls`, and `outgoingCalls`.

### Authentication

HTTP header `Auth-Token` carries the session token. Passwords are hex-encoded before transmission. Token is persisted in `localStorage['authToken']`. Settings pages poll auth validity every 30 seconds.
