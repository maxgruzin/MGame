# MGame / Overdrive

A native Rust rebuild of the original Crimsonland-style shooter. The original VB6 game is in `VBGame/`, the C# port is in `CSharpGame/`, and the new game lives in `src/`.

## Play

On Windows, double-click **Play.cmd**, or run `dist/MGame-Overdrive/mgame.exe` after building. The executable embeds its font, original forest/monster artwork, sounds, and **18 forest arenas** (15 more than the first Rust release). The adjacent `Maps/` and `levels/` folders supply additional arenas, including all nine original maps. Builds produce the ready-to-play `dist/MGame-Overdrive/` folder. Add `-Zip` to the build command only when a ZIP archive is wanted. Next/Previous changes both the selected arena and preview, with a visible page count; even a catalog with just three arenas responds to these buttons.

Choose an arena, choose 20-wave or endless mode, then click **Deploy**. Clear 20 increasingly difficult waves to secure a sector and advance to another arena. **Waves 7, 15, and 20 are boss battles**, announced during the countdown and on arrival with a warning sound and banner. Active bosses have a named health bar and attack faster below half health. Between waves you recover a little health; marked medical pads and dropped medkits provide additional healing. Consecutive kills build a score multiplier.

During a running stage, songs from `Music/` play randomly, with 20 seconds of silence after each song and no consecutive repeats when multiple tracks are available. Music stops in menus, the editor, on pause, and when a stage ends; resuming starts a fresh random song. The All Audio toggle controls both music and effects; their volumes can be adjusted separately in Display & Audio. Add `.ogg` (Vorbis) or `.wav` tracks to `Music/` in the working directory or beside the executable, then restart the game. The portable package includes the music folder.

| Control | Action |
| --- | --- |
| WASD | Move independently of aim |
| Mouse | Aim |
| Hold left mouse | Fire |
| Space / Left Shift | Dash; briefly evade damage |
| 1 / 2 / 3 / 4 / 5, mouse wheel | Rifle / shotgun / rail carbine / rockets / grenades |
| Walk into wooden blocks | Push them into a firing shelter at reduced movement speed |
| R | Reload; empty weapons also reload when firing |
| Escape | Pause / resume |
| F11 / Alt+Enter | Toggle fullscreen |
| Up / Down, Enter | Choose an arena and deploy |

Display & Audio is available from the arena menu and pause screen. It offers 720p, 900p, 1080p, 1440p, and 4K window sizes, fullscreen, audio, and effects toggles, plus separate Sound Effects and Music volume controls (0-100% in 5% steps using the -/+ buttons). Default volumes are 90% for sound effects and 70% for music. All Audio mutes both channels without changing their saved volumes. Windows can also be resized freely. UI and world scale with the display, and mouse coordinates use the same camera transform. High-DPI desktops may render more physical pixels than the requested window dimensions. On Windows, switching to another application pauses gameplay. Preferences are saved under `%LOCALAPPDATA%/MGame/settings.cfg` on Windows (XDG config directory on Linux, `~/.config` fallback).

## What changed

