# memcrate

> A portable, markdown-native, locally-owned personal context vault for AI tools. Three verbs. One vault. Any tool.

```bash
npm install -g memcrate
```

This package is a thin wrapper. On install it downloads the `memcrate` binary for your platform from [GitHub Releases](https://github.com/memcrate/memcrate/releases), verifies its published sha256, and puts it on your PATH. There are no dependencies.

## Quick start

```bash
memcrate                 # asks where your vault goes, then sets everything up
```

It creates the vault and installs the /load, /save, and /pin skills for Claude Code, Claude Desktop, and Codex. Then start your tool and run `/load`.

## What it is

Every AI coding tool eventually loses context. Sessions end, tools change, and each one has to be told from scratch what you are working on. Memcrate is a local markdown vault plus three verbs (`/save`, `/pin`, `/load`) that any AI tool can read and write. Your context lives in plain `.md` files you own.

Full docs at [memcrate.dev](https://memcrate.dev).

## Supported platforms

| Platform | Prebuilt binary |
| --- | --- |
| Linux x64 | yes |
| macOS Apple Silicon | yes |
| Windows x64 | yes |
| Anything else (including Intel Macs) | use `cargo install memcrate` |

## Other install methods

```bash
cargo install memcrate                                                            # any platform with Rust
curl -fsSL https://raw.githubusercontent.com/memcrate/memcrate/main/install.sh | sh  # Linux / macOS
```

```powershell
irm https://raw.githubusercontent.com/memcrate/memcrate/main/install.ps1 | iex     # Windows
```

## Troubleshooting

**"binary not found"** means the install script did not run, usually because of `--ignore-scripts` or a script-blocking npm policy. Fix it with `npm rebuild memcrate`, or install via cargo or the shell one-liners above.

## License

MIT. Source at [github.com/memcrate/memcrate](https://github.com/memcrate/memcrate).
