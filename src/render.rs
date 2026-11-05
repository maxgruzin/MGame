use crate::assets::Assets;
use macroquad::prelude::*;
use mgame::{
    level::{Level, TILE},
    simulation::{Game, Kind, PickupKind, WEAPONS},
};

pub const BG: Color = Color::new(0.035, 0.065, 0.035, 1.0);
pub const PANEL: Color = Color::new(0.035, 0.065, 0.045, 0.96);
pub const MINT: Color = Color::new(0.38, 0.92, 0.81, 1.0);
pub const MUTED: Color = Color::new(0.48, 0.60, 0.63, 1.0);
pub fn text(s: &str, x: f32, y: f32, size: f32, color: Color) {
    let (font_size, font_scale, font_scale_aspect) = macroquad::text::camera_font_scale(size);
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font_size,
            font_scale,
            font_scale_aspect,
            color,
            ..Default::default()
        },
    );
}
pub fn line(a: Vec2, b: Vec2, width: f32, color: Color) {
    draw_line(a.x, a.y, b.x, b.y, width, color);
}
pub fn circle(p: Vec2, r: f32, c: Color) {
    draw_circle(p.x, p.y, r, c);
}
fn alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}
pub fn camera(center: Vec2, scale: f32) -> Camera2D {
    viewport_camera(center, vec2(screen_width(), screen_height()) / scale)
}
pub fn viewport_camera(center: Vec2, size: Vec2) -> Camera2D {
    Camera2D {
        target: center,
        // Macroquad 0.4.16 negates screen-camera Y in Camera::matrix.
        // Positive zoom gives top-left origin, matching WASD and mouse coordinates.
        zoom: vec2(2.0 / size.x, 2.0 / size.y),
        ..Default::default()
    }
}
pub fn world(game: &Game, assets: &Assets, cam: &Camera2D, interpolation: f32, effects: bool) {
    set_camera(cam);
    let l = &game.level;
    let extent = vec2(1.0 / cam.zoom.x.abs(), 1.0 / cam.zoom.y.abs());
    let min = (cam.target - extent) / TILE - Vec2::splat(2.0);
    let max = (cam.target + extent) / TILE + Vec2::splat(2.0);
    for y in (min.y as i32).max(0)..=(max.y as i32).min(l.height as i32 - 1) {
        for x in (min.x as i32).max(0)..=(max.x as i32).min(l.width as i32 - 1) {
            let tile = l.tiles[y as usize * l.width + x as usize];
            let p = vec2(x as f32, y as f32) * TILE;
            let variant = ((x * 7 + y * 11).unsigned_abs() as usize) % 24;
            sprite(
                &assets.grass,
                Rect::new(
                    (variant % 8) as f32 * 50.0,
                    (variant / 8) as f32 * 50.0,
                    50.0,
                    50.0,
                ),
                p,
                vec2(TILE, TILE),
                Color::new(0.80, 0.91, 0.72, 1.0),
            );
            if tile == '~' {
                draw_rectangle(p.x, p.y, TILE, TILE, Color::from_hex(0x285d61));
                draw_rectangle(
                    p.x + 3.0,
                    p.y + 3.0,
                    TILE - 6.0,
                    TILE - 6.0,
                    Color::from_hex(0x326e71),
                );
                for k in 0..3 {
                    let yy = p.y + 9.0 + k as f32 * 13.0;
                    let xx = p.x + 8.0 + ((game.time * 0.7 + x as f32 + k as f32).sin() * 4.0);
                    draw_line(
                        xx,
                        yy,
                        xx + 19.0,
                        yy,
                        1.2,
                        Color::new(0.40, 0.68, 0.66, 0.45),
                    );
                }
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx >= 0
                        && ny >= 0
                        && nx < l.width as i32
                        && ny < l.height as i32
                        && l.tiles[ny as usize * l.width + nx as usize] != '~'
                    {
                        let edge = p + vec2(
                            if dx > 0 { TILE - 3.0 } else { 0.0 },
                            if dy > 0 { TILE - 3.0 } else { 0.0 },
                        );
                        draw_rectangle(
                            edge.x,
                            edge.y,
                            if dx == 0 { TILE } else { 3.0 },
                            if dy == 0 { TILE } else { 3.0 },
                            Color::from_hex(0x788460),
                        );
                    }
                }
            } else if Level::solid_tile(tile) {
                circle(p + vec2(28.0, 32.0), 27.0, alpha(BLACK, 0.28));
                let source = if (x + y) % 2 == 0 { 100.0 } else { 150.0 };
                sprite(
                    &assets.objects,
                    Rect::new(source, 0.0, 50.0, 50.0),
                    p - vec2(5.0, 7.0),
                    vec2(58.0, 58.0),
                    WHITE,
                );
            } else if tile == 'B' {
                circle(
                    p + Vec2::splat(24.0),
                    23.0,
                    alpha(Color::from_hex(0x1b3b20), 0.6),
                );
                sprite(
                    &assets.objects,
                    Rect::new(150.0, 0.0, 50.0, 50.0),
                    p + vec2(1.0, 2.0),
                    vec2(46.0, 42.0),
                    Color::new(0.65, 0.9, 0.52, 0.88),
                );
                for k in 0..4 {
                    circle(
                        p + vec2(10.0 + k as f32 * 8.0, 17.0 + (k % 2) as f32 * 10.0),
                        2.0,
                        Color::from_hex(0xaaa34f),
                    );
                }
            }
        }
    }
    for &p in &l.crates {
        wooden_crate(p);
    }
    for &p in &l.portals {
        circle(p, 20.0, Color::from_hex(0x302633));
        draw_poly_lines(p.x, p.y, 6, 17.0, 30.0, 1.5, Color::from_hex(0x815569));
        circle(p, 3.0, Color::from_hex(0xc46682));
    }
    for &p in &l.medical {
        circle(p, 30.0, alpha(MINT, 0.12));
        sprite(
            &assets.objects,
            Rect::new(50.0, 0.0, 50.0, 50.0),
            p - Vec2::splat(22.0),
            Vec2::splat(44.0),
            WHITE,
        );
        draw_circle_lines(p.x, p.y, 32.0, 1.0, alpha(MINT, 0.35));
    }
    for s in &game.spawns {
        let r = 22.0 + (s.timer * 12.0).sin() * 3.0;
        draw_circle_lines(s.pos.x, s.pos.y, r, 2.0, s.kind.color());
        line(
            s.pos - vec2(7.0, 0.0),
            s.pos + vec2(7.0, 0.0),
            2.0,
            s.kind.color(),
        );
        line(
            s.pos - vec2(0.0, 7.0),
            s.pos + vec2(0.0, 7.0),
            2.0,
            s.kind.color(),
        );
    }
    for pickup in &game.pickups {
        let pos = pickup.pos + vec2(0.0, (game.time * 4.0).sin() * 2.0);
        let c = pickup.kind.color();
        circle(pos, 27.0, alpha(c, 0.15));
        if pickup.kind == PickupKind::Health {
            sprite(
                &assets.objects,
                Rect::new(50.0, 0.0, 50.0, 50.0),
                pos - Vec2::splat(19.0),
                Vec2::splat(38.0),
                WHITE,
            );
        } else {
            draw_rectangle(
                pos.x - 17.0,
                pos.y - 17.0,
                34.0,
                34.0,
                Color::from_hex(0x23352c),
            );
            draw_rectangle_lines(pos.x - 17.0, pos.y - 17.0, 34.0, 34.0, 2.0, c);
            text(
                match pickup.kind {
                    PickupKind::Quad => "4X",
                    PickupKind::Immortal => "I",
                    _ => "2X",
                },
                pos.x - 11.0,
                pos.y + 7.0,
                21.0,
                c,
            );
        }
    }
    for e in &game.enemies {
        let p = e.previous.lerp(e.pos, interpolation);
        let c = e.kind.color();
        let r = e.kind.radius();
        circle(p + vec2(3.0, 6.0), r + 1.0, alpha(BLACK, 0.4));
        if e.tell > 0.0 {
            line(p, p + e.aim * 360.0, 1.0, alpha(c, 0.4));
            draw_circle_lines(
                p.x,
                p.y,
                r + 5.0,
                2.0,
                alpha(c, 0.5 + 0.5 * (game.time * 30.0).sin()),
            );
        }
        let size = match e.kind {
            Kind::Brute => 76.0,
            Kind::Siege => 84.0,
            Kind::Bomber => 66.0,
            _ => 58.0,
        };
        let frame = if e.previous.distance_squared(e.pos) > 0.001 {
            1 + (game.time * 11.0 + e.phase) as usize % 6
        } else {
            0
        };
        let sheet = if matches!(e.kind, Kind::Brute | Kind::Siege | Kind::Hunter) {
            &assets.brown
        } else {
            &assets.green
        };
        draw_texture_ex(
            sheet,
            p.x - size * 0.5,
            p.y - size * 0.5,
            match e.kind {
                Kind::Hunter => Color::new(0.85, 0.95, 1.0, 1.0),
                Kind::Siege => Color::new(1.0, 0.7, 0.6, 1.0),
                _ => WHITE,
            },
            DrawTextureParams {
                dest_size: Some(Vec2::splat(size)),
                source: Some(Rect::new(100.0, frame as f32 * 50.0, 50.0, 50.0)),
                rotation: e.aim.to_angle() + std::f32::consts::FRAC_PI_2,
                ..Default::default()
            },
        );
        let side = vec2(-e.aim.y, e.aim.x);
        // Organic weapon organs make the six enemy types recognizable at a glance.
        match e.kind {
            Kind::Bomber => {
                circle(p - e.aim * 9.0, 10.0, Color::from_hex(0x789b3b));
                circle(p - e.aim * 11.0, 5.0, Color::from_hex(0xd9e178));
            }
            Kind::Siege => {
                for sign in [-1.0, 1.0] {
                    line(
                        p + side * 12.0 * sign,
                        p + side * 12.0 * sign + e.aim * 22.0,
                        7.0,
                        Color::from_hex(0x543c2c),
                    );
                    circle(p + side * 12.0 * sign + e.aim * 22.0, 3.0, c);
                }
            }
            Kind::Hunter => {
                line(
                    p + side * 9.0,
                    p + side * 9.0 + e.aim * 28.0,
                    4.0,
                    Color::from_hex(0xc4c5a0),
                );
            }
            Kind::Gunner => {
                circle(p + e.aim * 12.0, 5.0, Color::from_hex(0xc6dc64));
            }
            _ => {}
        }
        for sign in [-1.0, 1.0] {
            circle(p + e.aim * 10.0 + side * 4.0 * sign, 1.7, c);
        }
        if e.health < e.max_health {
            draw_rectangle(
                p.x - 16.0,
                p.y - r - 10.0,
                32.0,
                3.0,
                Color::from_hex(0x17252e),
            );
            draw_rectangle(
                p.x - 16.0,
                p.y - r - 10.0,
                32.0 * (e.health / e.max_health).max(0.0),
                3.0,
                c,
            );
        }
    }
    let player = &game.player;
    let p = player.previous.lerp(player.pos, interpolation);
    let aim = player.aim;
    let side = vec2(-aim.y, aim.x);
    let bob = (game.time * 20.0).sin() * (player.velocity.length() / 245.0) * 3.0;
    circle(p + vec2(3.0, 7.0), 16.0, alpha(BLACK, 0.45));
    if player.dash_time > 0.0 {
        circle(p, 24.0, alpha(MINT, 0.15));
    }
    circle(p + side * 7.0 - aim * bob, 5.0, Color::from_hex(0x22343d));
    circle(p - side * 7.0 + aim * bob, 5.0, Color::from_hex(0x22343d));
    circle(
        p,
        13.0,
        if player.invulnerable > 0.0 && (game.time * 25.0).sin() > 0.0 {
            WHITE
        } else {
            MINT
        },
    );
    circle(p - aim * 2.0, 9.0, Color::from_hex(0x243c46));
    line(
        p + side * 6.0 + aim * 5.0,
        p - side * 6.0 + aim * 5.0,
        4.0,
        Color::from_hex(0xb9fff0),
    );
    line(
        p + side * 7.0,
        p + side * 7.0 + aim * 24.0,
        7.0,
        Color::from_hex(0x718b92),
    );
    line(
        p + side * 7.0 + aim * 15.0,
        p + side * 7.0 + aim * 27.0,
        3.0,
        Color::from_hex(0xdae8df),
    );
    if player.immortal > 0.0 {
        draw_circle_lines(p.x, p.y, 22.0, 2.5, PickupKind::Immortal.color());
    }
    if player.quad > 0.0 {
        draw_circle_lines(p.x, p.y, 19.0, 1.5, PickupKind::Quad.color());
    }
    if player.haste > 0.0 {
        line(
            p - aim * 22.0,
            p - aim * 38.0,
            3.0,
            PickupKind::Haste.color(),
        );
    }
    for g in &game.grenades {
        let c = if g.hostile {
            Color::from_hex(0xff7262)
        } else {
            Color::from_hex(0xffd671)
        };
        let pulse = 0.5 + 0.5 * (game.time * 16.0).sin();
        draw_circle_lines(
            g.target.x,
            g.target.y,
            145.0,
            1.5,
            alpha(c, 0.18 + pulse * 0.17),
        );
        draw_circle_lines(g.target.x, g.target.y, 16.0, 2.0, c);
        let flight = (g.age / 0.6).min(1.0);
        let height = (flight * std::f32::consts::PI).sin() * 48.0;
        circle(g.pos + vec2(2.0, 4.0), 6.0, alpha(BLACK, 0.35));
        circle(g.pos - vec2(0.0, height), 7.0, Color::from_hex(0x384631));
        circle(g.pos - vec2(0.0, height), 3.0, c);
        text(
            &format!("{:.1}", (g.fuse - g.age).max(0.0)),
            g.target.x + 20.0,
            g.target.y + 5.0,
            16.0,
            c,
        );
    }
    for blast in &game.blasts {
        let t = 1.0 - blast.life / 0.45;
        let c = if blast.hostile {
            Color::from_hex(0xff6e45)
        } else {
            Color::from_hex(0xffc761)
        };
        circle(blast.pos, blast.radius * t, alpha(c, (1.0 - t) * 0.20));
        draw_circle_lines(
            blast.pos.x,
            blast.pos.y,
            blast.radius * t,
            4.0 * (1.0 - t) + 1.0,
            alpha(c, 1.0 - t),
        );
    }
    for b in &game.bullets {
        let c = if b.blast_radius > 0.0 {
            Color::from_hex(0xffa64f)
        } else if b.hostile {
            Color::from_hex(0xff8c70)
        } else if b.rail {
            Color::from_hex(0xa6afff)
        } else {
            Color::from_hex(0xffe4a0)
        };
        let tail = b.pos
            - b.velocity.normalize_or_zero()
                * if b.rail {
                    65.0
                } else if b.hostile {
                    12.0
                } else {
                    22.0
                };
        if effects {
            line(tail, b.pos, 7.0, alpha(c, 0.12));
            circle(b.pos, 7.0, alpha(c, 0.10));
        }
        line(tail, b.pos, if b.hostile { 3.5 } else { 2.0 }, c);
        circle(b.pos, if b.blast_radius > 0.0 { 5.0 } else { 2.0 }, WHITE);
    }
    if effects {
        for particle in &game.particles {
            circle(
                particle.pos,
                particle.size * particle.life / particle.max_life,
                alpha(particle.color, particle.life / particle.max_life),
            );
        }
    }
    set_default_camera();
}
pub fn minimap(level: &Level, player: Option<Vec2>, origin: Vec2, size: Vec2, enemies: &[Vec2]) {
    let scale = (size.x / level.width as f32).min(size.y / level.height as f32);
    draw_rectangle(
        origin.x - 6.0,
        origin.y - 6.0,
        level.width as f32 * scale + 12.0,
        level.height as f32 * scale + 12.0,
        PANEL,
    );
    for (i, &tile) in level.tiles.iter().enumerate() {
        let p = origin + vec2((i % level.width) as f32, (i / level.width) as f32) * scale;
        let color = if tile == '~' {
            Color::from_hex(0x488e9b)
        } else if tile == 'B' {
            Color::from_hex(0x77a44c)
        } else if Level::solid_tile(tile) {
            Color::from_hex(0x315d35)
        } else {
            Color::from_hex(0x47673c)
        };
        draw_rectangle(p.x, p.y, scale - 0.25, scale - 0.25, color);
    }
    for p in &level.crates {
        let q = origin + *p / TILE * scale;
        draw_rectangle(
            q.x - scale * 0.35,
            q.y - scale * 0.35,
            scale * 0.7,
            scale * 0.7,
            Color::from_hex(0xc79a64),
        );
    }
    for p in enemies {
        circle(origin + *p / TILE * scale, 2.0, Color::from_hex(0xff6d80));
    }
    if let Some(p) = player {
        circle(origin + p / TILE * scale, 3.0, MINT);
    }
}
pub fn hud(g: &Game, scale: f32) {
    let w = screen_width() / scale;
    let h = screen_height() / scale;
    set_camera(&viewport_camera(vec2(w, h) * 0.5, vec2(w, h)));
    draw_rectangle(20.0, 20.0, 282.0, 94.0, PANEL);
    text("MG / OVERDRIVE", 36.0, 46.0, 21.0, MINT);
    text(
        &format!("HULL  {:03}", g.player.health.ceil() as u32),
        36.0,
        72.0,
        18.0,
        WHITE,
    );
    draw_rectangle(36.0, 86.0, 248.0, 5.0, Color::from_hex(0x263943));
    draw_rectangle(36.0, 86.0, 248.0 * g.player.health / 100.0, 5.0, MINT);
    text(
        &format!(
            "WAVE {:02} {}",
            g.wave,
            if g.endless { "/ ENDLESS" } else { "/ 06" }
        ),
        w * 0.5 - 85.0,
        43.0,
        24.0,
        WHITE,
    );
    text(
        &format!(
            "{} HOSTILES",
            g.enemies.len() + g.spawns.len() + g.remaining as usize
        ),
        w * 0.5 - 65.0,
        66.0,
        16.0,
        MUTED,
    );
    text(&format!("{:07}", g.score), w - 207.0, 44.0, 28.0, WHITE);
    text(&format!("{} KILLS", g.kills), w - 207.0, 65.0, 16.0, MUTED);
    minimap(
        &g.level,
        Some(g.player.pos),
        vec2(w - 201.0, 87.0),
        vec2(175.0, 126.0),
        &g.enemies.iter().map(|e| e.pos).collect::<Vec<_>>(),
    );
    draw_rectangle(20.0, h - 105.0, 330.0, 85.0, PANEL);
    let weapon = WEAPONS[g.player.weapon];
    text(weapon.name, 36.0, h - 77.0, 20.0, MINT);
    text(
        &format!(
            "{:02} / {:02}",
            g.player.ammo[g.player.weapon], weapon.magazine
        ),
        36.0,
        h - 43.0,
        30.0,
        WHITE,
    );
    text(
        if g.player.reload > 0.0 {
            "RELOADING"
        } else {
            "R / RELOAD"
        },
        189.0,
        h - 45.0,
        16.0,
        MUTED,
    );
    if g.player.reload > 0.0 {
        draw_rectangle(
            36.0,
            h - 30.0,
            298.0 * (1.0 - g.player.reload / weapon.reload),
            3.0,
            MINT,
        );
    }
    text("SPACE / DASH", w * 0.5 - 60.0, h - 44.0, 17.0, MUTED);
    draw_rectangle(
        w * 0.5 - 60.0,
        h - 32.0,
        120.0,
        4.0,
        Color::from_hex(0x263943),
    );
    draw_rectangle(
        w * 0.5 - 60.0,
        h - 32.0,
        120.0 * (1.0 - g.player.dash_cooldown / 1.4),
        4.0,
        MINT,
    );
    text(
        "1 2 3 4 5 / WEAPONS    ESC / PAUSE",
        w - 340.0,
        h - 30.0,
        16.0,
        MUTED,
    );
    for (i, (label, seconds, c)) in [
        ("QUAD DAMAGE", g.player.quad, PickupKind::Quad.color()),
        ("IMMORTAL", g.player.immortal, PickupKind::Immortal.color()),
        ("2X SPEED", g.player.haste, PickupKind::Haste.color()),
    ]
    .iter()
    .enumerate()
    {
        if *seconds > 0.0 {
            text(
                &format!("{label}  {:.1}s", seconds),
                36.0,
                180.0 + i as f32 * 24.0,
                19.0,
                *c,
            );
        }
    }
    if g.player.weapon >= 3 {
        text(
            "BLASTS HURT YOU TOO / KEEP CLEAR",
            36.0,
            h - 119.0,
            16.0,
            Color::from_hex(0xffd582),
        );
    }
    if g.combo >= 3 {
        text(
            &format!("{} CHAIN  /  x{}", g.combo, 1 + g.combo.min(20) / 5),
            36.0,
            144.0,
            20.0,
            Color::from_hex(0xffc66d),
        );
    }
    if g.remaining == 0 && g.enemies.is_empty() && g.spawns.is_empty() && !g.victory {
        text(
            &format!("NEXT WAVE IN {}", g.intermission.ceil() as u32),
            w * 0.5 - 110.0,
            h * 0.35,
            26.0,
            MINT,
        );
    }
    if g.hit_flash > 0.0 {
        draw_rectangle(
            0.0,
            0.0,
            w,
            h,
            Color::new(0.8, 0.08, 0.12, g.hit_flash * 0.7),
        );
    }
    set_default_camera();
}
pub fn crosshair() {
    let (x, y) = mouse_position();
    draw_circle_lines(x, y, 6.0, 1.3, MINT);
    for offset in [-1.0, 1.0] {
        draw_line(x + offset * 10.0, y, x + offset * 15.0, y, 1.5, WHITE);
        draw_line(x, y + offset * 10.0, x, y + offset * 15.0, 1.5, WHITE);
    }
}

