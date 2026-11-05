# Agent guidance

## Project context

Read `project.md` for the architecture and `README.md` for player controls, map format, and detailed build instructions. The active application is **MGame / Overdrive**, the Rust/Macroquad single-player shooter in `src/`. `VBGame/` contains the original VB6 game; `CSharpGame/` contains an earlier .NET 7 WinForms port. Work on the Rust application unless the task explicitly concerns a legacy implementation.

## Working conventions

- Inspect `git status --short` before editing. Preserve unrelated local modifications, untracked files, and deletions; do not reset or restore them as cleanup.
- Keep changes focused on the requested behavior and follow the surrounding Rust style. Avoid unrelated refactors or dependency upgrades.
- Keep gameplay in the window-independent library modules. Translate desktop input, render frames, and play audio in the application modules. Preserve deterministic simulation and the fixed 120 Hz step.
- Keep map document operations in `src/editor.rs` and graphical interaction in `src/editor_ui.rs`.
- Preserve legacy `.mmp` compatibility, ASCII `.arena` support, and original asset paths. Root `Sprites/` supplies compile-time embedded artwork; it is not redundant with the legacy copies.
- Treat `target/` and `dist/` as generated output. Change source and rebuild rather than editing packaged binaries or copies.
- Update `README.md` when controls, player-visible behavior, map rules, or build instructions change. Update `project.md` when architecture or project scope changes.

## Commands

Run from the repository root:

```powershell
cargo run --release
cargo run --release -- --editor
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
```

Use `cargo fmt --all` to apply formatting when needed. For Windows release packaging with checks:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build.ps1 -Check
```

The script fetches locked dependencies, runs formatting/tests/Clippy with `-Check`, builds release, and creates `dist/MGame-Overdrive/`. ZIP creation is opt-in with `-Zip`; omit it for normal rebuilds. An explicit GNU toolchain can be selected with `-Toolchain stable-x86_64-pc-windows-gnu`; the script handles the documented `imm32` import-library workaround.

`Play.cmd` prefers the packaged executable over `target/release/mgame.exe`. Use Cargo directly when checking recent source changes to avoid launching an older package.

## Verification

- For Rust behavior changes, run formatting, the applicable tests, and Clippy; run the full all-targets suite before declaring a broad gameplay change complete.
- Add regression coverage for changed gameplay or editor behavior where it can catch an actual failure. Existing tests live alongside modules and in `tests/maps.rs`, `tests/forest.rs`, and `tests/editor.rs`.
- For visual/input changes, use a graphical desktop to verify the affected interaction. Available scripted checks are `cargo run --release -- --smoke-test` and `cargo run --release -- --editor-smoke-test`; screenshots go under `target/`.
- Smoke tests require a windowing environment; gameplay smoke testing briefly changes display modes. Do not describe a compile or unit test pass as visual verification.
- Documentation-only changes need content and path review, not a game rebuild. Report what was actually checked and any remaining limitations.

## Behaviors to preserve

- Actor collision, dash substeps, projectile path checks, and reachable enemy navigation must remain consistent, including after crates move.
- Terrain distinguishes movement blocking from projectile blocking: ponds allow shots, bushes slow movement, and wooden crates provide indestructible pushable cover.
- Campaign completion requires clearing wave 20 and its remaining enemies/spawns; endless mode continues. Boss scheduling belongs in `src/waves.rs`.
- Editor operations must retain undo/redo, bounded history, unsaved-change protection, legacy tile round trips, and support for maps up to 1024 by 1024 tiles. Play testing must preserve the editor document.
- Asset discovery works from the working directory and beside the executable. Keep packaged maps, music, and the font license aligned with runtime expectations.