- 120 Hz fixed simulation with interpolated rendering, float positions, responsive acceleration, normalized diagonal movement, and collision substeps that slide along walls. Dashes obey the same collision system.
- The player has round mint armor, a dark helmet and visor, and a rifle, with articulated arms and legs. Legs step in the movement direction while bent arms sway and hold the weapon toward your aim. Animation follows distance traveled and stops when blocked or idle. Bullet trails grow from the muzzle without drawing backward across the player.
- Five distinct weapons with magazines, reloads, spread, muzzle effects, trails, hit sparks, knockback, generated sound, and fast projectiles checked along their full movement path. Rockets explode on impact with splash damage. Grenades lob about 175 units ahead, can clear cover, and explode **2 seconds after firing**; their landing area and fuse are visible. Explosions have distance falloff, trees and wooden blocks shield actors, and your own explosives can hurt you.
- Nine regular monster types and three bosses animate using the original sprite sheets, with distinct colors, silhouettes, and weapon organs. Ravagers pursue; spitters fire acid spreads; thornbacks absorb damage; hunters fire fast spike volleys; brood bombers lob grenades over cover; siege beasts fire rockets. New **stalkers (wave 8)** weave quickly toward you, **lancers (wave 11)** telegraph fast, heavy shots, and **sentinels (wave 16)** fire five-shot fans. Enemies look ahead along clear routes around trees and prioritize those routes when crowd separation, weaving, or strafing would push them into obstacles. Shared navigation is recalculated around pushed crates; enemies beside shifted cover reconnect to nearby clear routes instead of freezing when their tile center is blocked. Local separation reduces crowding. Reachable spawns have visible warnings.
- Bosses: the **Thorn Tyrant (wave 7)** fires circular thorn volleys; the **Brood Matriarch (wave 15)** lobs three grenades across your path; the **Overlord (wave 20)** combines rocket fans and mortar attacks. Boss health is **3x the previous amount**: base health is 1,980 / 3,150 / 4,800 respectively, before wave scaling. All attacks have visible windups, and bosses fit through the same paths as brutes.
- Every wave now contains **2.5x the previous enemy count, rounded up**, in both campaign and endless mode (wave 1 has 30 enemies). Boss waves retain one boss, with extra regular enemies filling out the increased total. Each wave increases the base encounter budget, monster health (+4 percentage points per wave), movement speed (+1.2 percentage points), and spawn frequency. All enemy movement, including bosses, is **10% faster** on top of the existing wave scaling and speed cap. Endless mode continues after wave 20; the campaign ends only after all 20 waves, including the final boss and its escorts, are cleared.
- The rail carbine deals **230 damage**, holds **8 rounds**, fires every **0.36 seconds**, and reloads in **1.05 seconds**. Shotgun damage is **21 per pellet** (up from 19); its handling is unchanged. Healing pickups, kill chains, and a dash cooldown reward movement.
- Green grass, trees, bushes, ponds, wooden cover, and larger healing crates restore the original forest style. Ponds block players and monsters but allow shooting. Bushes reduce movement to 52% speed. Wooden blocks are indestructible, block actors and shots, and can only be pushed by the player; walls, ponds, other crates, and monsters constrain pushing.
- Temporary pickups grant **quad damage for 12 seconds**, **immortality for 8 seconds**, or **double movement speed for 10 seconds**. Drops use picture icons: a bullet burst for extra damage, a shield for immortality, a moving boot for speed, and a medical bag for dropped health. Matching icons accompany the active-effect names and timers in the HUD. The speed boost displays a blue circle around the player while active. Recollecting the same powerup refreshes its duration. A rotating powerup is supplied each wave and monsters can drop additional ones. Dropped medical bags restore 30 health, while healing spots retain their original crate artwork and regenerate health nearby.
- Shadows, particle effects, camera follow, a minimap, fullscreen/high-DPI rendering, and in-game restart/level selection remain available.

## Online Party

Click **Online Party** in the arena menu. Enter the server's IPv4 or IPv6 address, TCP port (default **7777**), and your name, then connect to the waiting room. Tab moves between fields; Ctrl+A and Ctrl+V select and paste text. Up to **8 players** can join one room. The first player is the host (`*`); the role passes to the next player if the host leaves.

Every player clicks **I Am Ready**, then the host clicks **Start Match** once everyone is ready. All five weapons are available throughout the match: use **1-5 or the mouse wheel** to switch, just as in solo play. Ammunition reloads normally. The server selects the arena and campaign/endless mode and sends the map to every client, so local map files do not need to match.

Players spawn together near the arena start. Each has a consistent armor/ring color, their **player name above their character**, and an overhead health bar; the team list shows names, health, fallen players, and departures. Allies can move and shoot through each other. Friendly fire, including self damage from player explosives, is disabled in this mode. Crates cannot be pushed into a living ally.

When a teammate falls, a **revive beacon appears immediately at their exact death position**, without needing an enemy kill. An alive player within its **64-unit radius**, with a clear line to the beacon and no enemy contesting it, contributes one second of progress per second. The beacon needs **10 seconds of total progress**: two allies take 5 seconds, three take about 3.3 seconds. If everyone moves away or an enemy contests the beacon, progress **drains by two seconds per second**, down to zero; returning resumes from what remains.

