# BRlog

Cross-platform amateur radio logbook — native desktop application.

## Stack

- **Rust** + **Iced** (GPU-accelerated UI via `wgpu`)
- **SQLite** (through `rusqlite`) for local storage
- ADIF import/export
- Single binary, no Electron, no webview

Targets: Windows, Linux, macOS.

## Build

```bash
cargo run            # debug
cargo build --release
```

The first build downloads and compiles Iced and its dependencies (~5–10 min).

## Status

Early development. MVP scope:

- [x] Operator config (callsign, name, QTH, locator, license class)
- [x] Manual QSO entry
- [x] QSO table
- [x] SQLite storage
- [ ] Filter / search over the QSO table
- [ ] ADIF import / export
- [ ] F1–F12 macros (UI grid in place, behavior still to be wired up)

Already implemented beyond the original MVP: light/dark themes, runtime cs/en localization (Fluent), custom title bar with native window controls and resize handles.

Out of scope for now: QRZ.com lookup, LoTW/eQSL upload, map, CAT control, DX cluster, DXCC statistics.

## Bundled fonts

In `assets/fonts/`:

- **Inter** (Regular) — UI font, [rsms.me/inter](https://rsms.me/inter/), SIL OFL license — see `assets/fonts/Inter-LICENSE.txt`
- **JetBrains Mono** (Regular) — monospace for data (callsigns, RST, locator, table), [jetbrains.com/lp/mono](https://www.jetbrains.com/lp/mono/), SIL OFL license — see `assets/fonts/JetBrainsMono-OFL.txt`
- **Lucide** (icon font) — UI icons (window controls, header actions), [lucide.dev](https://lucide.dev/), ISC license — see `assets/fonts/lucide-LICENSE.txt`

## License

GPL-3.0-or-later — see [LICENSE](LICENSE).
