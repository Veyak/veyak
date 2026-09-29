<div align="center">
  
<img src="public/icon.png" alt="Veyak Logo - Fast, Native, Open-Source API Client" width="108" height="108" style="border-radius: 22px; box-shadow: 0 0 40px rgba(99, 102, 241, 0.45);" />

# ⚡ Veyak ⚡

### The Blazing-Fast, Offline-First API Studio
**A modern, lightweight open-source alternative to Postman & Insomnia — powered by Rust and Tauri 2.**

[![Download](https://img.shields.io/github/v/release/Veyak/veyak?style=for-the-badge&color=6366F1&label=Download)](https://github.com/Veyak/veyak/releases/latest)
[![GitHub Stars](https://img.shields.io/github/stars/Veyak/veyak?style=for-the-badge&logo=github&color=FACC15)](https://github.com/Veyak/veyak/stargazers)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey?style=for-the-badge)](https://github.com/Veyak/veyak/releases/latest)

<p align="center">
  <a href="#download">Download</a> •
  <a href="#why-veyak">Why Veyak?</a> •
  <a href="#features">Features</a> •
  <a href="#benchmarks">Benchmarks</a> •
  <a href="#keyboard-shortcuts">Shortcuts</a> •
  <a href="https://veyak.iamdhakrey.dev/">Website</a>
</p>

<img width="100%" alt="Veyak Interface Preview - Clean, Native API Client" src="https://github.com/user-attachments/assets/4c71d4ee-7507-4fbe-a2ff-3fee55f5e4e4" />

</div>

---

## Download

Get the latest installer for your operating system from the **[Releases Page](https://github.com/Veyak/veyak/releases/latest)** or the **[Official Website](https://veyak.iamdhakrey.dev/)**:

| Platform | Download | Format |
| :--- | :---: | :---: |
| **macOS** | [Download](https://github.com/Veyak/veyak/releases/latest) | `.dmg` (Apple Silicon & Intel) |
| **Windows** | [Download](https://github.com/Veyak/veyak/releases/latest) | `.msi` / `.exe` installer |
| **Linux** | [Download](https://github.com/Veyak/veyak/releases/latest) | `.AppImage` / `.deb` |

> **Auto-Updates:** Veyak checks for updates automatically and updates seamlessly in-app.

---

## Why Veyak?

Tired of slow, bloated API clients that consume **1 GB+ of RAM**, demand cloud accounts just to test `localhost`, and freeze on large payloads?

Veyak is built from the ground up for speed, privacy, and developer ergonomics:

- **Instant Startup (< 100ms)** — Launches instantly with zero Electron bloat.
- **Ultra-Low Memory (< 60MB RAM)** — Leaves your resources free for your local dev servers and containers.
- **100% Offline-First & Private** — No mandatory accounts, no telemetry, and no cloud lock-in. Your data stays on your machine.
- **OS Keyring Security** — Sensitive tokens and secrets are encrypted directly in your native OS keychain.
- **All-in-One Multi-Protocol Studio** — Seamlessly test REST, gRPC, WebSockets, and GraphQL in a single, unified workspace.

---

## Features

### Multi-Protocol Support
- **REST / HTTP:** Full method support (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`, etc.), dynamic query parameters, custom headers with autocompletion, auth helpers (Bearer, Basic, API Key), and versatile body modes (JSON, Multipart / Form-Data, URL-encoded, Raw).
- **gRPC Studio:** Dynamic `.proto` schema reflection, unary calls, and full streaming support (Server, Client, and Bidirectional) powered by native Rust.
- **WebSocket Inspector:** Persistent connections (`ws://` and `wss://`), live frame feed with timeline indicators, and reusable message presets.
- **GraphQL Explorer:** Interactive queries, mutations, variables, schema introspection, and real-time subscriptions over WebSockets.

### Developer-First Ergonomics
-  **Workspaces & Collections:** Organize endpoints into folders with instant search and filter capabilities.
- **Environment Variables:** Use `{{variable}}` syntax across URLs, headers, and bodies with rapid environment switching.
- **Command Palette (`Cmd/Ctrl + P` or `K`):** Quickly navigate requests, collections, and settings from anywhere.
- **Interactive History:** Audit and replay past requests and inspect received responses.
- **Appearance & Font Controls:** Customize themes and select your favorite monospace fonts (JetBrains Mono, Fira Code, Inter, etc.).

---

## Benchmarks

| Metric | Postman | Insomnia | Veyak |
| :--- | :---: | :---: | :---: |
| **Runtime Architecture** | Electron (Chromium bundle) | Electron (Chromium bundle) | **Rust + Tauri 2 (Native Webview)** |
| **Memory Footprint (Idle)** | ~600 MB – 1.4 GB | ~400 MB – 900 MB | **< 60 MB** |
| **Cold Start Time** | 4.0s – 10.0s | 3.0s – 7.0s | **< 100 ms** |
| **Account Requirement** | Mandatory login | Cloud sync default | **None (100% Local & Offline)** |
| **Sensitive Data Storage** | Cloud sync / Proprietary | Cloud sync / Local | **Encrypted OS Keyring** |
| **Download / Install Size** | ~350 MB – 500 MB | ~200 MB – 300 MB | **< 20 MB** |

---

## Keyboard Shortcuts

| Action | macOS | Windows / Linux |
| :--- | :---: | :---: |
| **Send Request** | <kbd>⌘</kbd> + <kbd>Enter</kbd> | <kbd>Ctrl</kbd> + <kbd>Enter</kbd> |
| **Save Request** | <kbd>⌘</kbd> + <kbd>S</kbd> | <kbd>Ctrl</kbd> + <kbd>S</kbd> |
| **New Tab** | <kbd>⌘</kbd> + <kbd>T</kbd> | <kbd>Ctrl</kbd> + <kbd>T</kbd> |
| **Open Command Palette** | <kbd>⌘</kbd> + <kbd>P</kbd> or <kbd>K</kbd> | <kbd>Ctrl</kbd> + <kbd>P</kbd> or <kbd>K</kbd> |
| **Toggle Request History** | <kbd>⌘</kbd> + <kbd>H</kbd> | <kbd>Ctrl</kbd> + <kbd>H</kbd> |
| **Open Settings** | <kbd>⌘</kbd> + <kbd>,</kbd> | <kbd>Ctrl</kbd> + <kbd>,</kbd> |

---

## Contributing

Contributions are welcome! If you'd like to report an issue or suggest a feature:

1. Check open [Issues](https://github.com/Veyak/veyak/issues) or submit a new one.
2. For code contributions, fork the repository, make your changes, and submit a [Pull Request](https://github.com/Veyak/veyak/pulls).

---

## Show Your Support

If you find Veyak useful, please consider giving it a **Star (⭐️)** on [GitHub](https://github.com/Veyak/veyak)! It helps others discover the project.

---

## License & Author

Distributed under the **MIT License**. See [LICENSE](LICENSE) for details.

Crafted with ❤️ by [**Hrithik Dhakrey**](https://github.com/iamdhakrey)  
- **Website & Docs**: [veyak.iamdhakrey.dev](https://veyak.iamdhakrey.dev/)
