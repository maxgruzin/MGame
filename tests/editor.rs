use macroquad::prelude::*;
use mgame::{
    editor::{Document, Grid},
    level::{Level, MAX_MAP_SIZE, TILE},
};

#[test]
fn legacy_tiles_and_crates_round_trip() {
    for file in std::fs::read_dir("Maps").unwrap() {
        let source = std::fs::read_to_string(file.unwrap().path()).unwrap();
        let grid = Grid::parse(&source).unwrap();
        assert_eq!(Grid::parse(&grid.encode()).unwrap(), grid);
        let level = Level::parse("round trip", &grid.encode()).unwrap();
        assert_eq!(Grid::from_level(&level), grid);
    }
    let mut grid = Grid::new(12, 12).unwrap();
    grid.tiles[60] = b'C';
    assert_eq!(
        Grid::from_level(&Level::parse("crates", &grid.encode()).unwrap()),
        grid
    );
}

#[test]
fn strokes_are_contiguous_and_undo_is_one_gesture() {
    let grid = Grid::new(64, 32).unwrap();
    let mut doc = Document::new(grid.clone(), true);
    doc.stroke((4, 8), (54, 8), 3, b'~');
    doc.stroke((54, 8), (4, 8), 3, b'B');
    doc.commit();
    for x in 4..55 {
        assert_eq!(doc.grid.tiles[8 * 64 + x], b'B');
    }
    assert!(doc.dirty());
    doc.undo();
    assert_eq!(doc.grid, grid);
    assert!(!doc.dirty());
    doc.redo();
    assert!(doc.dirty());
    assert_eq!(doc.grid.tiles[8 * 64 + 30], b'B');
    doc.undo();
    doc.brush(10, 10, 1, b'C');
    doc.commit();
    assert!(!doc.can_redo());
}

#[test]
fn fill_respects_regions_and_player_placement_is_unique() {
    let mut doc = Document::new(Grid::new(20, 20).unwrap(), false);
    doc.rectangle((10, 0), (10, 19), b'#');
    doc.commit();
    doc.fill(4, 4, b'~');
    doc.commit();
    assert_eq!(doc.grid.tiles[4 * 20 + 4], b'~');
    assert_eq!(doc.grid.tiles[4 * 20 + 14], b'.');
    doc.player(15, 15);
    doc.commit();
    assert_eq!(doc.grid.tiles.iter().filter(|&&c| c == b'P').count(), 1);
    assert_eq!(doc.grid.tiles[15 * 20 + 15], b'P');
    doc.undo();
    assert_eq!(doc.grid.tiles[2 * 20 + 2], b'P');
}

#[test]
fn maximum_size_fill_resize_and_history() {
    let mut doc = Document::new(Grid::new(MAX_MAP_SIZE, MAX_MAP_SIZE).unwrap(), false);
    doc.fill(5, 5, b'B');
    doc.commit();
    assert_eq!(doc.grid.tiles[900 * 1024 + 900], b'B');
    doc.resize(16, 32).unwrap();
    assert_eq!((doc.grid.width, doc.grid.height), (16, 32));
    doc.undo();
    assert_eq!(doc.grid.width, 1024);
    assert_eq!(doc.grid.tiles[900 * 1024 + 900], b'B');
    doc.undo();
    assert_eq!(doc.grid.tiles[900 * 1024 + 900], b'.');
    doc.redo();
    doc.redo();
    assert_eq!((doc.grid.width, doc.grid.height), (16, 32));
    assert!(Grid::new(1025, 5).is_err());
    assert!(Grid::new(5, 4).is_err());
}

#[test]
fn large_map_is_playable_and_serializes() {
    let grid = Grid::new(1024, 1024).unwrap();
    let doc = Document::new(grid.clone(), false);
    let level = doc.validate("Large arena").unwrap();
    assert_eq!(level.width * level.height, 1_048_576);
    assert_eq!(level.portals.len(), 1);
    assert!(level.reachable[level.index(level.portals[0])]);
    assert_eq!(Grid::parse(&grid.encode()).unwrap(), grid);
}

#[test]
fn invalid_gameplay_can_be_edited_but_not_tested() {
    let mut doc = Document::new(Grid::new(12, 12).unwrap(), false);
    doc.rectangle((0, 5), (11, 5), b'#');
    doc.commit();
    assert!(doc
        .validate("disconnected")
        .unwrap_err()
        .contains("cannot be reached"));
    assert!(Grid::parse(&doc.grid.encode()).is_ok());
    doc.rectangle((0, 0), (11, 11), b'#');
    doc.commit();
    assert!(doc
        .validate("no player")
        .unwrap_err()
        .contains("exactly one"));
    assert!(Grid::parse(&doc.grid.encode()).is_ok());
}

#[test]
fn saves_overwrite_and_track_saved_revision() {
    let directory = std::env::temp_dir().join(format!("mgame-editor-{}", std::process::id()));
    let path = directory.join("Map with spaces.arena");
    let mut doc = Document::new(Grid::new(16, 16).unwrap(), false);
    doc.save(&path).unwrap();
    assert!(!doc.dirty());
    doc.brush(5, 5, 1, b'B');
    doc.commit();
    doc.save(&path).unwrap();
    assert!(!doc.dirty());
    assert_eq!(
        Grid::parse(&std::fs::read_to_string(&path).unwrap()).unwrap(),
        doc.grid
    );
    doc.undo();
    assert!(doc.dirty());
    doc.redo();
    assert!(!doc.dirty());
    let before = std::fs::read(&path).unwrap();
    let temp = directory.join(format!(".Map with spaces.arena.{}.tmp", std::process::id()));
    std::fs::write(&temp, "occupied").unwrap();
    doc.brush(6, 6, 1, b'C');
    doc.commit();
    assert!(doc.save(&path).is_err());
    assert!(doc.dirty());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::read_to_string(&temp).unwrap(), "occupied");
    std::fs::remove_file(temp).unwrap();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn crate_rasterized_navigation_matches_collision_checks() {
    let mut level = Level::parse("crates", &Grid::new(32, 32).unwrap().encode()).unwrap();
    for y in (4..28).step_by(3) {
        for x in (4..28).step_by(3) {
            level
                .crates
                .push(vec2(x as f32 * TILE + 12.0, y as f32 * TILE + 35.0));
        }
    }
    let field = level.flow_field(level.spawn);
    // Reference BFS uses the original continuous circle collision for each edge.
    let mut reference = vec![u32::MAX; level.tiles.len()];
    let root = level.index(level.spawn);
    reference[root] = 0;
    let mut queue = std::collections::VecDeque::from([root]);
    while let Some(i) = queue.pop_front() {
        let (x, y) = ((i % level.width) as i32, (i / level.width) as i32);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if level.solid(nx, ny) {
                continue;
            }
            let n = ny as usize * level.width + nx as usize;
            if reference[n] == u32::MAX
                && level.circle_free(level.center(n), 19.0)
                && level.circle_free(level.center(i).lerp(level.center(n), 0.5), 19.0)
            {
                reference[n] = reference[i] + 1;
                queue.push_back(n);
            }
        }
    }
    assert_eq!(field, reference);
}
