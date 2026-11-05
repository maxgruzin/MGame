//! Window-independent editing model. Only changed cells are retained for brush history.
use crate::level::{parse_grid, Level, MAX_MAP_SIZE};
use std::{collections::VecDeque, io::Write, path::Path};

const HISTORY_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<u8>,
}
impl Grid {
    pub fn new(width: usize, height: usize) -> Result<Self, String> {
        if !(5..=MAX_MAP_SIZE).contains(&width) || !(5..=MAX_MAP_SIZE).contains(&height) {
            return Err(format!("Width and height must be 5 to {MAX_MAP_SIZE}."));
        }
        let mut grid = Self {
            width,
            height,
            tiles: vec![b'.'; width * height],
        };
        for y in 0..height {
            for x in 0..width {
                if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                    grid.tiles[y * width + x] = b'#';
                }
            }
        }
        grid.tiles[2 * width + 2] = b'P';
        grid.tiles[(height - 2) * width + width - 2] = b'E';
        Ok(grid)
    }
    pub fn parse(text: &str) -> Result<Self, String> {
        let (width, height, tiles) = parse_grid(text)?;
        Ok(Self {
            width,
            height,
            tiles,
        })
    }
    pub fn from_level(level: &Level) -> Self {
        let mut grid = Self {
            width: level.width,
            height: level.height,
            tiles: level.tiles.iter().map(|&c| c as u8).collect(),
        };
        for &p in &level.crates {
            grid.tiles[level.index(p)] = b'C';
        }
        grid
    }
    pub fn encode(&self) -> String {
        let mut bytes = Vec::with_capacity(self.tiles.len() + self.height);
        for row in self.tiles.chunks(self.width) {
            bytes.extend_from_slice(row);
            bytes.push(b'\n');
        }
        String::from_utf8(bytes).expect("map tiles are ASCII")
    }
}

#[derive(Debug)]
struct Change {
    index: usize,
    before: u8,
    after: u8,
}
#[derive(Debug)]
enum Edit {
    Cells(Vec<Change>),
    Resize { before: Grid, after: Grid },
}
impl Edit {
    fn bytes(&self) -> usize {
        match self {
            Self::Cells(c) => c.len() * std::mem::size_of::<Change>(),
            Self::Resize { before, after } => before.tiles.len() + after.tiles.len(),
        }
    }
    fn apply(&self, grid: &mut Grid, forward: bool) {
        match self {
            Self::Cells(c) => {
                for c in c {
                    grid.tiles[c.index] = if forward { c.after } else { c.before };
                }
            }
            Self::Resize { before, after } => {
                *grid = if forward {
                    after.clone()
                } else {
                    before.clone()
                }
            }
        }
    }
}
#[derive(Debug)]
struct Revision {
    edit: Edit,
    before: u64,
    after: u64,
}

