use macroquad::prelude::*;
use mgame::{
    arenas,
    level::{Level, CRATE_HALF},
    simulation::{Game, Input, Pickup, PickupKind, PLAYER_RADIUS, STEP, WEAPONS},
};

fn terrain() -> Level {
    Level::parse(
        "terrain",
        "############\n#P........E#\n#..~..C....#\n#..B.......#\n#..........#\n############",
    )
    .unwrap()
}
#[test]
fn eighteen_unique_embedded_arenas_have_every_terrain_type() {
    let mut layouts = std::collections::HashSet::new();
    for i in 0..arenas::NAMES.len() {
        let level = Level::builtin(i);
        assert!(layouts.insert(level.tiles.clone()), "duplicate arena {i}");
        for tile in ['~', 'B', 'T'] {
            assert!(level.tiles.contains(&tile), "missing {tile} in arena {i}");
        }
        assert!(!level.crates.is_empty());
        assert!(level.circle_free(level.spawn, PLAYER_RADIUS));
        assert_eq!(
            level.portals.len(),
            5,
            "all intended entries must remain reachable in arena {i}"
        );
    }
    assert_eq!(layouts.len(), 18);
}
#[test]
fn pond_blocks_movement_but_not_bullets_or_sight() {
    let l = terrain();
    let a = vec2(120.0, 120.0);
    let b = vec2(216.0, 120.0);
    assert!(!l.circle_free(vec2(168.0, 120.0), 13.0));
    assert!(l.projectile_free(vec2(168.0, 120.0), 2.0));
    assert!(!l.clear_line(a, b, 13.0));
    assert!(l.clear_shot(a, b));
    let p = l.slide(a, vec2(100.0, 0.0), 13.0);
    assert!(p.x < 144.0);
}
#[test]
fn bushes_are_walkable_and_slow_actors() {
    let l = terrain();
    let bush = vec2(168.0, 168.0);
    assert!(l.circle_free(bush, 19.0));
    assert!(l.projectile_free(bush, 2.0));
    assert!(l.terrain_speed(bush) < l.terrain_speed(l.spawn));
    let mut g = Game::new(l, false);
    g.player.pos = bush;
    for _ in 0..12 {
        g.update(
            Input {
                movement: Vec2::X,
                ..Default::default()
            },
            STEP,
        );
    }
    assert!(g.player.velocity.length() < 245.0 * 0.52);
}
#[test]
fn player_pushes_cover_without_walking_through_it_and_walls_stop_it() {
    let mut l = terrain();
    let original = l.crates[0];
    let start = original - vec2(CRATE_HALF + 13.0, 0.0);
    let moved = l.push_slide(start, vec2(80.0, 0.0), 13.0, &[]);
    assert!(l.crates[0].x > original.x + 25.0);
    assert!(l.circle_free(moved, 13.0));
    let stopped = l.push_slide(moved, vec2(1000.0, 0.0), 13.0, &[]);
    assert!(l.crates[0].x <= 11.0 * 48.0 - CRATE_HALF + 0.01);
    assert!(l.circle_free(stopped, 13.0));
    assert!(!l.clear_shot(stopped, l.crates[0] + vec2(50.0, 0.0)));
    let enemy = l.slide(stopped, vec2(100.0, 0.0), 13.0);
    assert!(enemy.x <= stopped.x + 0.02);
}
#[test]
fn crate_push_cannot_overlap_an_enemy() {
    let mut l = terrain();
    let c = l.crates[0];
    let actors = [(c + vec2(33.0, 0.0), 13.0)];
    l.push_slide(c - vec2(33.0, 0.0), vec2(60.0, 0.0), 13.0, &actors);
    assert!(l.crates[0].distance(c) < 0.05);
}
#[test]
fn moved_cover_changes_navigation_and_still_allows_a_detour() {
    let mut l = terrain();
    let from = vec2(456.0, 120.0);
    let target = vec2(216.0, 120.0);
    l.crates[0] = vec2(312.0, 120.0);
    let field = l.flow_field(target);
    let mut p = from;
    for _ in 0..2000 {
        let goal = l.waypoint(p, target, &field, 19.0);
        p = l.slide(p, (goal - p).normalize_or_zero() * 2.0, 19.0);
    }
    assert!(p.distance(target) < 3.0);
}
#[test]
fn all_temporary_powerups_activate_expire_and_quad_applies_to_all_weapons() {
    for (weapon, spec) in WEAPONS.iter().enumerate() {
        let mut g = Game::new(terrain(), false);
        g.player.weapon = weapon;
        for kind in [PickupKind::Quad, PickupKind::Immortal, PickupKind::Haste] {
            g.pickups.push(Pickup {
                pos: g.player.pos,
                life: 10.0,
                kind,
            });
        }
        g.update(Input::default(), STEP);
        assert!(g.player.quad > 11.0 && g.player.immortal > 7.0 && g.player.haste > 9.0);
        g.update(
            Input {
                fire: true,
                aim: g.player.pos + Vec2::X * 500.0,
                ..Default::default()
            },
            STEP,
        );
        let damage = if weapon == 4 {
            g.grenades[0].damage
        } else {
            g.bullets[0].damage
        };
        assert_eq!(damage, spec.damage * 4.0);
        g.player.quad = STEP * 0.5;
        g.player.immortal = STEP * 0.5;
        g.player.haste = STEP * 0.5;
        g.update(Input::default(), STEP);
        assert_eq!(
            (g.player.quad, g.player.immortal, g.player.haste),
            (0.0, 0.0, 0.0)
        );
    }
}
#[test]
fn speed_powerup_doubles_open_ground_speed_and_stops_at_walls() {
    let mut normal = Game::new(terrain(), false);
    let mut fast = Game::new(terrain(), false);
    fast.player.haste = 10.0;
    for _ in 0..45 {
        let input = Input {
            movement: Vec2::X,
            ..Default::default()
        };
        normal.update(input, STEP);
        fast.update(input, STEP);
    }
    assert!(
        (fast.player.pos.x
            - fast.level.spawn.x
            - (normal.player.pos.x - normal.level.spawn.x) * 2.0)
            .abs()
            < 0.1
    );
    for _ in 0..240 {
        fast.update(
            Input {
                movement: Vec2::X,
                dash: true,
                ..Default::default()
            },
            STEP,
        );
    }
    assert!(fast.level.circle_free(fast.player.pos, PLAYER_RADIUS));
}
