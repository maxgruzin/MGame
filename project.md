# MGame / Overdrive

## Purpose and current scope

MGame / Overdrive is a native top-down survival shooter inspired by Crimsonland and rebuilt from the original VB6 game. The Rust application supports single-player, Online Party cooperative play for up to eight players, and a built-in graphical map editor. The repository also retains the VB6 original and an earlier C# WinForms port for reference. Online Party uses a new Rust authoritative server rather than the legacy C# networking implementation.

This document describes the inspected working tree, including work that may not yet be committed. It is an orientation guide, not a release or test certification. Consult the source for exact balance values and `README.md` for full player-facing documentation.

## Technology

- Cargo package: `mgame`, version `0.3.0`, Rust edition 2021.
- Macroquad `=0.4.16` provides graphics, windows, input, and audio.
- Audrey `=0.3.0`, with Ogg Vorbis and WAV support, decodes music for duration calculation.
- The default `desktop` feature enables audio/Audrey and the `mgame` binary. `party-server` builds with `--no-default-features`; it uses the shared simulation without initializing a window or audio. Serde/JSON and Macroquad's `glam-serde` feature encode network state.
- `Cargo.lock` records dependency resolution; release builds use thin LTO and one codegen unit.
- Windows is the documented tested platform. The Rust code also targets Linux/macOS through Macroquad, but platform support should be verified before claiming it. Linux audio builds may need ALSA development libraries.

## Architecture

`src/lib.rs` exports the arena, editor document, level, simulation, wave, and party/network modules. These support tests without opening a game window. The binary in `src/main.rs` integrates these systems with Macroquad and the desktop-facing modules.

| Location | Responsibility |
| --- | --- |
| `src/main.rs` | Startup and CLI options, arena discovery, menus and application flow, input, fixed-step loop, editor integration, smoke-test orchestration |
| `src/simulation.rs` | Game state, deterministic random sequence, movement, weapons, enemies, projectiles, damage, pickups, wave execution, sound events; `STEP = 1/120` second |
| `src/waves.rs` | Campaign length, boss scheduling, enemy rosters, threat costs, and progression scaling |
| `src/level.rs` | Map parsing/loading, terrain, collision, movable crates, reachability, and navigation |
| `src/arenas.rs` | Eighteen embedded forest arena layouts |
| `src/editor.rs` | Window-independent grid/document model, editing, bounded undo/redo, validation, serialization, and saving |
| `src/editor_ui.rs` | Editor tools, dialogs, camera, overview, and graphical interaction |
| `src/render.rs` | World/HUD rendering and camera transforms |
| `src/menu.rs` | Arena paging logic |
| `src/assets.rs` | Embedded BMP atlas decoding and transparency handling |
| `src/sound.rs` | Generated effects, external music discovery/playback, and decoded track durations |
| `src/settings.rs` | Display/audio/effects preferences and persistence |
| `src/party.rs` | Versioned messages, lobby rules, stable player slots/colors, authoritative render snapshots |
| `src/party_net.rs` | Nonblocking TCP connection, length-prefixed JSON, bounded read/write queues |
| `src/party_server.rs` | Admission, input validation/rate limits, timeouts, host transfer, fixed-step authoritative room |
| `src/party_client.rs` | Nonblocking client, map handoff, snapshots, heartbeat and round-trip measurement |
| `src/online_ui.rs` | Server address/name fields, waiting room difficulty controls, online weapon/input handling and spectator flow |
| `src/bin/party-server.rs` | Separate headless server executable and stage/mode CLI configuration |
| `server/Dockerfile`, `compose.yaml` | Linux server image and one-room Docker Compose deployment |
| `src/party_smoke.rs`, `tests/party.rs` | Desktop party smoke scenario and window-independent multiplayer regressions |

The application supplies input to fixed-step simulation and renders interpolated state. Simulation emits sound events for the audio layer. The editor retains its document when entering and returning from an unsaved play test.

Online Party keeps one authoritative `Game` on the server at 120 Hz. `Game.player` remains the solo-compatible primary slot; `allies` hold additional actors. Each player updates movement/weapons once, then enemies/projectiles/pickups/waves update once for the whole world. Enemies select reachable living targets using per-player navigation fields, refreshed around movable crates. The starting player count and host-selected `Difficulty` percentages fix encounter, health, and damage multipliers; each player weapon has its own damage adjustment. Revive beacons are created by lethal damage at the exact death position. Helpers contribute additive progress, empty/contested beacons drain at twice the base rate, and the deterministic simulation RNG selects a connected fallen teammate to revive at full health.

