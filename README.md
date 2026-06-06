# ConfUI

A standalone settings window for applications that use TOML configuration files.
Ships as a separate binary — it reads and writes TOML only, so its lifecycle is fully independent of the parent app.

|             | Light                                                       | Dark                                                        |
| ----------- | ----------------------------------------------------------- | ----------------------------------------------------------- |
| **macOS**   | ![macOS Light (Japanese)](docs/assets/mac-light-jp.png)     | ![macOS Dark (English)](docs/assets/mac-dark-en.png)        |
| **Windows** | ![Windows Light (German)](docs/assets/windows-light-de.png) | ![Windows Dark (Japanese)](docs/assets/windows-dark-jp.png) |
| **Linux**   | ![Linux Light (English)](docs/assets/linux-light-en.png)    | ![Linux Dark (German)](docs/assets/linux-dark-de.png)       |

**日本語:** [README.ja.md](README.ja.md)

---

## Features

- **Schema-driven** — describe your settings in a TOML file; no code required
- **16 built-in languages** — Arabic, Mandarin, German, English, French, Hindi, Italian, Japanese, Korean, Dutch, Portuguese, Russian, Spanish, Swedish, Turkish, Vietnamese
- **Light / Dark** — OS-adaptive or forced; fully customizable color palette
- **External change detection** — watches the config file and prompts on conflict
- **Cross-platform** — macOS, Windows, Linux

## Quick Start

**1. Write a `schema.toml`:**

```toml
icon_style = "rounded"
theme      = "os"
lang       = "os"

[[tabs]]
id    = "general"
label = { en = "General", ja = "一般" }
icon  = "settings"

[[tabs.fields]]
key    = "server.host"
label  = { en = "Host", ja = "ホスト" }
type   = "string"
widget = "text_input"

[[tabs.fields]]
key    = "server.port"
label  = { en = "Port", ja = "ポート" }
type   = "number"
widget = "drag_value"
min    = 1
max    = 65535
```

**2. Build:**

```bash
cd confui
CONFUI_SCHEMA=/path/to/your/schema.toml make
```

**3. Launch from your app:**

```rust
std::process::Command::new("path/to/confui").spawn()?;
```

## Embedding in Your App

### macOS `.app` bundle

```
MyApp.app/Contents/MacOS/
├── MyApp
└── confui
```

```rust
let confui = std::env::current_exe()?
    .parent().unwrap()
    .join("confui");
std::process::Command::new(confui).spawn()?;
```

### Windows

Place `ConfUI.exe` in the same directory as your executable and install both together.

### Cargo workspace (Git submodule)

```toml
# Parent app's Cargo.toml
[workspace]
members = ["myapp", "confui"]
```

Point the schema via `CONFUI_SCHEMA` when building. The `make` default is `../schema.toml` (the parent app's schema, one level up from `confui/`). When building directly with `cargo build` without `CONFUI_SCHEMA`, the fallback is `demo/schema.toml`.

## Build

| Scenario                              | Schema used                                                         |
| ------------------------------------- | ------------------------------------------------------------------- |
| `make` (no override)                  | `../schema.toml` — parent app's schema, one level up from `confui/` |
| `make SCHEMA=/path/to/schema.toml`    | the specified path                                                  |
| `cargo build` without `CONFUI_SCHEMA` | `demo/schema.toml` — standalone dev fallback                        |

```bash
cd confui

# First run / when changing icon style
make icons                          # Download Material Symbols (rounded)
make icons ICON_STYLE=outlined      # Variants: rounded / outlined / sharp

# macOS / Linux binary
make                                # Release binary
make SCHEMA=/path/to/schema.toml    # Override schema path

# Windows .exe (cross-compile from macOS; requires mingw-w64 + x86_64-pc-windows-gnu)
make win
make win-zip                        # Zip for distribution

make clean
```

Output:

| Path                             | Description                               |
| -------------------------------- | ----------------------------------------- |
| `target/release/confui`          | Binary for embedding in a macOS/Linux app |
| `dist/confui-windows/ConfUI.exe` | Windows executable                        |

## Documentation

Full schema reference, widget guide, theming, and localization:

**https://emotiongraphics.jp/docs/confui/**

## License

MIT License
