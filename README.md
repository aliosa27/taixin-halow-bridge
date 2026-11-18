# TXW81x WNB Web UI

[简体中文](README.zh-CN.md) | English

Web administration UI for the Taixin Semiconductor TXW81x family of SoC WNB
(Wireless Network Bridge) projects.

This repository contains a Vue 3 single-page application used to configure and
manage WNB devices. The production build is optimized to ship as a single HTML
file that can be embedded in device firmware.

## Features

- Device login and authenticated configuration flow
- Status overview and connected client information
- LAN network configuration
- Basic and advanced WiFi configuration
- Device management and firmware upgrade pages
- English and Simplified Chinese localization
- Development proxy for a device reachable at `192.168.123.1`

## Tech Stack

- Vue 3
- TypeScript
- Pinia
- Vue Router
- Vue I18n
- Vite

## Requirements

- Node.js 18 or newer
- npm
- A TXW81x WNB device available at `http://192.168.123.1` for live development
  against device APIs

## Getting Started

Install dependencies:

```bash
npm install
```

Start the development server:

```bash
npm run dev
```

During development, Vite proxies these device API paths to
`http://192.168.123.1`:

- `/post.action`
- `/auth.action`
- `/fwupg.data`

## Build

Create a production build:

```bash
npm run build
```

The build runs TypeScript checking through `vue-tsc` and then builds with Vite.
The output is written to `dist/` and is configured for single-file HTML output
through `vite-plugin-singlefile`.

Preview the production build locally:

```bash
npm run preview
```

## Project Layout

```text
src/
  assets/       Static assets and icons
  components/   Shared Vue components
  i18n/         Localization setup
  router/       Route definitions and auth guard
  store/        Pinia stores
  utils/        Shared validators, constants, and helpers
  views/        Page-level Vue components
  wnbCom/       WNB device API communication layer
```

## Notes

- The device API uses a custom HTTP protocol implemented in `src/wnbCom/`.
- The final web UI bundle is intended for constrained firmware environments, so
  bundle size matters.
- There are currently no dedicated lint or unit test scripts configured.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