Each completed beacon revives **one randomly selected connected fallen teammate**, regardless of join order, at **100 health**, with all magazines full and **3 seconds of protection**. The revived player appears in a safe spot near the beacon. Every death creates its own beacon, and unused beacons disappear when no connected fallen teammates remain. Fallen players spectate living allies; **Tab** changes the followed player. Normal healing and between-wave recovery do not revive anyone.

The waiting room's **Difficulty** panel replaces weapon selection. Everyone can see the settings; only the host can adjust them with **-/+ buttons in 5% steps**. A change clears everyone's ready status. Settings stay fixed during a match and are retained when returning to the waiting room; an empty room resets them to defaults.

| Setting | Range | Default |
| --- | --- | --- |
| Additional enemy count per additional player | 0% to +300% | +75% |
| Additional enemy damage per additional player | 0% to +300% | +15% |
| Additional enemy health per additional player | 0% to +300% | +15% |
| Player damage adjustment, separately for each of the five weapons | -50% to +100% | 0% |

Enemy multipliers use the number of players at match start: `1 + (starting players - 1) * bonus / 100`. They stay fixed after deaths or disconnections. For example, with the defaults, four players face **3.25x enemy count**, **1.45x health**, and **1.45x damage**. Enemy counts round up. These multipliers apply on top of normal wave progression; boss waves retain exactly one scheduled boss. Count settings also scale spawn frequency and the active-enemy limit. All enemy attacks scale, including melee, bullets, rockets, and grenades. Each weapon's damage adjustment applies equally to all players, including pellets and explosive damage, and combines with quad damage.

**Escape opens the party menu; online combat keeps running**, including while a player changes settings or switches applications. The whole team loses when no connected player remains alive. After defeat or victory, the host can return everyone to the waiting room and start another match. A running match does not accept late joins or reconnections; a disconnected player leaves that match. Once everyone leaves, the server resets its room automatically.

This is a direct-IP party mode with no accounts, matchmaking, or room password. Anyone able to reach the server can join an available seat. Use the same client/server build (party protocol 2); update the server as well as all clients for these settings. The server owns movement, hits, health, drops, and readiness; clients send controls and render authoritative updates. Aim for a low-latency connection: this implementation uses TCP and interpolated snapshots, without local movement prediction.

### Run the Linux server with Docker Compose

Install Docker Engine and its Compose plugin, copy this repository to the Linux server, and run from the repository root:

```sh
docker compose up --build -d
docker compose logs -f party
```

The separate `party-server` executable runs headlessly, without graphics/audio initialization or game assets. Allow **TCP 7777** through the server firewall and forward that TCP port on the router when hosting over the Internet. Players enter the Linux server's reachable IP; for Docker on the same PC, use `127.0.0.1`. Stop the service with `docker compose down`.

The default stage is **01 / Forest Outpost**, in the 20-wave campaign. The server chooses the stage. Optional `.env` settings beside `compose.yaml` select a built-in arena (zero-based 0-17) and the published port:

```dotenv
PARTY_ARENA=3
PARTY_PORT=7777
```

Apply changes with `docker compose up -d` (this restarts an active room). Stage selection is a server setting; it is not a lobby control. To host a custom map or endless mode, create `compose.override.yaml`, for example:

```yaml
services:
  party:
    command: ["--bind", "0.0.0.0:7777", "--map", "/maps/Crossfire.arena", "--endless"]
    volumes:
      - ./levels:/maps:ro
```

Custom `.arena` and legacy `.mmp` maps are supported. Online arenas are limited to **128 x 128 tiles** to bound server navigation work; the single-player/editor limit remains 1024 x 1024. The map must have safe room for all players near its start. Every server process hosts one room. The container runs as a non-root user with a read-only filesystem; room state is held in memory and resets on restart.

To run the server without Docker:

```sh
cargo run --locked --release --no-default-features --bin party-server -- --bind 0.0.0.0:7777 --arena 0
# Optional: --map /path/to/map.mmp --endless
```

