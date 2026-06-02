# MD Reader

A fast, native Markdown reader with **Mermaid.js** diagram support, built in Rust using [`wry`](https://github.com/tauri-apps/wry) + [`tao`](https://github.com/tauri-apps/tao) (the same WebView stack powering Tauri).

## Features

- ⚡ **Near-instant startup** — Rust binary + pulldown-cmark
- 🔒 **Low memory** — ~15–28 MB RSS
- 📊 **Mermaid diagrams** — click to zoom, scroll-to-zoom, drag to pan
- 🌙 **Dark theme** — teal/amber palette, sidebar TOC, scroll progress bar
- 🖨️ **Print support** — clean print CSS
- ⌨️ **Keyboard shortcuts** — `[` sidebar, `f` fullscreen, `Esc` close
- 🖥️ **Cross-platform** — Windows (WebView2), Linux (WebKit2GTK), macOS (WKWebView)
- 📂 **File dialog** — opens if no CLI argument given
- 🔌 **Install/uninstall** — register file associations, Start Menu shortcut, Windows Control Panel uninstall entry
- 🏷️ **Version badge** — release version shown in sidebar footer

## Prerequisites

**Windows:** WebView2 Runtime (pre-installed on Win10/11).

**Linux:**
```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev pkg-config libssl-dev
```

**macOS:** Xcode Command Line Tools.

## Build from source

```bash
git clone <repo-url>
cd md-reader
cargo build --release
```

Binary: `target/release/md-reader` (or `md-reader.exe` on Windows).

### Source assets

Place these files in `src/` before building:

```
src/
├── dist/
│   └── mermaid.min.js   ← Mermaid v11 offline copy
├── icon/
│   └── favicon.ico      ← App icon (Windows)
├── main.rs
├── markdown.rs
├── install.rs
└── template.rs
```

## Usage

```bash
# Open file dialog
md-reader

# Open specific file
md-reader document.md

# Install (register file associations + Start Menu)
md-reader install

# Uninstall (remove all registrations)
md-reader uninstall
```

## Offline Mermaid

The app searches for `dist/mermaid.min.js` in this order:

1. Next to the binary
2. `../../src/dist/` relative to binary (development)
3. Next to the `.md` file
4. User data dir (`%LOCALAPPDATA%\MDReader\dist\`)
5. CDN fallback (defensive)

After `md-reader install`, Mermaid is copied into the install directory automatically.

## Install details

### Windows

- Copies binary to `%LOCALAPPDATA%\MDReader\`
- Registers `.md` / `.markdown` file associations (HKCU)
- Adds Control Panel uninstall entry (HKCU\...\Uninstall\MDReader)
- Creates Start Menu shortcut
- Embeds `favicon.ico` into `.exe` via `build.rs` + `app.rc`

### Linux

- Writes `~/.local/share/applications/md-reader.desktop`
- Updates `~/.config/mimeapps.list` for `text/markdown`
- Calls `xdg-mime default` if available

### macOS

- Creates `~/Applications/MD Reader.app` bundle with `Info.plist`
- Registers for `net.daringfireball.markdown` via LaunchServices

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `[` | Toggle sidebar |
| `f` | Toggle fullscreen |
| `Esc` | Close zoom / sidebar / fullscreen |
| Scroll (on diagram) | Zoom in/out |
| Click+drag (on zoom) | Pan diagram |

## CI/CD

GitHub Actions automatically:

| Trigger | Action |
|---------|--------|
| PR / push to `main` | Lint (fmt + clippy) + `cargo check` on Win/Linux/macOS |
| Push tag `v*.*.*` | Full release build → GitHub Release with 3 binaries |

To release:

```bash
git tag v0.0.63
git push origin v0.0.63
```

Artifacts:
- `md-reader-windows-x86_64.zip`
- `md-reader-linux-x86_64.tar.gz`
- `md-reader-macos-universal.tar.gz`

## Architecture

```
md-reader/
├── src/
│   ├── main.rs        # Entry point: window, subcommands, event loop, cache
│   ├── markdown.rs    # Markdown→HTML (pulldown-cmark, Mermaid extraction)
│   ├── install.rs     # Install/uninstall logic (Windows, Linux, macOS)
│   ├── template.rs    # HTML/CSS/JS page template
│   ├── dist/
│   │   └── mermaid.min.js
│   └── icon/
│       └── favicon.ico
├── build.rs           # Icon embedding (Windows-only)
├── app.rc             # Resource script for favicon.ico
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
└── Cargo.toml
```

## Performance

| Metric | Value |
|--------|-------|
| Cold startup | ~0.15–0.35 s |
| Memory (idle) | ~15–28 MB |
| Binary size (stripped) | ~8–14 MB |
| Markdown parse (1 MB file) | ~4 ms |

## License

MIT
