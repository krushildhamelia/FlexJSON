# FlexJSON Parser v2.0.0

FlexJSON is a fault-tolerant, multi-extraction JSON parser capable of ingesting arbitrary text and producing one or more standard-compliant JSON structures. It handles non-standard syntax (alternate separators, mixed quoting, unquoted keys/values) and is designed for high-performance rendering via a virtualized tree viewer.

## 🚀 Features

- **Fault-Tolerant Parsing**: Handles missing commas, unquoted keys, and mixed separators (`:`, `=`, `=>`).
- **Unquoted String Boundary Heuristic**: Intelligently parses unquoted values containing commas and quotes by looking ahead for the next key-value pair.
- **Multi-Fragment Extraction**: Scans arbitrary text (prose, logs, etc.) and extracts all valid JSON-like structures.
- **High-Performance UI**: Renders 100,000+ nodes smoothly using `@tanstack/react-virtual`.
- **Material Design 3**: Modern, accessible UI built with Tailwind CSS and Radix UI.
- **Cross-Platform**: Available as a Web App (WASM-powered) and a Native Desktop App (Tauri v2).

## 🛠️ Tech Stack

- **Core**: Rust (`flexjson-core`) with `logos` for lexing.
- **Frontend**: React 18, TypeScript, Vite.
- **Bindings**: WebAssembly via `wasm-pack` and `serde-wasm-bindgen`.
- **Desktop**: Tauri v2.
- **Styling**: Tailwind CSS + Radix UI Primitives.
- **State Management**: Zustand.

## 📂 Project Structure

```text
flexjson/
├── crates/
│   └── flexjson-core/      # Pure Rust library (Lexer, Parser, Normalizer)
├── packages/
│   ├── core-wasm/          # WASM bindings for the web
│   ├── web/                # React/Vite frontend
│   └── desktop/            # Tauri v2 desktop shell
├── Cargo.toml              # Rust Workspace
├── pnpm-workspace.yaml     # JS Monorepo configuration
└── package.json            # Root scripts
```

## 🏁 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (v8+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Installation

1. Clone the repository.
2. Install JS dependencies:
   ```bash
   pnpm install
   ```
3. Build the WASM core:
   ```bash
   cd packages/core-wasm
   wasm-pack build --target bundler
   ```

### Development

- **Run Web App**:
  ```bash
  pnpm web:dev
  ```
- **Run Desktop App**:
  ```bash
  pnpm desktop:dev
  ```

## 📦 Releases & Distribution

FlexJSON uses [Tauri v2](https://v2.tauri.app/) to package the application into native executables for major operating systems.

### Build Commands

To generate a production-ready installer for your current OS, run:

```bash
pnpm desktop:build
```

### Supported Formats

| Platform | Output Formats |
|---|---|
| **Windows** | `.exe` (NSIS installer), `.msi` (WiX) |
| **macOS** | `.dmg` (Disk Image), `.app` bundle |
| **Linux** | `.deb`, `.AppImage`, `.rpm` |

### CI/CD (GitHub Actions)

The project is configured with a GitHub Actions workflow (`.github/workflows/release.yml`) that automatically builds and attaches assets to GitHub Releases for all three platforms whenever a new tag is pushed.

---

## ⌨️ Key Bindings (Tree Viewer)

| Action | Binding |
|---|---|
| **Toggle Node** | `Space` or `Enter` |
| **Recursive Expand/Collapse** | `Alt + Click` |
| **Search** | `Ctrl + F` (UI Search Bar) |

## 🧪 Testing

Run core parser tests:
```bash
cd crates/flexjson-core
cargo test
```

## 📜 License

MIT