For a socket-level check against an **empty test server**, run `python scripts/test-party-server.py 127.0.0.1 7777`. It joins eight clients, starts a match, checks admission/weapon switching/host rules, then disconnects them. It intentionally occupies the room during the test.

## Build from source

Install a stable Rust toolchain. On Windows the standard MSVC toolchain also needs the Visual Studio C++ build tools and Windows SDK; the GNU toolchain needs MinGW-w64. Macroquad supplies the graphics/window/audio layer; no game engine editor is required.

```powershell
cargo run --release
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

To check, build, and create a portable Windows package:

Double-click **Rebuild.cmd**. It runs formatting checks, tests, Clippy, and a release build, updates `dist/MGame-Overdrive/` for **Play.cmd**, and keeps the window open to show success or errors. It uses installed Rust or the existing local GNU toolchain under `target/tools/` when available. Optional arguments are forwarded to the build script (for example, `Rebuild.cmd -Zip`). Close the game before rebuilding so Windows can replace its executable.

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build.ps1 -Check
```

For an installed GNU toolchain:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build.ps1 -Toolchain stable-x86_64-pc-windows-gnu -Check
```

The GNU helper accommodates minimal toolchains missing the ordinary `imm32` import-library name by using the equivalent import library already supplied by the `winapi` dependency. It does not modify your toolchain.

The code also targets desktop Linux/macOS through Macroquad, but this change was tested on Windows. Linux builds with audio may require ALSA development libraries. Dependency versions are recorded in `Cargo.lock`.

## Load or create levels

Click **Map Editor** in the arena menu to edit the selected arena, double-click **MapEditor.cmd**, or launch directly with `Play.cmd --editor`. Built-in arenas open as editable copies. Use **New** for a blank map with a tree border and starting markers; choose a size preset or enter any width and height from **5 to 1024 tiles** (up to 1,048,576 tiles). **Resize** preserves the top-left area; cropping can be undone.

The editor includes a labeled terrain palette, brush sizes from 1 to 31 tiles, filled rectangles, flood fill, a clickable overview, and play-testing. Rendering is limited to the visible tiles; zoomed-out views use a cached overview. Undo history retains changed cells and is bounded to approximately 32 MiB, so older edits eventually expire.

| Editor control | Action |
| --- | --- |
| Left drag / right drag | Paint / erase with the selected tool |
| 1–8 | Choose terrain or a marker |
| B / R / F | Brush / filled rectangle / flood fill |
| `[` / `]` | Smaller / larger brush |
| Alt + click | Pick a tile, including legacy decorative terrain |
| Mouse wheel | Zoom around the pointer |
| Middle drag / Space + left drag / arrow keys | Pan |
| Click or drag the overview | Jump to another part of the map |
| Home / G | Fit the whole map / toggle grid |
| Ctrl+Z / Ctrl+Y or Ctrl+Shift+Z | Undo / redo |
| Ctrl+N / Ctrl+O | New / open |
| Ctrl+S / Ctrl+Shift+S | Save / save as |
| F5 / F6 | Play test / return to editor from a test |
| Escape | Close a dialog or return to the arena menu |

**Open** lists built-in and discovered maps and accepts a path to any `.arena` or `.mmp` file. File fields support Ctrl+A, Ctrl+V, arrow keys, Home/End, Backspace, and Delete. **Save as** defaults to `levels/My Arena.arena`; maps saved under `levels/` or `Maps/` appear when you return to the arena menu. Existing files require confirmation when using Save as. Unsaved edits are protected when opening a different map, creating a new map, leaving the editor, or closing the window.

**Validate** checks that the map has one player start, a connected play area, and reachable enemy/medical markers. Unfinished maps can still be saved and reopened. **Play test** runs the current map without saving and preserves the editor document; click **Back to Editor**, press F6, or use the pause menu to return. Playing very large, densely populated maps can still cost more than editing them.

```powershell
Play.cmd --editor --map "levels/Crossfire.arena"
cargo run --release -- --editor
```

The arena selector scans `Maps/` and `levels/` in the working directory and beside the executable. Place `.mmp` or `.arena` files there and restart the game. Duplicate filenames appear once, with working-directory copies taking precedence. Files can also be selected from anywhere:

```powershell
cargo run --release -- --map "Maps/(1)forest.mmp"
.\dist\MGame-Overdrive\mgame.exe --map "C:\My Maps\arena.arena"
```

Maps are rectangular ASCII grids, 5–1024 tiles per side. Each tile is 48 world units. See `levels/Crossfire.arena` for an editable example.

| Tile | Meaning |
| --- | --- |
| `.` / `0` | Floor |
| `P` / `1` | Player start (first marker is used) |
| `E` / `2` | Enemy entry point, walkable |
| `+` / `3` | Medical pad |
| `#` / `T` / `4` / `5` / `6` | Solid tree/obstacle; blocks movement and shots |
| `~` | Pond; blocks movement, permits shots |
| `B` | Bush; slows movement, permits shots |
| `C` | Indestructible wooden block; player can push it |
| `a`–`y` | Walkable decorative terrain from the original VB6 format |

