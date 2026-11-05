use macroquad::prelude::*;
use mgame::level::Level;

#[test]
fn every_shipped_map_loads_and_has_reachable_spawns() {
    for file in std::fs::read_dir("Maps").unwrap() {
        let path = file.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("mmp") {
            continue;
        }
        let level = Level::load(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(level.circle_free(level.spawn, 13.0));
        let field = level.flow_field(level.spawn);
        for p in &level.portals {
            assert_ne!(field[level.index(*p)], u32::MAX);
        }
    }
}

#[test]
fn enemies_reach_player_from_every_builtin_portal() {
    for i in 0..mgame::arenas::NAMES.len() {
        let level = Level::builtin(i);
        let field = level.flow_field(level.spawn);
        for &spawn in &level.portals {
            for radius in [13.0, 19.0] {
                let mut p = spawn;
                for _ in 0..5000 {
                    let goal = level.waypoint(p, level.spawn, &field, radius);
                    p = level.slide(p, (goal - p).normalize_or_zero() * 2.0, radius);
                    if p.distance(level.spawn) < 3.0 {
                        break;
                    }
                }
                assert!(
                    p.distance(level.spawn) < 3.0,
                    "{} radius {radius}: stuck at {p:?} from {spawn:?}",
                    level.name
                );
            }
        }
    }
}

#[test]
fn unreachable_portals_do_not_block_wave_completion() {
    let level = Level::parse(
        "islands",
        "############\n#P.....#..E#\n#......#...#\n#......#...#\n#.....E#...#\n############",
    )
    .unwrap();
    assert_eq!(level.portals.len(), 1);
    assert!(level.portals[0].x < 7.0 * 48.0);
}

#[test]
fn wall_collision_is_stable_across_frame_partitions() {
    let level = Level::builtin(0);
    let start = vec2(360.0, 300.0);
    let total = vec2(200.0, 80.0);
    let one = level.slide(start, total, 13.0);
    let mut many = start;
    for _ in 0..120 {
        many = level.slide(many, total / 120.0, 13.0);
    }
    assert!(one.distance(many) < 0.2, "{one:?} vs {many:?}");
}

#[test]
fn legacy_maze_spawns_have_working_enemy_routes() {
    for file in std::fs::read_dir("Maps").unwrap() {
        let path = file.unwrap().path();
        let level = Level::load(&path).unwrap();
        let field = level.flow_field(level.spawn);
        for &spawn in &level.portals {
            for radius in [13.0, 19.0] {
                let mut p = spawn;
                let limit = (field[level.index(spawn)] as usize + 2) * 60;
                for _ in 0..limit {
                    let goal = level.waypoint(p, level.spawn, &field, radius);
                    p = level.slide(p, (goal - p).normalize_or_zero() * 2.0, radius);
                    if p.distance(level.spawn) < 3.0 {
                        break;
                    }
                }
                assert!(
                    p.distance(level.spawn) < 3.0,
                    "{} radius {radius}: stuck at {p:?}",
                    level.name
                );
            }
        }
    }
}
