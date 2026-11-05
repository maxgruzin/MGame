use macroquad::prelude::*;
use std::{collections::VecDeque, path::Path};

pub const TILE: f32 = 48.0;
pub const CRATE_HALF: f32 = 20.0;
pub const MAX_MAP_SIZE: usize = 1024;

/// Shared syntax validation; the editor can open unfinished, unplayable maps.
pub fn parse_grid(text: &str) -> Result<(usize, usize, Vec<u8>), String> {
    let rows: Vec<&str> = text
        .trim_start_matches('\u{feff}')
        .lines()
        .map(|s| s.trim_end_matches('\r'))
        .filter(|s| !s.is_empty())
        .collect();
    let width = rows.first().ok_or("The map is empty")?.len();
    if !(5..=MAX_MAP_SIZE).contains(&width) || !(5..=MAX_MAP_SIZE).contains(&rows.len()) {
        return Err(format!(
            "Map dimensions must be between 5 and {MAX_MAP_SIZE} tiles"
        ));
    }
    if rows.iter().any(|r| r.len() != width || !r.is_ascii()) {
        return Err("Map rows must have equal widths and contain ASCII characters".into());
    }
    let tiles: Vec<u8> = rows.iter().flat_map(|r| r.bytes()).collect();
    if let Some(&c) = tiles.iter().find(|&&c| !matches!(c, b'0'..=b'6' | b'a'..=b'y' | b'#' | b'.' | b'P' | b'E' | b'+' | b'T' | b'~' | b'B' | b'C')) {
        return Err(format!("Unknown map tile: {}", c as char));
    }
    Ok((width, rows.len(), tiles))
}

#[derive(Clone, Debug)]
pub struct Level {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<char>,
    pub spawn: Vec2,
    pub portals: Vec<Vec2>,
    pub medical: Vec<Vec2>,
    pub reachable: Vec<bool>,
    pub crates: Vec<Vec2>,
}

