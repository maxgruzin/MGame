#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod assets;
mod editor_ui;
mod menu;
mod online_ui;
mod party_smoke;
mod render;
mod settings;
mod sound;

use macroquad::prelude::*;
use mgame::{
    level::Level,
    simulation::{Game, Input, STEP},
    waves::CAMPAIGN_WAVES,
};
use render::{text, BG, MINT, MUTED, PANEL};
use settings::{Settings, RESOLUTIONS};
use std::path::PathBuf;

fn window_conf() -> Conf {
    let s = Settings::load();
    let (w, h) = RESOLUTIONS[s.resolution];
    Conf {
        window_title: "MGame / Overdrive".into(),
        window_width: w as i32,
        window_height: h as i32,
        fullscreen: s.fullscreen,
        high_dpi: true,
        window_resizable: true,
        sample_count: 4,
        ..Default::default()
    }
}
struct Entry {
    name: String,
    source: Option<PathBuf>,
    builtin: usize,
}
impl Entry {
    fn load(&self) -> Result<Level, String> {
        match &self.source {
            Some(path) => Level::load(path),
            None => Ok(Level::builtin(self.builtin)),
        }
    }
}
fn catalog() -> Vec<Entry> {
    let mut levels: Vec<Entry> = (0..mgame::arenas::NAMES.len())
        .map(|i| Entry {
            name: Level::builtin(i).name,
            source: None,
            builtin: i,
        })
        .collect();
    let mut roots = vec![PathBuf::from("Maps"), PathBuf::from("levels")];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            roots.push(parent.join("Maps"));
            roots.push(parent.join("levels"));
        }
    }
    let mut files = vec![];
    for root in roots {
        if let Ok(dir) = std::fs::read_dir(root) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
                    s.eq_ignore_ascii_case("mmp") || s.eq_ignore_ascii_case("arena")
                }) {
                    if let Ok(path) = path.canonicalize() {
                        if !files.iter().any(|existing: &PathBuf| {
                            existing
                                .file_name()
                                .map(|s| s.to_string_lossy().to_lowercase())
                                == path.file_name().map(|s| s.to_string_lossy().to_lowercase())
                        }) {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }
    files.sort();
    for path in files {
        levels.push(Entry {
            name: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            source: Some(path),
            builtin: 0,
        });
    }
    levels
}
fn ui_scale() -> f32 {
    (screen_width() / 1280.0)
        .min(screen_height() / 720.0)
        .max(0.45)
}
fn ui_begin(scale: f32) -> (f32, f32) {
    let (w, h) = (screen_width() / scale, screen_height() / scale);
    set_camera(&render::viewport_camera(vec2(w, h) * 0.5, vec2(w, h)));
    (w, h)
}
fn button(rect: Rect, label: &str, scale: f32, selected: bool) -> bool {
    let mouse = Vec2::from(mouse_position()) / scale;
    let hover = rect.contains(mouse);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if selected {
            Color::from_hex(0x21483f)
        } else if hover {
            Color::from_hex(0x223842)
        } else {
            PANEL
        },
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        if hover || selected {
            MINT
        } else {
            Color::from_hex(0x28414b)
        },
    );
    text(
        label,
        rect.x + 18.0,
        rect.y + rect.h * 0.5 + 6.0,
        20.0,
        if hover || selected { MINT } else { WHITE },
    );
    hover && is_mouse_button_pressed(MouseButton::Left)
}
fn apply_settings(settings: &Settings) {
    set_fullscreen(settings.fullscreen);
    if !settings.fullscreen {
        let (w, h) = RESOLUTIONS[settings.resolution];
        request_new_screen_size(w as f32, h as f32);
    }
}
fn volume_control(label: &str, value: &mut u8, x: f32, y: f32, scale: f32) {
    text(
        &format!("{label}    {value}%"),
        x + 18.0,
        y + 32.0,
        20.0,
        WHITE,
    );
    if button(Rect::new(x + 340.0, y, 54.0, 48.0), "-", scale, false) {
        *value = value.saturating_sub(5);
    }
    if button(Rect::new(x + 406.0, y, 54.0, 48.0), "+", scale, false) {
        *value = value.saturating_add(5).min(100);
    }
}
fn settings_panel(s: &mut Settings, scale: f32, w: f32, h: f32) -> bool {
    let x = w * 0.5 - 230.0;
    let y = h * 0.5 - 240.0;
    draw_rectangle(x - 32.0, y - 56.0, 524.0, 592.0, PANEL);
    text("DISPLAY & AUDIO", x, y - 12.0, 31.0, MINT);
    let (rw, rh) = RESOLUTIONS[s.resolution];
    if button(
        Rect::new(x, y + 18.0, 460.0, 52.0),
        &format!("RESOLUTION    {rw} x {rh}   >"),
        scale,
        false,
    ) {
        s.resolution = (s.resolution + 1) % RESOLUTIONS.len();
        apply_settings(s);
    }
    if button(
        Rect::new(x, y + 82.0, 460.0, 52.0),
        &format!(
            "FULLSCREEN    {}   / F11",
            if s.fullscreen { "ON" } else { "OFF" }
        ),
        scale,
        s.fullscreen,
    ) {
        s.fullscreen = !s.fullscreen;
        apply_settings(s);
    }
    if button(
        Rect::new(x, y + 146.0, 460.0, 52.0),
        &format!(
            "PARTICLES & SHAKE    {}",
            if s.effects { "ON" } else { "OFF" }
        ),
        scale,
        s.effects,
    ) {
        s.effects = !s.effects;
    }
    if button(
        Rect::new(x, y + 210.0, 460.0, 52.0),
        &format!("ALL AUDIO    {}", if s.sound { "ON" } else { "OFF" }),
        scale,
        s.sound,
    ) {
        s.sound = !s.sound;
    }
    volume_control("SOUND EFFECTS", &mut s.sound_volume, x, y + 274.0, scale);
    volume_control("MUSIC", &mut s.music_volume, x, y + 332.0, scale);
    text(
        "Window can also be resized freely.",
        x,
        y + 413.0,
        18.0,
        MUTED,
    );
    button(
        Rect::new(x, y + 436.0, 460.0, 50.0),
        "SAVE & BACK",
        scale,
        true,
    ) || is_key_pressed(KeyCode::Escape)
}