fn sprite(texture: &Texture2D, source: Rect, pos: Vec2, size: Vec2, color: Color) {
    draw_texture_ex(
        texture,
        pos.x,
        pos.y,
        color,
        DrawTextureParams {
            dest_size: Some(size),
            source: Some(source),
            ..Default::default()
        },
    );
}
fn wooden_crate(p: Vec2) {
    draw_rectangle(p.x - 18.0, p.y - 16.0, 42.0, 42.0, alpha(BLACK, 0.30));
    draw_rectangle(
        p.x - 20.0,
        p.y - 20.0,
        40.0,
        40.0,
        Color::from_hex(0x593c24),
    );
    draw_rectangle(
        p.x - 17.0,
        p.y - 17.0,
        34.0,
        34.0,
        Color::from_hex(0xa27843),
    );
    for y in [-10.0, 0.0, 10.0] {
        line(
            p + vec2(-17.0, y),
            p + vec2(17.0, y),
            1.5,
            Color::from_hex(0x6a4c2c),
        );
    }
    line(
        p + vec2(-15.0, -15.0),
        p + vec2(15.0, 15.0),
        4.0,
        Color::from_hex(0xc39a5b),
    );
    line(
        p + vec2(-15.0, 15.0),
        p + vec2(15.0, -15.0),
        4.0,
        Color::from_hex(0xc39a5b),
    );
    for x in [-16.0, 16.0] {
        for y in [-16.0, 16.0] {
            circle(p + vec2(x, y), 1.7, Color::from_hex(0x34382d));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::camera::Camera;

    #[test]
    fn camera_matches_screen_and_mouse_y_axis() {
        let camera = viewport_camera(vec2(640.0, 360.0), vec2(1280.0, 720.0));
        let matrix = camera.matrix();
        assert!(
            matrix
                .transform_point3(vec3(0.0, 0.0, 0.0))
                .distance(vec3(-1.0, 1.0, 0.0))
                < 0.001
        );
        assert!(
            matrix
                .transform_point3(vec3(1280.0, 720.0, 0.0))
                .distance(vec3(1.0, -1.0, 0.0))
                < 0.001
        );
    }
}