pub struct Document {
    pub grid: Grid,
    undo: VecDeque<Revision>,
    redo: Vec<Revision>,
    // Dense lookup avoids a million-entry hash table during a large flood fill.
    original: Vec<Option<u8>>,
    touched: Vec<usize>,
    revision: u64,
    next_revision: u64,
    saved_revision: Option<u64>,
    pub generation: u64,
}
impl Document {
    pub fn new(grid: Grid, saved: bool) -> Self {
        Self {
            original: vec![None; grid.tiles.len()],
            grid,
            undo: VecDeque::new(),
            redo: vec![],
            touched: vec![],
            revision: 0,
            next_revision: 1,
            saved_revision: saved.then_some(0),
            generation: 0,
        }
    }
    pub fn dirty(&self) -> bool {
        self.saved_revision != Some(self.revision) || !self.touched.is_empty()
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    fn put(&mut self, x: i32, y: i32, tile: u8) {
        if x < 0 || y < 0 || x >= self.grid.width as i32 || y >= self.grid.height as i32 {
            return;
        }
        let i = y as usize * self.grid.width + x as usize;
        if self.grid.tiles[i] == tile {
            return;
        }
        if self.original[i].is_none() {
            self.original[i] = Some(self.grid.tiles[i]);
            self.touched.push(i);
        }
        self.grid.tiles[i] = tile;
        self.generation += 1;
    }
    pub fn player(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= self.grid.width as i32 || y >= self.grid.height as i32 {
            return;
        }
        for i in 0..self.grid.tiles.len() {
            if matches!(self.grid.tiles[i], b'P' | b'1') {
                self.put(
                    (i % self.grid.width) as i32,
                    (i / self.grid.width) as i32,
                    b'.',
                );
            }
        }
        self.put(x, y, b'P');
    }
    pub fn brush(&mut self, x: i32, y: i32, size: i32, tile: u8) {
        if tile == b'P' {
            self.player(x, y);
            return;
        }
        let radius = size.clamp(1, 31) / 2;
        self.rectangle((x - radius, y - radius), (x + radius, y + radius), tile);
    }
    pub fn stroke(&mut self, from: (i32, i32), to: (i32, i32), size: i32, tile: u8) {
        let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()).max(1);
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            self.brush(
                (from.0 as f32 + (to.0 - from.0) as f32 * t).round() as i32,
                (from.1 as f32 + (to.1 - from.1) as f32 * t).round() as i32,
                size,
                tile,
            );
        }
    }
    pub fn rectangle(&mut self, a: (i32, i32), b: (i32, i32), tile: u8) {
        if tile == b'P' {
            self.player(b.0, b.1);
            return;
        }
        for y in a.1.min(b.1).max(0)..=a.1.max(b.1).min(self.grid.height as i32 - 1) {
            for x in a.0.min(b.0).max(0)..=a.0.max(b.0).min(self.grid.width as i32 - 1) {
                self.put(x, y, tile);
            }
        }
    }
    pub fn fill(&mut self, x: i32, y: i32, tile: u8) {
        if x < 0 || y < 0 || x >= self.grid.width as i32 || y >= self.grid.height as i32 {
            return;
        }
        if tile == b'P' {
            self.player(x, y);
            return;
        }
        let old = self.grid.tiles[y as usize * self.grid.width + x as usize];
        if old == tile {
            return;
        }
        let mut queue = VecDeque::from([(x, y)]);
        self.put(x, y, tile);
        while let Some((x, y)) = queue.pop_front() {
            for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if nx >= 0
                    && ny >= 0
                    && nx < self.grid.width as i32
                    && ny < self.grid.height as i32
                    && self.grid.tiles[ny as usize * self.grid.width + nx as usize] == old
                {
                    self.put(nx, ny, tile);
                    queue.push_back((nx, ny));
                }
            }
        }
    }
    fn record(&mut self, edit: Edit) {
        self.undo.push_back(Revision {
            edit,
            before: self.revision,
            after: self.next_revision,
        });
        self.revision = self.next_revision;
        self.next_revision += 1;
        self.redo.clear();
        let mut bytes: usize = self.undo.iter().map(|r| r.edit.bytes()).sum();
        while bytes > HISTORY_BYTES && self.undo.len() > 1 {
            bytes -= self.undo.pop_front().unwrap().edit.bytes();
        }
    }
    pub fn commit(&mut self) {
        let mut changes = vec![];
        for i in self.touched.drain(..) {
            let before = self.original[i].take().unwrap();
            let after = self.grid.tiles[i];
            if before != after {
                changes.push(Change {
                    index: i,
                    before,
                    after,
                });
            }
        }
        if !changes.is_empty() {
            self.record(Edit::Cells(changes));
        }
    }
    pub fn resize(&mut self, width: usize, height: usize) -> Result<(), String> {
        let mut after = Grid::new(width, height)?;
        if (width, height) == (self.grid.width, self.grid.height) {
            return Ok(());
        }
        self.commit();
        after.tiles.fill(b'.');
        for y in 0..height.min(self.grid.height) {
            let n = width.min(self.grid.width);
            after.tiles[y * width..y * width + n]
                .copy_from_slice(&self.grid.tiles[y * self.grid.width..y * self.grid.width + n]);
        }
        let before = std::mem::replace(&mut self.grid, after.clone());
        self.record(Edit::Resize { before, after });
        self.original = vec![None; width * height];
        self.generation += 1;
        Ok(())
    }
    pub fn undo(&mut self) {
        self.commit();
        if let Some(r) = self.undo.pop_back() {
            r.edit.apply(&mut self.grid, false);
            self.revision = r.before;
            self.redo.push(r);
            self.original.resize(self.grid.tiles.len(), None);
            self.generation += 1;
        }
    }
    pub fn redo(&mut self) {
        self.commit();
        if let Some(r) = self.redo.pop() {
            r.edit.apply(&mut self.grid, true);
            self.revision = r.after;
            self.undo.push_back(r);
            self.original.resize(self.grid.tiles.len(), None);
            self.generation += 1;
        }
    }
    /// Write beside the destination first; errors leave the previous map intact.
    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        self.commit();
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let name = path
            .file_name()
            .ok_or("Choose a file name")?
            .to_string_lossy();
        let temp = parent.join(format!(".{name}.{}.tmp", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("Could not create {}: {e}", temp.display()))?;
        let result = (|| -> std::io::Result<()> {
            file.write_all(self.grid.encode().as_bytes())?;
            file.sync_all()?;
            drop(file);
            std::fs::rename(&temp, path)
        })();
        if let Err(e) = result {
            // This temporary file was created by this save; preserve the destination.
            let _ = std::fs::remove_file(&temp);
            return Err(format!(
                "Could not save {}: {e}. Previous file preserved.",
                path.display()
            ));
        }
        self.saved_revision = Some(self.revision);
        Ok(())
    }
    pub fn validate(&self, name: &str) -> Result<Level, String> {
        let count = self
            .grid
            .tiles
            .iter()
            .filter(|&&c| matches!(c, b'P' | b'1'))
            .count();
        if count != 1 {
            return Err(format!("Place exactly one player start (found {count})."));
        }
        let level = Level::parse(name, &self.grid.encode())?;
        let unreachable = self
            .grid
            .tiles
            .iter()
            .enumerate()
            .filter(|&(i, &c)| matches!(c, b'E' | b'2' | b'+' | b'3') && !level.reachable[i])
            .count();
        if unreachable > 0 {
            return Err(format!(
                "{unreachable} enemy/medical markers cannot be reached from the player."
            ));
        }
        Ok(level)
    }
}
