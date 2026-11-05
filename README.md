# MGame / Overdrive

A native Rust rebuild of the original Crimsonland-style shooter. The VB6 and C# sources are retained for reference; the new game lives in `src/`.

## Play

On Windows, double-click **Play.cmd**, or run `dist/MGame-Overdrive/mgame.exe` after building. The executable embeds its font, original forest/monster artwork, sounds, and **18 forest arenas** (15 more than the first Rust release). The adjacent `Maps/` and `levels/` folders supply additional arenas, including all nine original maps. `dist/MGame-Overdrive-Windows.zip` is the portable release package. Next/Previous changes both the selected arena and preview, with a visible page count; even a catalog with just three arenas responds to these buttons.

Choose an arena, choose six-wave or endless mode, then click **Deploy**. Clear six increasingly difficult waves to secure a sector and advance to another arena. Between waves you recover a little health; marked medical pads and dropped medkits provide additional healing. Consecutive kills build a score multiplier.

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

Display & Audio is available from the arena menu and pause screen. It offers 720p, 900p, 1080p, 1440p, and 4K window sizes, fullscreen, sound, and effects toggles. Windows can also be resized freely. UI and world scale with the display, and mouse coordinates use the same camera transform. High-DPI desktops may render more physical pixels than the requested window dimensions. On Windows, switching to another application pauses gameplay. Preferences are saved under `%LOCALAPPDATA%/MGame/settings.cfg` on Windows (XDG config directory on Linux, `~/.config` fallback).

## What changed

- 120 Hz fixed simulation with interpolated rendering, float positions, responsive acceleration, normalized diagonal movement, and collision substeps that slide along walls. Dashes obey the same collision system.
- Five distinct weapons with magazines, reloads, spread, muzzle effects, trails, hit sparks, knockback, generated sound, and fast projectiles checked along their full movement path. Rockets explode on impact with splash damage. Grenades lob about 175 units ahead, can clear cover, and explode **2 seconds after firing**; their landing area and fuse are visible. Explosions have distance falloff, trees and wooden blocks shield actors, and your own explosives can hurt you.
- Six animated monster types reuse the original sprite sheets. Ravagers pursue; spitters strafe and fire three-shot acid spreads; thornback brutes absorb damage; hunters predict movement and fire fast spike volleys; brood bombers lob timed mortar grenades over cover; siege beasts fire explosive projectiles. The advanced types appear from waves 3, 5, and 6 respectively. Shared navigation is recalculated around pushed crates, and local separation reduces crowding. Reachable spawns have visible warnings.
- Escalating waves, finite objectives or endless survival, healing pickups, kill chains, and a dash cooldown that rewards movement.
- Green grass, trees, bushes, ponds, wooden cover, and larger healing crates restore the original forest style. Ponds block players and monsters but allow shooting. Bushes reduce movement to 52% speed. Wooden blocks are indestructible, block actors and shots, and can only be pushed by the player; walls, ponds, other crates, and monsters constrain pushing.
- Temporary pickups grant **quad damage for 12 seconds**, **immortality for 8 seconds**, or **double movement speed for 10 seconds**. Timers appear in the HUD. Recollecting the same powerup refreshes its duration. A rotating powerup is supplied each wave and monsters can drop additional ones. Dropped medical crates restore 30 health, while permanent healing crates regenerate health nearby.
- Shadows, particle effects, camera follow, a minimap, fullscreen/high-DPI rendering, and in-game restart/level selection remain available.

This rebuild is single-player. The old C# networking code has not been ported.

## Build from source

Install a stable Rust toolchain. On Windows the standard MSVC toolchain also needs the Visual Studio C++ build tools and Windows SDK; the GNU toolchain needs MinGW-w64. Macroquad supplies the graphics/window/audio layer; no game engine editor is required.

```powershell
cargo run --release
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

To check, build, and create a portable Windows package:

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

The arena selector scans `Maps/` and `levels/` in the working directory and beside the executable. Place `.mmp` or `.arena` files there and restart the game. Duplicate filenames appear once, with working-directory copies taking precedence. Files can also be selected from anywhere:

```powershell
cargo run --release -- --map "Maps/(1)forest.mmp"
.\dist\MGame-Overdrive\mgame.exe --map "C:\My Maps\arena.arena"
```

Maps are rectangular ASCII grids, 5–256 tiles per side. Each tile is 48 world units. See `levels/Crossfire.arena` for an editable example.

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

`src/arenas.rs` contains the 18 embedded layouts. `src/level.rs` handles parsing, terrain, pushing, collision, reachability, and navigation. `src/simulation.rs` contains deterministic gameplay, independent of window/input state. `src/assets.rs` decodes the embedded original BMP atlases with transparency; `src/menu.rs` handles paging. Rendering, audio, settings, and desktop integration live in the other `src/` modules.

Tests cover wall sliding, dash tunneling, diagonal speed, camera coordinates, navigation for both enemy sizes across every original and built-in arena, terrain rules, pushing and cover, powerup effects/expiry, the 2-second grenade fuse, splash damage, advanced enemy weapons, arena uniqueness, paging, and sustained combat. The executable also accepts `--smoke-test`: from the repository root it runs scripted gameplay, captures `target/smoke-game.png`, `target/smoke-menu.png`, `target/smoke-menu-page2.png`, and settings/fullscreen/resized screenshots, then exits automatically. This requires a graphical desktop and briefly changes the test window's display mode; it does not save test settings.

The embedded Barlow font is by Jeremy Tribby, distributed under the SIL Open Font License; see `assets/fonts/OFL.txt`. Original artwork is embedded from the repository's `Sprites/` folder. New effects and sound are generated by the game; no assets need downloading at runtime.