Clients submit inputs at 30 Hz and receive render snapshots at 20 Hz, interpolating player/enemy positions without running their own gameplay simulation. Maps transfer once at match start; snapshots include movable crates, projectiles, pickups, beacons, and sound events. The client places its own player in the existing HUD slot while preserving network slot/color IDs. The protocol limits commands to 2 KiB and snapshots to 2 MiB, bounds queues, rejects invalid inputs, expires controls after 250 ms without new input, and removes silent/slow connections. TCP is deliberately simple for this first direct-IP mode; prediction, matchmaking, authentication, and mid-match reconnects are not implemented. One process hosts one in-memory room, reset after the last departure.

The server chooses a built-in arena or custom `.arena`/`.mmp` map and campaign/endless mode. Online maps are capped at 128 x 128 to bound eight navigation fields; solo/editor maps retain their 1024 x 1024 limit. The first connected player controls match start and post-match return to the lobby; host departure transfers that role. All players must ready up before starting and can switch freely between all five weapons. Host-only difficulty changes clear readiness; validated settings travel in the lobby and match-start messages and remain immutable during play. Party protocol 2 carries these settings and player names; client and server builds must match. The connection UI drains queued text while inactive so gameplay keystrokes cannot leak into its fields. Escape/focus loss never pauses the server.

## Game systems

Players select an arena and either a 20-wave campaign or endless mode. Bosses appear on waves 7, 15, and 20. Combat includes five weapons (rifle, shotgun, rail carbine, rockets, grenades), dashing, reloading, healing, kill multipliers, and temporary damage, immunity, and speed pickups. Nine regular monster types and three bosses use different movement and attacks.

Forest terrain includes solid trees, ponds that block actors but allow shots, slowing bushes, healing pads, and indestructible wooden crates pushed by the player. Crates affect cover and navigation. Explosives use splash damage and cover checks; player explosives can hurt the player in solo mode. Online Party disables player explosive damage to the team and prevents pushing crates into living allies.

The editor supports brush/rectangle/fill tools, tile picking, resizing, undo/redo, validation, saving, and play testing. Undo history is bounded to approximately 32 MiB. Large maps use visible-tile rendering and a cached overview.

## Content and runtime files

| Path | Role |
| --- | --- |
| `Maps/` | Nine original maps in `.mmp` format |
| `levels/` | Editable `.arena` maps, including `Crossfire.arena` |
| `Sprites/` | Original artwork used by the Rust build through `include_bytes!` |
| `assets/fonts/` | Embedded font and its SIL Open Font License |
| `Music/` | External `.ogg` / `.wav` tracks copied into the portable package |
| `VBGame/` | Original VB6 source, executable, dependencies, maps, and sprites |
| `CSharpGame/` | Earlier .NET 7 Windows Forms port, including networking code |
| `scripts/build.ps1` | Windows checks, release build, and portable ZIP packaging |
| `target/`, `dist/` | Ignored build artifacts, smoke screenshots, and release packages |

Maps are rectangular ASCII grids from 5 to 1024 tiles per side; each tile spans 48 world units. `.mmp` and `.arena` files share the current grid parser. The README documents supported tile characters and fallback behavior for absent or unreachable markers.

The arena catalog scans `Maps/` and `levels/` in the working directory and beside the executable. Filenames are deduplicated case-insensitively, with working-directory copies taking precedence. `--map <path>` selects a file directly. `--editor` opens the editor and can be combined with `--map`.

Music is discovered from `Music/` in the working directory and beside the executable. It plays during active stages, with a 20-second gap between songs and no consecutive repeats when multiple tracks exist. The Sound setting controls both effects and music.

Settings live at `%LOCALAPPDATA%/MGame/settings.cfg` on Windows, or under the XDG configuration directory with a `~/.config` fallback. Font/artwork and generated sound effects require no runtime download; music and additional maps remain external files.

## Development and validation

Use a stable Rust toolchain. Windows MSVC builds require Visual Studio C++ build tools and the Windows SDK; GNU builds require MinGW-w64. See `AGENTS.md` for routine commands and `README.md` for detailed usage.

`Play.cmd` launches the packaged executable first, then the local release build. `MapEditor.cmd` adds `--editor`. The packaging script produces `dist/MGame-Overdrive/mgame.exe` and, only with `-Zip`, `dist/MGame-Overdrive-Windows.zip`, including maps, music, README, editor launcher, and the font license.

Unit and integration tests cover simulation, wave progression, collisions, navigation, terrain, powerups, map compatibility, party rules/real sockets, and editor operations, including million-tile maps. Graphical smoke modes (`--smoke-test`, `--editor-smoke-test`, `--party-smoke-test`) capture screenshots under `target/` and exit. They require a graphical desktop and complement the automated tests. `scripts/test-party-server.py` exercises an empty running server, including a Docker deployment, with eight TCP clients.