#[macroquad::main(window_conf)]
async fn main() {
    prevent_quit();
    if let Ok(mut font) =
        load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Barlow-Medium.ttf"))
    {
        font.set_filter(FilterMode::Linear);
        macroquad::text::set_default_font(font);
    }
    render::prepare_font();
    let args: Vec<String> = std::env::args().collect();
    let smoke = args.iter().any(|s| s == "--smoke-test");
    let editor_smoke = args.iter().any(|s| s == "--editor-smoke-test");
    let party_smoke = args.iter().any(|s| s == "--party-smoke-test");
    let mut settings = Settings::load();
    if smoke || editor_smoke || party_smoke {
        settings.fullscreen = false;
        settings.sound = false;
        set_fullscreen(false);
        request_new_screen_size(1280.0, 720.0);
    }
    let mut audio = sound::Audio::new().await;
    let assets = assets::Assets::new();
    let mut entries = catalog();
    let mut selected = 0usize;
    let mut message = String::new();
    if let Some(arg) = args.iter().position(|s| s == "--map") {
        if let Some(path) = args.get(arg + 1) {
            entries.push(Entry {
                name: PathBuf::from(path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                source: Some(path.into()),
                builtin: 0,
            });
            selected = entries.len() - 1;
        } else {
            message = "--map requires a path to an .mmp or .arena file".into();
        }
    }
    let mut preview = entries[selected].load();
    let mut editor = if args.iter().any(|s| s == "--editor") || editor_smoke {
        if args.iter().any(|s| s == "--map") {
            match editor_ui::Editor::from_entry(&entries[selected]) {
                Ok(e) => Some(e),
                Err(e) => {
                    message = e;
                    Some(editor_ui::Editor::blank())
                }
            }
        } else {
            Some(editor_ui::Editor::blank())
        }
    } else {
        None
    };
    if let Some(e) = &mut editor {
        if !message.is_empty() {
            e.notice(std::mem::take(&mut message));
        }
    }
    let mut testing_editor = false;
    let mut online: Option<online_ui::OnlineUi> = None;
    let mut party_smoke = if party_smoke {
        let (smoke, ui) = party_smoke::Smoke::new();
        online = Some(ui);
        Some(smoke)
    } else {
        None
    };
    let mut editor_frames = 0;
    let mut game: Option<Game> = if smoke { Some(smoke_game()) } else { None };
    let mut paused = false;
    let mut show_settings = false;
    let mut endless = false;
    let mut accumulator = 0.0;
    let mut camera_center = game
        .as_ref()
        .map_or_else(|| Level::builtin(0).spawn, |g| g.player.pos);
    let mut pending_dash = false;
    let mut pending_reload = false;
    let mut pending_weapon = None;
    let mut frame = 0usize;
    let mut page = selected / 7;
    loop {
        let frame_time = if smoke {
            1.0 / 60.0
        } else {
            get_frame_time().min(0.1)
        };
        let scale = ui_scale();
        if is_key_pressed(KeyCode::F11)
            || (is_key_pressed(KeyCode::Enter)
                && (is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt)))
        {
            settings.fullscreen = !settings.fullscreen;
            apply_settings(&settings);
            if let Err(e) = settings.save() {
                message = format!("Could not save settings: {e}");
            }
        }
        clear_background(BG);
        if editor.is_some() && !testing_editor {
            audio.update_music(false, settings.music_volume);
            let e = editor.as_mut().unwrap();
            if editor_smoke {
                e.smoke_step(editor_frames);
            }
            match e.frame(scale) {
                editor_ui::Action::None => {}
                editor_ui::Action::Quit => break,
                editor_ui::Action::Exit => {
                    editor = None;
                    entries = catalog();
                    selected = selected.min(entries.len() - 1);
                    page = selected / menu::PAGE_SIZE;
                    preview = entries[selected].load();
                }
                editor_ui::Action::Play(level) => {
                    camera_center = level.spawn;
                    game = Some(Game::new(level, false));
                    testing_editor = true;
                    paused = false;
                    accumulator = 0.0;
                }
            }
            if editor_smoke {
                if editor_frames == 5 {
                    get_screen_data().export_png("target/smoke-editor.png");
                }
                if editor_frames == 15 {
                    get_screen_data().export_png("target/smoke-editor-large.png");
                }
                if editor_frames == 25 {
                    get_screen_data().export_png("target/smoke-editor-dialog.png");
                }
                if editor_frames == 30 {
                    break;
                }
                editor_frames += 1;
            }
            next_frame().await;
            continue;
        }
        if is_quit_requested() {
            if testing_editor {
                game = None;
                audio.update_music(false, settings.music_volume);
                testing_editor = false;
                if let Some(e) = &mut editor {
                    if matches!(e.confirm_quit(), editor_ui::Action::Quit) {
                        break;
                    }
                }
                next_frame().await;
                continue;
            }
            break;
        }
        if let Some(party) = &mut online {
            let exit = party.frame(&assets, &mut audio, &mut settings, frame_time, scale);
            if party_smoke.as_mut().is_some_and(|smoke| smoke.frame(party)) {
                break;
            }
            if exit {
                online = None;
            }
            next_frame().await;
            continue;
        }
        let mut return_to_menu = false;
        let mut restart = false;
        let mut next_level = false;
        if let Some(g) = game.as_mut() {
            if !show_settings && is_key_pressed(KeyCode::Escape) {
                paused = !paused;
                accumulator = 0.0;
            }
            // A long frame often means the window was dragged or suspended. Never catch up seconds of combat.
            if (get_frame_time() > 0.5 || !app_focused()) && !smoke {
                paused = true;
                accumulator = 0.0;
            }
            let aim_offset = (Vec2::from(mouse_position())
                - vec2(screen_width(), screen_height()) * 0.5)
                / scale
                * 0.12;
            let target = g.player.pos + aim_offset.clamp_length_max(85.0);
            if !paused {
                camera_center = camera_center.lerp(target, 1.0 - (-7.0 * frame_time).exp());
            }
            let shake = if settings.effects && !paused {
                vec2((g.time * 93.0).sin(), (g.time * 117.0).cos()) * g.shake
            } else {
                Vec2::ZERO
            };
            let cam = render::camera(camera_center + shake, scale);
            if !paused && !show_settings && g.player.health > 0.0 && !g.victory {
                pending_dash |=
                    is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::LeftShift);
                pending_reload |= is_key_pressed(KeyCode::R);
                for (key, index) in [
                    (KeyCode::Key1, 0),
                    (KeyCode::Key2, 1),
                    (KeyCode::Key3, 2),
                    (KeyCode::Key4, 3),
                    (KeyCode::Key5, 4),
                ] {
                    if is_key_pressed(key) {
                        pending_weapon = Some(index);
                    }
                }
                let wheel = mouse_wheel().1;
                if wheel != 0.0 {
                    pending_weapon = Some((g.player.weapon + if wheel > 0.0 { 1 } else { 4 }) % 5);
                }
                let movement = vec2(
                    (is_key_down(KeyCode::D) as i32 - is_key_down(KeyCode::A) as i32) as f32,
                    (is_key_down(KeyCode::S) as i32 - is_key_down(KeyCode::W) as i32) as f32,
                );
                let aim = cam.screen_to_world(Vec2::from(mouse_position()));
                accumulator += frame_time;
                while accumulator >= STEP {
                    let mut input = Input {
                        movement,
                        aim,
                        fire: is_mouse_button_down(MouseButton::Left),
                        dash: pending_dash,
                        reload: pending_reload,
                        weapon: pending_weapon,
                    };
                    if smoke {
                        input = Input {
                            movement: match frame {
                                40..100 => Vec2::X,
                                100..140 => -Vec2::X,
                                _ => Vec2::ZERO,
                            },
                            aim: g
                                .enemies
                                .iter()
                                .min_by(|a, b| {
                                    a.pos
                                        .distance_squared(g.player.pos)
                                        .total_cmp(&b.pos.distance_squared(g.player.pos))
                                })
                                .map_or(g.player.pos + Vec2::X * 250.0, |e| e.pos),
                            fire: true,
                            dash: false,
                            weapon: Some((frame / 40) % 5),
                            ..Default::default()
                        };
                    }
                    g.update(input, STEP);
                    accumulator -= STEP;
                    pending_dash = false;
                    pending_reload = false;
                    pending_weapon = None;
                }
            } else {
                accumulator = 0.0;
                pending_dash = false;
                pending_reload = false;
                pending_weapon = None;
            }
            audio.play(&g.sounds, settings.sound, settings.sound_volume);
            g.sounds.clear();
            render::world(
                g,
                &assets,
                &cam,
                if paused { 1.0 } else { accumulator / STEP },
                settings.effects,
            );
            render::hud(g, scale);
            let ended = g.player.health <= 0.0 || g.victory;
            show_mouse(paused || ended || show_settings);
            if paused || ended {
                let (w, h) = ui_begin(scale);
                draw_rectangle(0.0, 0.0, w, h, Color::new(0.01, 0.025, 0.04, 0.82));
                if !show_settings {
                    let x = w * 0.5 - 200.0;
                    let y = h * 0.5 - 160.0;
                    text(
                        if g.victory {
                            "SECTOR SECURED"
                        } else if ended {
                            "SIGNAL LOST"
                        } else {
                            "PAUSED"
                        },
                        x,
                        y,
                        38.0,
                        if ended && !g.victory {
                            Color::from_hex(0xff7788)
                        } else {
                            MINT
                        },
                    );
                    text(
                        &format!(
                            "{} kills  /  {} points  /  wave {}",
                            g.kills, g.score, g.wave
                        ),
                        x,
                        y + 35.0,
                        19.0,
                        MUTED,
                    );
                    if button(
                        Rect::new(x, y + 65.0, 400.0, 52.0),
                        if g.victory {
                            "NEXT ARENA"
                        } else if ended {
                            "TRY AGAIN"
                        } else {
                            "RESUME"
                        },
                        scale,
                        true,
                    ) || is_key_pressed(KeyCode::Enter)
                    {
                        if g.victory {
                            next_level = true;
                        } else if ended {
                            restart = true;
                        } else {
                            paused = false;
                        }
                    }
                    if button(
                        Rect::new(x, y + 130.0, 400.0, 52.0),
                        "DISPLAY & AUDIO",
                        scale,
                        false,
                    ) {
                        show_settings = true;
                    }
                    if button(
                        Rect::new(x, y + 195.0, 400.0, 52.0),
                        if testing_editor {
                            "BACK TO EDITOR"
                        } else {
                            "ARENA SELECT"
                        },
                        scale,
                        false,
                    ) {
                        return_to_menu = true;
                    }
                }
                set_default_camera();
            } else {
                render::crosshair();
            }
            if testing_editor {
                let (w, _) = ui_begin(scale);
                if button(
                    Rect::new(w * 0.5 - 140.0, 112.0, 280.0, 36.0),
                    "BACK TO EDITOR / F6",
                    scale,
                    true,
                ) || is_key_pressed(KeyCode::F6)
                {
                    return_to_menu = true;
                }
                set_default_camera();
            }
        } else {
            show_mouse(true);
            let (w, h) = ui_begin(scale);
            for i in 0..32 {
                let x = i as f32 * 64.0 - (get_time() as f32 * 8.0) % 64.0;
                draw_line(x, 0.0, x, h, 1.0, Color::new(0.15, 0.28, 0.3, 0.15));
            }
            text("M G A M E", 64.0, 75.0, 24.0, MINT);
            text("OVERDRIVE", 60.0, 139.0, 70.0, WHITE);
            text(
                "MOVE FAST. FIND AN ANGLE. SURVIVE.",
                64.0,
                173.0,
                20.0,
                MUTED,
            );
            text(
                &format!(
                    "{} ARENAS  /  PAGE {} OF {}",
                    entries.len(),
                    page + 1,
                    entries.len().div_ceil(menu::PAGE_SIZE)
                ),
                64.0,
                226.0,
                20.0,
                MINT,
            );
            if !show_settings {
                if button(
                    Rect::new(w - 440.0, 55.0, 250.0, 36.0),
                    "ONLINE PARTY",
                    scale,
                    false,
                ) {
                    online = Some(online_ui::OnlineUi::new());
                }
                let old = selected;
                if is_key_pressed(KeyCode::Down) {
                    selected = (selected + 1) % entries.len();
                    page = selected / 7;
                }
                if is_key_pressed(KeyCode::Up) {
                    selected = (selected + entries.len() - 1) % entries.len();
                    page = selected / 7;
                }
                for (i, entry) in entries.iter().enumerate().skip(page * 7).take(7) {
                    if button(
                        Rect::new(64.0, 246.0 + (i % 7) as f32 * 43.0, 450.0, 36.0),
                        &entry.name,
                        scale,
                        i == selected,
                    ) {
                        selected = i;
                    }
                }
                if button(
                    Rect::new(64.0, 556.0, 216.0, 36.0),
                    "< PREVIOUS",
                    scale,
                    false,
                ) {
                    selected = menu::page_selection(selected, entries.len(), false);
                    page = selected / menu::PAGE_SIZE;
                }
                if button(Rect::new(298.0, 556.0, 216.0, 36.0), "NEXT >", scale, false) {
                    selected = menu::page_selection(selected, entries.len(), true);
                    page = selected / menu::PAGE_SIZE;
                }
                if old != selected {
                    preview = entries[selected].load();
                    message.clear();
                }
            }
            let x = w - 636.0;
            if let Ok(level) = &preview {
                render::minimap(
                    level,
                    Some(level.spawn),
                    vec2(x, 244.0),
                    vec2(320.0, 222.0),
                    &[],
                );
                text(
                    &format!(
                        "{} x {} / {} entry points",
                        level.width,
                        level.height,
                        level.portals.len()
                    ),
                    x,
                    494.0,
                    18.0,
                    MUTED,
                );
            } else if let Err(error) = &preview {
                text(
                    "MAP COULD NOT BE LOADED",
                    x,
                    280.0,
                    22.0,
                    Color::from_hex(0xff7788),
                );
                for (i, line) in wrap(error, 48).iter().enumerate() {
                    text(line, x, 313.0 + i as f32 * 22.0, 17.0, MUTED);
                }
            }
            text("WASD", x + 362.0, 258.0, 23.0, WHITE);
            text("Move", x + 362.0, 284.0, 18.0, MUTED);
            text("MOUSE", x + 362.0, 329.0, 23.0, WHITE);
            text("Aim + hold LMB", x + 362.0, 355.0, 18.0, MUTED);
            text("SPACE", x + 362.0, 400.0, 23.0, WHITE);
            text("Dash through danger", x + 362.0, 426.0, 18.0, MUTED);
            text("1 / 2 / 3 / 4 / 5", x + 362.0, 471.0, 23.0, WHITE);
            text("Switch weapons", x + 362.0, 497.0, 18.0, MUTED);
            if !show_settings {
                let mode_label = if endless {
                    "MODE / ENDLESS SURVIVAL".to_string()
                } else {
                    format!("MODE / CLEAR {CAMPAIGN_WAVES} WAVES")
                };
                if button(Rect::new(x, 521.0, 560.0, 42.0), &mode_label, scale, false) {
                    endless = !endless;
                }
                if button(
                    Rect::new(x, 577.0, 560.0, 56.0),
                    "DEPLOY   /   ENTER",
                    scale,
                    true,
                ) || (is_key_pressed(KeyCode::Enter)
                    && !is_key_down(KeyCode::LeftAlt)
                    && !is_key_down(KeyCode::RightAlt))
                {
                    if let Ok(level) = &preview {
                        camera_center = level.spawn;
                        game = Some(Game::new(level.clone(), endless));
                        paused = false;
                        accumulator = 0.0;
                    }
                }
                if button(
                    Rect::new(64.0, h - 65.0, 216.0, 40.0),
                    "DISPLAY & AUDIO",
                    scale,
                    false,
                ) {
                    show_settings = true;
                }
                if button(
                    Rect::new(298.0, h - 65.0, 216.0, 40.0),
                    "MAP EDITOR",
                    scale,
                    false,
                ) {
                    match editor_ui::Editor::from_entry(&entries[selected]) {
                        Ok(e) => {
                            editor = Some(e);
                            message.clear();
                        }
                        Err(e) => message = e,
                    }
                }
                if button(
                    Rect::new(w - 164.0, 55.0, 100.0, 36.0),
                    "QUIT",
                    scale,
                    false,
                ) {
                    break;
                }
            }
            text(
                "F11 / FULLSCREEN      MAP EDITOR / Create, paint, save and play",
                x,
                h - 29.0,
                16.0,
                MUTED,
            );
            set_default_camera();
        }
        if return_to_menu {
            game = None;
            paused = false;
            show_settings = false;
            testing_editor = false;
        }
        if restart || next_level {
            if testing_editor {
                if let Some(e) = &editor {
                    if let Ok(level) = e.doc.validate("Editor play test") {
                        camera_center = level.spawn;
                        game = Some(Game::new(level, false));
                        paused = false;
                        accumulator = 0.0;
                    }
                }
                next_frame().await;
                continue;
            }
            if next_level {
                selected = (selected + 1) % entries.len();
                page = selected / 7;
            }
            preview = entries[selected].load();
            match &preview {
                Ok(level) => {
                    camera_center = level.spawn;
                    game = Some(Game::new(level.clone(), endless));
                    paused = false;
                    accumulator = 0.0;
                }
                Err(e) => {
                    message = e.clone();
                    game = None;
                }
            }
        }
        if show_settings {
            let (w, h) = ui_begin(scale);
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.01, 0.02, 0.03, 0.90));
            if settings_panel(&mut settings, scale, w, h) {
                show_settings = false;
                if let Err(e) = settings.save() {
                    message = format!("Could not save settings: {e}");
                }
            }
            set_default_camera();
        }
        if !message.is_empty() {
            let (w, _) = ui_begin(scale);
            draw_rectangle(0.0, 0.0, w, 27.0, Color::from_hex(0x5e2835));
            text(&message, 12.0, 19.0, 16.0, WHITE);
            set_default_camera();
        }
        if smoke {
            if matches!(frame, 72 | 88 | 120 | 170) {
                get_screen_data().export_png(&format!("target/smoke-player-{frame}.png"));
            }
            if frame == 45 {
                get_screen_data().export_png("target/smoke-monsters.png");
            }
            if frame == 180 {
                get_screen_data().export_png("target/smoke-game.png");
                let mut warning = Game::new(Level::builtin(0), false);
                warning.wave = 6;
                warning.intermission = 4.0;
                camera_center = warning.player.pos;
                game = Some(warning);
            }
            if frame == 190 {
                get_screen_data().export_png("target/smoke-boss-warning.png");
            }
            if frame == 210 {
                game = None;
            }
            if frame == 215 {
                get_screen_data().export_png("target/smoke-menu.png");
            }
            if frame == 216 {
                selected = menu::page_selection(selected, entries.len(), true);
                page = selected / menu::PAGE_SIZE;
                preview = entries[selected].load();
            }
            if frame == 218 {
                get_screen_data().export_png("target/smoke-menu-page2.png");
            }
            if frame == 220 {
                selected = menu::page_selection(selected, entries.len(), false);
                page = selected / menu::PAGE_SIZE;
                preview = entries[selected].load();
                assert_eq!(selected, 0);
                show_settings = true;
            }
            if frame == 225 {
                get_screen_data().export_png("target/smoke-settings.png");
            }
            if frame == 230 {
                settings.fullscreen = true;
                set_fullscreen(true);
            }
            if frame == 240 {
                get_screen_data().export_png("target/smoke-fullscreen.png");
            }
            if frame == 245 {
                settings.fullscreen = false;
                settings.resolution = 1;
                set_fullscreen(false);
                request_new_screen_size(1600.0, 900.0);
            }
            if frame == 255 {
                let cam = render::camera(camera_center, ui_scale());
                let screen = vec2(screen_width() * 0.3, screen_height() * 0.7);
                assert!(
                    cam.world_to_screen(cam.screen_to_world(screen))
                        .distance(screen)
                        < 0.1
                );
                get_screen_data().export_png("target/smoke-resized.png");
            }
            if frame == 260 {
                break;
            }
        }
        audio.update_music(
            settings.sound
                && !paused
                && !show_settings
                && game
                    .as_ref()
                    .is_some_and(|g| g.player.health > 0.0 && !g.victory),
            settings.music_volume,
        );
        frame += 1;
        next_frame().await;
    }
}
fn smoke_game() -> Game {
    use mgame::simulation::{Enemy, Kind, Pickup, PickupKind};
    let mut g = Game::new(Level::builtin(0), false);
    g.player.pos = vec2(960.0, 696.0);
    g.player.previous = g.player.pos;
    g.player.quad = 12.0;
    g.player.immortal = 8.0;
    g.player.haste = 10.0;
    g.wave = 20;
    g.remaining = 0;
    for (i, kind) in [
        Kind::Runner,
        Kind::Gunner,
        Kind::Brute,
        Kind::Hunter,
        Kind::Bomber,
        Kind::Siege,
        Kind::Stalker,
        Kind::Lancer,
        Kind::Sentinel,
        Kind::Overlord,
    ]
    .into_iter()
    .enumerate()
    {
        let desired =
            g.player.pos + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 10.0) * 260.0;
        let pos = (0..g.level.tiles.len())
            .map(|n| g.level.center(n))
            .filter(|&p| g.level.circle_free(p, 19.0) && p.distance(g.player.pos) > 170.0)
            .min_by(|a, b| {
                a.distance_squared(desired)
                    .total_cmp(&b.distance_squared(desired))
            })
            .unwrap();
        g.enemies.push(Enemy {
            pos,
            previous: pos,
            kind,
            health: kind.base_health() * mgame::waves::health_scale(g.wave),
            max_health: kind.base_health() * mgame::waves::health_scale(g.wave),
            attack: 0.7,
            tell: 0.0,
            aim: Vec2::X,
            phase: i as f32,
        });
    }
    for (i, kind) in [
        PickupKind::Health,
        PickupKind::Quad,
        PickupKind::Immortal,
        PickupKind::Haste,
    ]
    .into_iter()
    .enumerate()
    {
        g.pickups.push(Pickup {
            pos: g.player.pos + vec2(-110.0 + i as f32 * 65.0, 90.0),
            life: 30.0,
            kind,
        });
    }
    g
}
fn wrap(text: &str, limit: usize) -> Vec<String> {
    let mut result = vec![];
    let mut line = String::new();
    for word in text.split_whitespace() {
        if line.len() + word.len() > limit && !line.is_empty() {
            result.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        result.push(line);
    }
    result
}

#[cfg(windows)]
fn app_focused() -> bool {
    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut std::ffi::c_void;
        fn GetWindowThreadProcessId(window: *mut std::ffi::c_void, process_id: *mut u32) -> u32;
    }
    let mut process_id = 0;
    // Both pointers are either a Windows-provided HWND or valid stack storage.
    unsafe {
        GetWindowThreadProcessId(GetForegroundWindow(), &mut process_id);
    }
    process_id == std::process::id()
}
#[cfg(not(windows))]
fn app_focused() -> bool {
    true
}