impl Level {
    pub fn builtin(index: usize) -> Self {
        crate::arenas::builtin(index)
    }
    pub fn parse(name: &str, text: &str) -> Result<Self, String> {
        let (width, height, bytes) = parse_grid(text)?;
        let tiles: Vec<char> = bytes.into_iter().map(char::from).collect();
        let mut level = Self {
            name: name.into(),
            width,
            height,
            tiles,
            spawn: Vec2::ZERO,
            portals: vec![],
            medical: vec![],
            reachable: vec![false; width * height],
            crates: vec![],
        };
        let start = level
            .tiles
            .iter()
            .position(|c| matches!(c, 'P' | '1'))
            .or_else(|| {
                level
                    .tiles
                    .iter()
                    .position(|&c| !Self::solid_tile(c) && c != 'C')
            })
            .ok_or("Map has no walkable tile")?;
        level.spawn = level.center(start);
        for i in 0..level.tiles.len() {
            if level.tiles[i] == 'C' {
                level.crates.push(level.center(i));
                level.tiles[i] = '.';
            }
        }
        let distances = level.flow_field(level.spawn);
        level.reachable = distances.iter().map(|&n| n != u32::MAX).collect();
        for i in 0..level.tiles.len() {
            if !level.reachable[i] {
                continue;
            }
            match level.tiles[i] {
                'E' | '2' => level.portals.push(level.center(i)),
                '+' | '3' => level.medical.push(level.center(i)),
                _ => {}
            }
        }
        if level.portals.is_empty() {
            let mut candidates: Vec<usize> = (0..level.tiles.len())
                .filter(|&i| {
                    level.reachable[i] && level.center(i).distance(level.spawn) > TILE * 3.0
                })
                .collect();
            candidates.sort_by_key(|&i| std::cmp::Reverse(distances[i]));
            for i in candidates {
                let p = level.center(i);
                if level.portals.iter().all(|q| p.distance(*q) > TILE * 4.0) {
                    level.portals.push(p);
                }
                if level.portals.len() == 6 {
                    break;
                }
            }
        }
        if level.portals.is_empty() {
            return Err("Map needs a larger connected play area".into());
        }
        Ok(level)
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Custom arena"),
            &text,
        )
    }
    pub fn solid_tile(c: char) -> bool {
        matches!(c, '#' | '4'..='6' | 'T' | '~')
    }
    pub fn solid(&self, x: i32, y: i32) -> bool {
        x < 0
            || y < 0
            || x >= self.width as i32
            || y >= self.height as i32
            || Self::solid_tile(self.tiles[y as usize * self.width + x as usize])
    }
    pub fn index(&self, p: Vec2) -> usize {
        let x = (p.x / TILE).floor().clamp(0.0, self.width as f32 - 1.0) as usize;
        let y = (p.y / TILE).floor().clamp(0.0, self.height as f32 - 1.0) as usize;
        y * self.width + x
    }
    pub fn center(&self, i: usize) -> Vec2 {
        vec2(
            i as f32 % self.width as f32 + 0.5,
            (i / self.width) as f32 + 0.5,
        ) * TILE
    }
    pub fn circle_free(&self, p: Vec2, r: f32) -> bool {
        self.static_free(p, r, false)
            && self
                .crates
                .iter()
                .all(|&c| !circle_box(p, r, c, CRATE_HALF))
    }
    pub fn projectile_free(&self, p: Vec2, r: f32) -> bool {
        self.static_free(p, r, true)
            && self
                .crates
                .iter()
                .all(|&c| !circle_box(p, r, c, CRATE_HALF))
    }
    fn static_free(&self, p: Vec2, r: f32, projectile: bool) -> bool {
        for y in ((p.y - r) / TILE).floor() as i32..=((p.y + r) / TILE).floor() as i32 {
            for x in ((p.x - r) / TILE).floor() as i32..=((p.x + r) / TILE).floor() as i32 {
                let water = x >= 0
                    && y >= 0
                    && x < self.width as i32
                    && y < self.height as i32
                    && self.tiles[y as usize * self.width + x as usize] == '~';
                if self.solid(x, y) && !(projectile && water) {
                    let nearest = p.clamp(
                        vec2(x as f32, y as f32) * TILE,
                        vec2(x as f32 + 1.0, y as f32 + 1.0) * TILE,
                    );
                    if p.distance_squared(nearest) < r * r - 0.001 {
                        return false;
                    }
                }
            }
        }
        true
    }
    pub fn terrain_speed(&self, p: Vec2) -> f32 {
        if self.tiles[self.index(p)] == 'B' {
            0.52
        } else {
            1.0
        }
    }
    pub fn clear_shot(&self, a: Vec2, b: Vec2) -> bool {
        let steps = (a.distance(b) / 4.0).ceil().max(1.0) as usize;
        (0..=steps).all(|i| self.projectile_free(a.lerp(b, i as f32 / steps as f32), 1.0))
    }
    /// Player-only pushing; crates remain solid, indestructible cover for every actor.
    pub fn push_slide(
        &mut self,
        p: Vec2,
        delta: Vec2,
        radius: f32,
        actors: &[(Vec2, f32)],
    ) -> Vec2 {
        let steps = (delta.length() / 4.0).ceil().max(1.0) as usize;
        let step = delta / steps as f32;
        let mut pos = p;
        for _ in 0..steps {
            for axis in [vec2(step.x, 0.0), vec2(0.0, step.y)] {
                if axis.length_squared() < 0.000001 {
                    continue;
                }
                let contact = self
                    .crates
                    .iter()
                    .position(|&c| circle_box(pos + axis, radius, c, CRATE_HALF));
                if let Some(i) = contact {
                    let slow = axis * 0.5;
                    let before = self.crates.clone();
                    if self.push_crate(i, slow, actors, &mut vec![false; self.crates.len()])
                        && self.circle_free(pos + slow, radius)
                    {
                        pos += slow;
                    } else {
                        self.crates = before;
                        pos = self.slide(pos, axis, radius);
                    }
                } else {
                    pos = self.slide(pos, axis, radius);
                }
            }
        }
        pos
    }
    fn push_crate(
        &mut self,
        i: usize,
        delta: Vec2,
        actors: &[(Vec2, f32)],
        visited: &mut [bool],
    ) -> bool {
        if visited[i] {
            return false;
        }
        visited[i] = true;
        let dest = self.crates[i] + delta;
        let lo = dest - Vec2::splat(CRATE_HALF - 0.01);
        let hi = dest + Vec2::splat(CRATE_HALF - 0.01);
        for y in (lo.y / TILE).floor() as i32..=(hi.y / TILE).floor() as i32 {
            for x in (lo.x / TILE).floor() as i32..=(hi.x / TILE).floor() as i32 {
                if self.solid(x, y) {
                    return false;
                }
            }
        }
        if actors
            .iter()
            .any(|&(p, r)| circle_box(p, r, dest, CRATE_HALF))
        {
            return false;
        }
        for j in 0..self.crates.len() {
            if i != j
                && (dest - self.crates[j])
                    .abs()
                    .cmplt(Vec2::splat(CRATE_HALF * 2.0 - 0.001))
                    .all()
                && !self.push_crate(j, delta, actors, visited)
            {
                return false;
            }
        }
        self.crates[i] = dest;
        true
    }
    /// Small swept steps plus independent axes preserve tangential motion against walls.
    pub fn slide(&self, p: Vec2, delta: Vec2, radius: f32) -> Vec2 {
        let steps = (delta.length() / (radius * 0.45)).ceil().max(1.0) as usize;
        let step = delta / steps as f32;
        let mut result = p;
        for _ in 0..steps {
            for axis in [vec2(step.x, 0.0), vec2(0.0, step.y)] {
                if self.circle_free(result + axis, radius) {
                    result += axis;
                } else {
                    let (mut lo, mut hi) = (0.0, 1.0);
                    for _ in 0..8 {
                        let mid = (lo + hi) * 0.5;
                        if self.circle_free(result + axis * mid, radius) {
                            lo = mid;
                        } else {
                            hi = mid;
                        }
                    }
                    result += axis * lo;
                }
            }
        }
        result
    }
    pub fn clear_line(&self, a: Vec2, b: Vec2, radius: f32) -> bool {
        let steps = (a.distance(b) / 6.0).ceil().max(1.0) as usize;
        (0..=steps).all(|i| self.circle_free(a.lerp(b, i as f32 / steps as f32), radius))
    }
    /// Breadth-first distance field, shared by every enemy. Cardinal edges cannot cut corners.
    pub fn flow_field(&self, target: Vec2) -> Vec<u32> {
        let mut field = vec![u32::MAX; self.tiles.len()];
        // Rasterize crate collisions once, rather than testing every crate at every
        // BFS edge. Cardinal moves only touch their two tiles (19 < TILE / 2).
        let mut blocked = vec![[false; 3]; self.tiles.len()];
        for &c in &self.crates {
            let lo = ((c - Vec2::splat(CRATE_HALF + 19.0)) / TILE).floor();
            let hi = ((c + Vec2::splat(CRATE_HALF + 19.0)) / TILE).ceil();
            for y in (lo.y as i32 - 1).max(0)..=(hi.y as i32).min(self.height as i32 - 1) {
                for x in (lo.x as i32 - 1).max(0)..=(hi.x as i32).min(self.width as i32 - 1) {
                    let i = y as usize * self.width + x as usize;
                    let p = self.center(i);
                    for (k, offset) in [Vec2::ZERO, vec2(TILE * 0.5, 0.0), vec2(0.0, TILE * 0.5)]
                        .iter()
                        .enumerate()
                    {
                        blocked[i][k] |= circle_box(p + *offset, 19.0, c, CRATE_HALF);
                    }
                }
            }
        }
        let root = self.index(target);
        field[root] = 0;
        let mut queue = VecDeque::from([root]);
        while let Some(i) = queue.pop_front() {
            let (x, y) = ((i % self.width) as i32, (i / self.width) as i32);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if self.solid(nx, ny) {
                    continue;
                }
                let n = ny as usize * self.width + nx as usize;
                if field[n] != u32::MAX {
                    continue;
                }
                let edge_blocked = match (dx, dy) {
                    (1, 0) => blocked[i][1],
                    (-1, 0) => blocked[n][1],
                    (0, 1) => blocked[i][2],
                    _ => blocked[n][2],
                };
                if blocked[n][0] || edge_blocked || Self::solid_tile(self.tiles[i]) {
                    continue;
                }
                if field[n] == u32::MAX {
                    field[n] = field[i] + 1;
                    queue.push_back(n);
                }
            }
        }
        field
    }
    // A moved crate can block a tile's center while leaving room for an actor
    // near its edge. Attach that actor to a visible neighboring route instead.
    fn route_anchor(&self, p: Vec2, field: &[u32], radius: f32) -> Option<usize> {
        let i = self.index(p);
        if field[i] != u32::MAX && self.clear_line(p, self.center(i), radius) {
            return Some(i);
        }
        let (x, y) = ((i % self.width) as i32, (i / self.width) as i32);
        let mut best = None;
        for ny in (y - 1).max(0)..=(y + 1).min(self.height as i32 - 1) {
            for nx in (x - 1).max(0)..=(x + 1).min(self.width as i32 - 1) {
                let n = ny as usize * self.width + nx as usize;
                if field[n] != u32::MAX
                    && best.is_none_or(|b| field[n] < field[b])
                    && self.clear_line(p, self.center(n), radius)
                {
                    best = Some(n);
                }
            }
        }
        best
    }

    pub fn route_reachable(&self, p: Vec2, target: Vec2, field: &[u32], radius: f32) -> bool {
        field[self.index(p)] != u32::MAX
            || self.route_anchor(p, field, radius).is_some()
            || self.clear_line(p, target, radius)
    }

    pub fn waypoint(&self, p: Vec2, target: Vec2, field: &[u32], radius: f32) -> Vec2 {
        if self.clear_line(p, target, radius) {
            return target;
        }
        let Some(mut best) = self.route_anchor(p, field, radius) else {
            return p;
        };
        // Look a few tiles ahead along the descending field, skipping intermediate
        // centers only when the whole body can reach the next point directly.
        // This rounds tree detours without cutting corners or crossing cover.
        for _ in 0..4 {
            let (x, y) = ((best % self.width) as i32, (best / self.width) as i32);
            let previous = best;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if self.solid(x + dx, y + dy) {
                    continue;
                }
                let n = (y + dy) as usize * self.width + (x + dx) as usize;
                if field[n] < field[best] && self.clear_line(p, self.center(n), radius) {
                    best = n;
                }
            }
            if best == previous {
                break;
            }
        }
        self.center(best)
    }
}