The map boundary is always solid, including maps without a wall border. Maps without a player marker use the first floor tile. Maps without reachable entry markers get automatic entry points on reachable distant floor tiles. Disconnected entry markers are ignored. Invalid maps show an error in the selector and cannot be deployed.

## Verification and structure

`src/party.rs` defines room rules and snapshots; `party_net.rs` implements bounded TCP framing; `party_server.rs` runs the authoritative fixed-step room; `party_client.rs` receives state; and `online_ui.rs` implements the connection screen, waiting room, spectator view, and online controls. The separate server entry point is `src/bin/party-server.rs`, packaged by `server/Dockerfile` and `compose.yaml`.

`tests/party.rs` checks lobby authorization, eight-player spawning, custom difficulty, weapon switching, death beacons, revive draining/acceleration/random selection, damage, disconnections, framing, and eight clients over real sockets. `cargo run --release -- --party-smoke-test` opens a desktop client with a local test server and seven bots, captures `target/smoke-party-connect.png`, `target/smoke-party-lobby.png`, `target/smoke-party-game.png`, `target/smoke-party-revive.png`, and `target/smoke-party-results.png`, verifies a revive and return to lobby, then exits. No externally hosted room is used. The server feature configuration can be checked with `cargo test --locked --no-default-features --all-targets` and the corresponding Clippy command.

`src/arenas.rs` contains the 18 embedded layouts. `src/level.rs` handles parsing, terrain, pushing, collision, reachability, and navigation. `src/simulation.rs` contains deterministic gameplay, independent of window/input state; `src/waves.rs` defines campaign progression, enemy rosters, and boss scheduling. `src/assets.rs` decodes the embedded original BMP atlases with transparency; `src/menu.rs` handles paging. Rendering, audio, settings, and desktop integration live in the other `src/` modules.

`src/editor.rs` contains the window-independent map document, editing operations, bounded undo/redo, validation, and file saving. `src/editor_ui.rs` implements the graphical editor. Editor tests cover legacy round trips, continuous strokes, region filling, unique player placement, resize recovery, save failures, and million-tile maps. `--editor-smoke-test` captures `target/smoke-editor.png`, `target/smoke-editor-large.png`, and `target/smoke-editor-dialog.png` on a graphical desktop, then exits.

Tests cover wall sliding, dash tunneling, diagonal speed, camera coordinates, navigation for both enemy sizes across every original and built-in arena, terrain rules, pushing and cover, powerup effects/expiry, the 2-second grenade fuse, splash damage, advanced enemy weapons, arena uniqueness, paging, and sustained combat. The executable also accepts `--smoke-test`: from the repository root it runs scripted gameplay, captures `target/smoke-game.png`, `target/smoke-menu.png`, `target/smoke-menu-page2.png`, and settings/fullscreen/resized screenshots, then exits automatically. This requires a graphical desktop and briefly changes the test window's display mode; it does not save test settings.

The embedded Barlow font is by Jeremy Tribby, distributed under the SIL Open Font License; see `assets/fonts/OFL.txt`. Original artwork is embedded from the repository's `Sprites/` folder. New effects and sound are generated by the game; no assets need downloading at runtime.
