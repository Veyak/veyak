<div align="center">
  
<img src="public/icon.png" alt="Veyak Logo - Native, Offline-First Multi-Protocol API Studio" width="108" height="108" style="border-radius: 22px; box-shadow: 0 0 40px rgba(99, 102, 241, 0.45);" />

# ⚡ Veyak ⚡

### The Native Multi-Protocol API Studio Built for Speed & Privacy
**Sub-100ms startup, native OS keyring encryption, embedded QuickJS scripting, and zero cloud walls — powered by Rust & Tauri 2.**

[![Download](https://img.shields.io/github/v/release/Veyak/veyak?style=for-the-badge&color=6366F1&label=Download)](https://github.com/Veyak/veyak/releases/latest)
[![GitHub Stars](https://img.shields.io/github/stars/Veyak/veyak?style=for-the-badge&logo=github&color=FACC15)](https://github.com/Veyak/veyak/stargazers)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey?style=for-the-badge)](https://github.com/Veyak/veyak/releases/latest)

<p align="center">
  <a href="#download">Download</a> •
  <a href="#what-makes-veyak-different">What Makes Veyak Different</a> •
  <a href="#features">Features</a> •
  <a href="#architectural-comparison">Architectural Comparison</a> •
  <a href="#keyboard-shortcuts">Shortcuts</a> •
  <a href="https://veyak.iamdhakrey.dev/">Website</a>
</p>

<img width="100%" alt="Veyak Interface Preview - Native API Client" src="https://github.com/user-attachments/assets/4c71d4ee-7507-4fbe-a2ff-3fee55f5e4e4" />

</div>

---

## Download

Get the latest native build for your platform from the **[Releases Page](https://github.com/Veyak/veyak/releases/latest)** or the **[Official Website](https://veyak.iamdhakrey.dev/)**:

| Platform | Download | Architecture / Format |
| :--- | :---: | :---: |
| **macOS** | [Download](https://github.com/Veyak/veyak/releases/latest) | Universal `.dmg` (Apple Silicon & Intel) |
| **Windows** | [Download](https://github.com/Veyak/veyak/releases/latest) | Native `.msi` / `.exe` installer |
| **Linux** | [Download](https://github.com/Veyak/veyak/releases/latest) | Portable `.AppImage` / `.deb` package |

> **Seamless Updates:** Veyak includes an integrated, cryptographically verified updater to keep your studio up-to-date with zero friction.

---

## What Makes Veyak Different

Most modern API tools have drifted away from developer needs: bloated Chromium runtimes that hog gigabytes of memory, forced cloud logins just to ping `localhost`, and sensitive API keys saved in unencrypted text files.

Veyak is engineered around five fundamental pillars:

### 1. 🛡️ Native OS Keyring Encryption
Your API keys, bearer tokens, OAuth credentials, and environment secrets should never live in unencrypted plaintext JSON on disk. Veyak integrates directly with your operating system's native keychain (**macOS Keychain**, **Windows Credential Manager**, and **Linux Secret Service / libsecret**). Secrets are encrypted at the hardware/OS level, ensuring private credentials never leak.

### 2. 🚀 Native Systems Engineering (Rust + Tauri 2)
No Electron. No bundled Chromium. No multi-hundred-megabyte memory footprints. Veyak pairs a high-performance native Rust core with Tauri 2 to achieve:
- **Sub-100ms cold starts** — launches instantly when inspiration strikes.
- **< 60 MB RAM footprint** — leaves your system resources free for heavy local containers, microservices, and compilers.
- **< 20 MB installer** — tiny on disk, lightning-fast to download and update.

### 3. 🌐 True Multi-Protocol Unification
Veyak doesn't treat non-REST protocols as an afterthought. It provides a cohesive, purpose-built interface for every modern protocol:
- **REST / HTTP/2** with full verb and body support.
- **gRPC Studio** with dynamic Protobuf schema reflection, unary calls, and full streaming (client, server, and bidirectional).
- **WebSocket Inspector** with persistent connections and live millisecond timeline feed.
- **GraphQL Studio** with schema introspection, query/mutation builders, and real-time subscriptions over WebSockets.

### 4. ⚡ Embedded QuickJS Scripting Engine
Automate pre-request token generation, sign custom headers, and validate response assertions using modern JavaScript. Unlike tools that spin up heavy background Node.js processes, Veyak embeds the ultra-fast **QuickJS** engine directly within its Rust binary for near-instant execution with zero extra runtime dependencies.

### 5. 🔒 100% Offline-First & Zero Vendor Lock-in
- **Zero Login Walls:** Never create an account just to test an API endpoint.
- **Local-First Privacy:** All collections, environments, and histories are stored in a fast local database on your device (`veyak-db`). Zero telemetry, zero cloud tracking.
- **Frictionless Migration:** Lossless 1-click import and export for Postman Collections, Insomnia v4/v5, and cURL commands. Your data always belongs to you.

---

## Features

### 🔌 Multi-Protocol Engine
- **REST / HTTP:** Full support for `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`. Path and query parameters, dynamic form-data / multipart file uploads, URL-encoded payloads, raw body formats (JSON, XML, YAML, Plaintext), and auth presets (Bearer, OAuth 2.0 PKCE, Basic Auth, API Key).
- **gRPC Studio:** Load `.proto` files or dynamically introspect endpoints via Server Reflection. Execute unary calls, client-streaming, server-streaming, and bidirectional streaming powered by native Rust `tonic`.
- **WebSocket Inspector:** Establish persistent `ws://` and `wss://` connections, monitor real-time frame feeds with millisecond timestamps, inspect incoming/outgoing payloads, and save reusable frame presets.
- **GraphQL Studio:** Run queries, mutations, and subscriptions with automated schema introspection, autocompleting field explorer, variable substitution, and WebSocket transport support.

### ⚙️ Developer Ergonomics & Scripting
- **Pre & Post-Request Scripting:** Write JavaScript hooks with access to request metadata and environment variables. Automatically compute hashes, timestamps, and dynamically chain tokens across requests.
- **Collection-Scoped Environments:** Isolate environments per collection or workspace. Seamlessly interpolate `{{variable}}` syntax across URLs, headers, params, auth, and payloads.
- **Interactive Command Palette (`⌘K` / `Ctrl+K` or `⌘P` / `Ctrl+P`):** Instantly jump between requests, collections, environments, and settings without touching the mouse.
- **Request & Response History Drawer:** Audit past executions, inspect headers and payloads, and re-execute historical requests with one click.
- **Custom Desktop Theme Engine:** Personalize your workspace with a 1-click theme manager, syntax highlighting palettes, and developer monospace fonts (JetBrains Mono, Fira Code, Inter, and system fonts).
- **Resizable Layout & Split Panes:** Persistent, flexible panes tailored for wide monitors and multi-tasking workflows.

---

## Architectural Comparison

| Dimension | Legacy Web-Wrapper Clients (Postman / Insomnia) | Veyak Native Studio |
| :--- | :--- | :--- |
| **Runtime Architecture** | Heavy Electron bundle (Chromium + Node.js) | **Native Rust + Tauri 2 (OS Webview)** |
| **Idle Memory Consumption** | ~600 MB – 1.5 GB RAM | **< 60 MB RAM** |
| **Cold Start Time** | 3.0s – 10.0s | **< 100 ms** |
| **Account Requirement** | Mandatory cloud account / login walls | **100% Offline-First (No account required)** |
| **Secret & Token Security** | Plaintext files on disk or third-party cloud | **Native OS Keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)** |
| **Scripting Runtime** | External Node.js or V8 child processes | **Embedded QuickJS in Rust (Zero-overhead)** |
| **Multi-Protocol Depth** | Fragmented or bolt-on plugins | **Native REST, gRPC (Unary + Streaming), WS, GraphQL Subscriptions** |
| **Data Privacy & Telemetry** | Cloud sync default, usage tracking | **Zero telemetry, 100% local database (`veyak-db`)** |
| **Application Package Size** | ~200 MB – 500 MB download | **< 20 MB download** |

---

## Keyboard Shortcuts

Designed for speed and keyboard-first developer workflows:

| Action | macOS | Windows / Linux |
| :--- | :---: | :---: |
| **Send Request** | <kbd>⌘</kbd> + <kbd>Enter</kbd> | <kbd>Ctrl</kbd> + <kbd>Enter</kbd> |
| **Save Request** | <kbd>⌘</kbd> + <kbd>S</kbd> | <kbd>Ctrl</kbd> + <kbd>S</kbd> |
| **New Request Tab** | <kbd>⌘</kbd> + <kbd>T</kbd> | <kbd>Ctrl</kbd> + <kbd>T</kbd> |
| **Command Palette** | <kbd>⌘</kbd> + <kbd>K</kbd> / <kbd>⌘</kbd> + <kbd>P</kbd> | <kbd>Ctrl</kbd> + <kbd>K</kbd> / <kbd>Ctrl</kbd> + <kbd>P</kbd> |
| **Toggle Request History** | <kbd>⌘</kbd> + <kbd>H</kbd> | <kbd>Ctrl</kbd> + <kbd>H</kbd> |
| **Open Settings** | <kbd>⌘</kbd> + <kbd>,</kbd> | <kbd>Ctrl</kbd> + <kbd>,</kbd> |

---

## Migration & Interoperability

Switching to Veyak takes seconds. You can import existing assets directly:

- **Postman:** Collections (`v2.1`), Environment files, and Global variables.
- **Insomnia:** Workspaces and environments (`v4` & `v5` JSON/YAML).
- **cURL:** Paste any `curl` command directly into the URL bar to auto-populate headers, method, and body.
- **Export:** Export collections at any time in portable Veyak JSON, Postman, or Insomnia formats.

---

## Contributing

Veyak is open-source and built for the developer community. Contributions, suggestions, and bug reports are warmly welcome!

1. Check open [Issues](https://github.com/Veyak/veyak/issues) or start a discussion.
2. Fork the repository, create your feature branch, and submit a [Pull Request](https://github.com/Veyak/veyak/pulls).

---

## Show Your Support

If you love using Veyak, give the project a **Star (⭐️)** on [GitHub](https://github.com/Veyak/veyak)! It helps more developers discover a faster, more private way to test APIs.

---

## License & Author

Distributed under the **MIT License**. See [LICENSE](LICENSE) for details.

Crafted with ❤️ by [**Hrithik Dhakrey**](https://github.com/iamdhakrey)  
- **Website & Docs**: [veyak.iamdhakrey.dev](https://veyak.iamdhakrey.dev/)