pub fn circle_box(p: Vec2, r: f32, center: Vec2, half: f32) -> bool {
    p.distance_squared(p.clamp(center - Vec2::splat(half), center + Vec2::splat(half)))
        < r * r - 0.001
}

#[cfg(test)]
mod tests {
    use super::*;
    fn arena() -> Level {
        Level::parse(
            "test",
            "#########\n#P..#..E#\n#...#...#\n#...#...#\n#.......#\n#########",
        )
        .unwrap()
    }
    #[test]
    fn slides_along_wall_without_tunneling() {
        let l = arena();
        let p = vec2(180.0, 80.0);
        let q = l.slide(p, vec2(600.0, 80.0), 12.0);
        assert!(q.x <= 180.01);
        assert!((q.y - 160.0).abs() < 0.01);
        assert!(l.circle_free(q, 12.0));
    }
    #[test]
    fn path_routes_around_barrier() {
        let l = arena();
        let field = l.flow_field(l.spawn);
        let mut p = l.portals[0];
        for _ in 0..2400 {
            let goal = l.waypoint(p, l.spawn, &field, 13.0);
            p = l.slide(p, (goal - p).normalize_or_zero() * 1.5, 13.0);
        }
        assert!(p.distance(l.spawn) < 4.0, "enemy stuck at {p:?}");
    }
    #[test]
    fn rejects_bad_maps() {
        assert!(Level::parse("empty", "").is_err());
        assert!(Level::parse("bad", "#####\n#P..#\n#..#\n#..E#\n#####").is_err());
    }
    #[test]
    fn legacy_ground_is_walkable() {
        assert!(!Level::solid_tile('a'));
        assert!(Level::solid_tile('5'));
    }
}
