//! Deterministic, embedded forest layouts. No external map files are needed to page through them.
use crate::level::Level;

pub const NAMES: [&str; 18] = [
    "01 / FOREST OUTPOST",
    "02 / PINE SWITCHBACK",
    "03 / OLD FOUNDRY",
    "04 / MIRROR PONDS",
    "05 / BRAMBLE RUN",
    "06 / LUMBER CAMP",
    "07 / CROOKED CREEK",
    "08 / WOLF HOLLOW",
    "09 / THE ORCHARD",
    "10 / MARSH CROSSING",
    "11 / TIMBER FORT",
    "12 / FERN GULLY",
    "13 / TWIN LAKES",
    "14 / HUNTER'S TRAIL",
    "15 / ROOT MAZE",
    "16 / FLOODED RUINS",
    "17 / THORN GARDEN",
    "18 / THE DEEP WOODS",
];

pub fn builtin(index: usize) -> Level {
    let index = index % NAMES.len();
    let (w, h) = (40, 28);
    let mut grid = vec![vec!['.'; w]; h];
    for (y, row) in grid.iter_mut().enumerate() {
        for (x, c) in row.iter_mut().enumerate() {
            if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                *c = 'T';
            }
        }
    }
    // Six spatial arrangements with three distinct terrain/cover distributions each.
    match index % 6 {
        0 => {
            for (x, y) in [(9, 6), (27, 6), (9, 19), (27, 19)] {
                rect(&mut grid, x, y, 4, 3, 'T');
            }
            rect(&mut grid, 18, 10, 4, 2, '~');
            rect(&mut grid, 18, 17, 4, 2, '~');
        }
        1 => {
            for (x, y) in [(9, 4), (19, 8), (29, 4)] {
                rect(&mut grid, x, y, 2, 15, 'T');
            }
            rect(&mut grid, 4, 21, 5, 3, '~');
            rect(&mut grid, 32, 21, 5, 3, '~');
        }
        2 => {
            for (x, y) in [(8, 6), (25, 6), (8, 20), (25, 20)] {
                rect(&mut grid, x, y, 7, 2, 'T');
            }
            rect(&mut grid, 18, 11, 4, 6, 'T');
            rect(&mut grid, 4, 12, 3, 3, '~');
        }
        3 => {
            rect(&mut grid, 8, 6, 8, 7, '~');
            rect(&mut grid, 24, 15, 8, 7, '~');
            for (x, y) in [(9, 20), (29, 7), (19, 5), (19, 22)] {
                rect(&mut grid, x, y, 3, 3, 'T');
            }
        }
        4 => {
            for y in [6, 12, 18] {
                for x in [8, 18, 28] {
                    rect(&mut grid, x, y, 3, 3, 'T');
                }
            }
            rect(&mut grid, 12, 22, 5, 2, '~');
            rect(&mut grid, 31, 3, 5, 2, '~');
        }
        _ => {
            for (x, y, bw, bh) in [
                (8, 6, 8, 2),
                (8, 8, 2, 4),
                (24, 6, 8, 2),
                (30, 8, 2, 4),
                (8, 20, 8, 2),
                (24, 20, 8, 2),
                (18, 11, 4, 5),
            ] {
                rect(&mut grid, x, y, bw, bh, 'T');
            }
            rect(&mut grid, 3, 19, 3, 4, '~');
        }
    }
    // Additional ponds and thickets give every arena its own approach lanes.
    let patches = [
        (14, 4),
        (23, 23),
        (4, 8),
        (33, 17),
        (14, 16),
        (24, 10),
        (5, 22),
        (33, 4),
    ];
    for (k, &(x, y)) in patches.iter().enumerate() {
        let tile = if (k + index).is_multiple_of(4) {
            '~'
        } else {
            'B'
        };
        let sx = x + index / 6;
        let sy = y - index / 6;
        for row in grid.iter_mut().skip(sy).take(2) {
            for c in row.iter_mut().skip(sx).take(2 + index % 2) {
                if *c == '.' {
                    *c = tile;
                }
            }
        }
    }
    for (k, (x, y)) in [
        (6, 13),
        (7, 14),
        (12, 10),
        (26, 17),
        (32, 12),
        (21, 22),
        (16, 7),
        (34, 22),
    ]
    .into_iter()
    .enumerate()
    {
        let x = x + (index + k) % 2;
        if grid[y][x] == '.' {
            grid[y][x] = 'C';
        }
    }
    // Keep deployment and entry clearings open. Terrain remains deterministic per arena.
    for (x, y, c) in [
        (4, 14, 'P'),
        (3, 3, 'E'),
        (36, 3, 'E'),
        (3, 24, 'E'),
        (36, 24, 'E'),
        (35, 14, 'E'),
        (14, 14, '+'),
        (26, 14, '+'),
    ] {
        for row in grid.iter_mut().take(y + 2).skip(y - 1) {
            for cell in row.iter_mut().take(x + 2).skip(x - 1) {
                *cell = '.';
            }
        }
        grid[y][x] = c;
    }
    let text = grid
        .iter()
        .map(|r| r.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    Level::parse(NAMES[index], &text).expect("embedded forest arena must be valid")
}
fn rect(grid: &mut [Vec<char>], x: usize, y: usize, w: usize, h: usize, c: char) {
    for row in grid.iter_mut().skip(y).take(h) {
        for cell in row.iter_mut().skip(x).take(w) {
            *cell = c;
        }
    }
}
