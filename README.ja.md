# ConfUI

TOML 形式の設定ファイルを持つアプリケーション向けの，独立した設定ウィンドウ。
別プロセスのバイナリとして動作し，TOML の読み書きのみを担うため，親アプリのライフサイクルに依存しません。

|             | Light                                                          | Dark                                                       |
| ----------- | -------------------------------------------------------------- | ---------------------------------------------------------- |
| **macOS**   | ![macOS Light（日本語）](docs/assets/mac-light-jp.png)         | ![macOS Dark（英語）](docs/assets/mac-dark-en.png)         |
| **Windows** | ![Windows Light（ドイツ語）](docs/assets/windows-light-de.png) | ![Windows Dark（日本語）](docs/assets/windows-dark-jp.png) |
| **Linux**   | ![Linux Light（英語）](docs/assets/linux-light-en.png)         | ![Linux Dark（ドイツ語）](docs/assets/linux-dark-de.png)   |

**English:** [README.md](README.md)

---

## 機能

- **スキーマ駆動** — TOML ファイルで設定を記述するだけ。コード不要
- **16 言語組み込み** — アラビア語・中国語（普通話）・ドイツ語・英語・フランス語・ヒンディー語・イタリア語・日本語・韓国語・オランダ語・ポルトガル語・ロシア語・スペイン語・スウェーデン語・トルコ語・ベトナム語
- **Light / Dark テーマ** — OS 追従または固定。カラーパレットを完全カスタマイズ可能
- **外部変更検知** — 設定ファイルを監視し，競合時にダイアログを表示
- **クロスプラットフォーム** — macOS・Windows・Linux

## クイックスタート

**1. `schema.toml` を作成する:**

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

**2. ビルド:**

```bash
cd confui
CONFUI_SCHEMA=/path/to/your/schema.toml make
```

**3. 親アプリから起動:**

```rust
std::process::Command::new("path/to/confui").spawn()?;
```

## 親アプリへの組み込み

### macOS `.app` バンドル

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

`ConfUI.exe` を親アプリの実行ファイルと同じディレクトリに置き，インストーラーで両方を配置します。

### Cargo workspace（Git submodule）

```toml
# 親アプリの Cargo.toml
[workspace]
members = ["myapp", "confui"]
```

スキーマはビルド時に `CONFUI_SCHEMA` 環境変数で指定します。`make` のデフォルトは `../schema.toml`（`confui/` の一階層上，親アプリのスキーマ）です。`CONFUI_SCHEMA` を設定せず `cargo build` を直接実行した場合は `demo/schema.toml` にフォールバックします。

## ビルド

| 場面                                 | 使用されるスキーマ                                          |
| ------------------------------------ | ----------------------------------------------------------- |
| `make`（上書きなし）                 | `../schema.toml` — `confui/` の一階層上，親アプリのスキーマ |
| `make SCHEMA=/path/to/schema.toml`   | 指定パス                                                    |
| `CONFUI_SCHEMA` なしの `cargo build` | `demo/schema.toml` — スタンドアロン開発用フォールバック     |

```bash
cd confui

# 初回 / アイコンスタイル変更時
make icons                          # Material Symbols をダウンロード（rounded）
make icons ICON_STYLE=outlined      # バリアント: rounded / outlined / sharp

# macOS / Linux バイナリ
make                                # リリースバイナリ
make SCHEMA=/path/to/schema.toml    # スキーマパスを上書き

# Windows .exe（macOS ホストからクロスコンパイル。mingw-w64 + x86_64-pc-windows-gnu が必要）
make win
make win-zip                        # 配布用 zip 化

make clean
```

出力先：

| パス                             | 説明                                           |
| -------------------------------- | ---------------------------------------------- |
| `target/release/confui`          | macOS / Linux アプリに同梱するための生バイナリ |
| `dist/confui-windows/ConfUI.exe` | Windows 実行ファイル                           |

## ドキュメント

スキーマリファレンス・ウィジェットガイド・テーマ・ローカライズの詳細:

**https://emotiongraphics.jp/docs/confui/**

## ライセンス

MIT License
