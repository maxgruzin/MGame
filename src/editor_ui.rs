use crate::render::{text, MINT, MUTED, PANEL};
use macroquad::prelude::*;
use mgame::{
    editor::{Document, Grid},
    level::Level,
};
use std::path::PathBuf;

const INK: Color = Color::new(0.055, 0.075, 0.095, 1.0);
const EDGE: Color = Color::new(0.16, 0.23, 0.27, 1.0);
const GOLD: Color = Color::new(1.0, 0.77, 0.40, 1.0);
const PALETTE: [(u8, &str, &str); 8] = [
    (b'.', "Grass", "Walkable ground"),
    (b'#', "Tree", "Blocks actors and shots"),
    (b'~', "Pond", "Blocks actors; shots pass"),
    (b'B', "Bush", "Slows movement"),
    (b'C', "Wooden block", "Pushable cover"),
    (b'+', "Medical pad", "Restores nearby health"),
    (b'P', "Player start", "One start per map"),
    (b'E', "Enemy entry", "Must connect to player"),
];

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Brush,
    Rectangle,
    Fill,
}
#[derive(Clone, Copy)]
enum Pending {
    New,
    Open,
    Exit,
    Quit,
}
#[derive(Clone, Copy)]
enum Dialog {
    New,
    Resize,
    Open,
    Save,
    Unsaved(Pending),
    Overwrite,
}
pub enum Action {
    None,
    Exit,
    Quit,
    Play(Level),
}

pub struct Editor {
    pub doc: Document,
    path: Option<PathBuf>,
    tile: u8,
    tool: Tool,
    brush: i32,
    center: Vec2,
    zoom: f32,
    grid_lines: bool,
    previous_mouse: Vec2,
    stroke: Option<(i32, i32)>,
    rectangle: Option<(i32, i32)>,
    stroke_tile: u8,
    panning: bool,
    dialog: Option<Dialog>,
    fields: [String; 2],
    active_field: usize,
    select_all: bool,
    cursor: usize,
    browser_page: usize,
    pending: Option<Pending>,
    save_target: Option<PathBuf>,
    status: String,
    overview: Option<Texture2D>,
    overview_generation: u64,
    overview_time: f64,
    fit_requested: bool,
}
impl Editor {
    pub fn new(grid: Grid, path: Option<PathBuf>) -> Self {
        let saved = path.is_some();
        Self {
            doc: Document::new(grid, saved),
            path,
            tile: b'#',
            tool: Tool::Brush,
            brush: 1,
            center: Vec2::ZERO,
            zoom: 24.0,
            grid_lines: true,
            previous_mouse: Vec2::ZERO,
            stroke: None,
            rectangle: None,
            stroke_tile: b'#',
            panning: false,
            dialog: None,
            fields: [String::new(), String::new()],
            active_field: 0,
            select_all: true,
            cursor: 0,
            browser_page: 0,
            pending: None,
            save_target: None,
            status: "Choose a tile, then drag to paint. Start with New for a larger canvas.".into(),
            overview: None,
            overview_generation: u64::MAX,
            overview_time: -1.0,
            fit_requested: true,
        }
    }
    pub fn blank() -> Self {
        Self::new(Grid::new(64, 64).unwrap(), None)
    }
    pub fn confirm_quit(&mut self) -> Action {
        self.request(Pending::Quit)
    }
    pub fn notice(&mut self, message: String) {
        self.status = message;
    }
    pub fn smoke_step(&mut self, frame: usize) {
        if frame == 0 {
            self.doc.rectangle((7, 7), (19, 12), b'~');
            self.doc.rectangle((24, 8), (30, 20), b'B');
            self.doc.rectangle((8, 24), (15, 24), b'C');
            self.doc.brush(20, 20, 1, b'+');
            self.doc.commit();
            self.center = vec2(22.0, 18.0);
            self.zoom = 20.0;
            self.fit_requested = false;
            assert!(self.doc.validate("Editor smoke test").is_ok());
        }
        if frame == 10 {
            self.doc.resize(1024, 1024).unwrap();
            self.doc.rectangle((100, 100), (800, 106), b'~');
            self.doc.commit();
            self.doc.undo();
            self.doc.redo();
            self.fit_requested = true;
            self.overview = None;
        }
        if frame == 20 {
            self.open_dialog(Dialog::New);
        }
    }
    pub fn from_entry(entry: &crate::Entry) -> Result<Self, String> {
        match &entry.source {
            Some(path) => {
                let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
                Ok(Self::new(Grid::parse(&data)?, Some(path.clone())))
            }
            None => Ok(Self::new(
                Grid::from_level(&Level::builtin(entry.builtin)),
                None,
            )),
        }
    }
    fn title(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled.arena".into())
    }
    fn fit(&mut self, view: Rect) {
        self.center = vec2(self.doc.grid.width as f32, self.doc.grid.height as f32) * 0.5;
        self.zoom = ((view.w - 40.0) / self.doc.grid.width as f32)
            .min((view.h - 40.0) / self.doc.grid.height as f32)
            .clamp(0.1, 64.0);
        self.fit_requested = false;
    }
    fn world(&self, mouse: Vec2, view: Rect) -> Vec2 {
        self.center + (mouse - view.center()) / self.zoom
    }
    fn zoom_at(&mut self, mouse: Vec2, view: Rect, wheel: f32) {
        let anchor = self.world(mouse, view);
        self.zoom = (self.zoom * 1.18_f32.powf(wheel.clamp(-5.0, 5.0))).clamp(0.1, 80.0);
        self.center = anchor - (mouse - view.center()) / self.zoom;
    }
    fn cell(&self, mouse: Vec2, view: Rect) -> Option<(i32, i32)> {
        if !view.contains(mouse) {
            return None;
        }
        let p = self.world(mouse, view).floor();
        (p.x >= 0.0
            && p.y >= 0.0
            && p.x < self.doc.grid.width as f32
            && p.y < self.doc.grid.height as f32)
            .then_some((p.x as i32, p.y as i32))
    }
    fn open_dialog(&mut self, dialog: Dialog) {
        self.finish_stroke();
        self.dialog = Some(dialog);
        self.active_field = 0;
        self.select_all = true;
        self.fields = match dialog {
            Dialog::New => ["256".into(), "256".into()],
            Dialog::Resize => [
                self.doc.grid.width.to_string(),
                self.doc.grid.height.to_string(),
            ],
            Dialog::Open => ["levels/".into(), String::new()],
            Dialog::Save => [
                self.path
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("levels/My Arena.arena"))
                    .to_string_lossy()
                    .into_owned(),
                String::new(),
            ],
            _ => [String::new(), String::new()],
        };
        self.cursor = self.fields[0].len();
        while get_char_pressed().is_some() {}
    }
    fn request(&mut self, pending: Pending) -> Action {
        self.finish_stroke();
        if self.doc.dirty() {
            self.open_dialog(Dialog::Unsaved(pending));
            Action::None
        } else {
            self.proceed(pending)
        }
    }
    fn proceed(&mut self, pending: Pending) -> Action {
        self.dialog = None;
        match pending {
            Pending::New => self.open_dialog(Dialog::New),
            Pending::Open => self.open_dialog(Dialog::Open),
            Pending::Exit => return Action::Exit,
            Pending::Quit => return Action::Quit,
        }
        Action::None
    }
    fn finish_stroke(&mut self) {
        self.doc.commit();
        self.stroke = None;
        self.rectangle = None;
    }
    fn save(&mut self, target: PathBuf) -> Action {
        match self.doc.save(&target) {
            Ok(()) => {
                self.status = format!("Saved {}", target.display());
                self.path = Some(target);
                self.dialog = None;
                if let Some(p) = self.pending.take() {
                    return self.proceed(p);
                }
            }
            Err(e) => {
                self.open_dialog(Dialog::Save);
                self.fields[0] = target.to_string_lossy().into_owned();
                self.cursor = self.fields[0].len();
                self.status = e;
            }
        }
        Action::None
    }
    fn save_request(&mut self, save_as: bool) -> Action {
        self.finish_stroke();
        if !save_as {
            if let Some(path) = self.path.clone() {
                return self.save(path);
            }
        }
        self.open_dialog(Dialog::Save);
        Action::None
    }
    fn play(&mut self) -> Action {
        self.finish_stroke();
        match self.doc.validate(&self.title()) {
            Ok(level) => {
                self.status = "Play test ended. Your edits are preserved.".into();
                Action::Play(level)
            }
            Err(e) => {
                self.status = format!("Cannot play: {e}");
                Action::None
            }
        }
    }
    pub fn frame(&mut self, scale: f32) -> Action {
        let (w, h) = crate::ui_begin(scale);
        let mouse = Vec2::from(mouse_position()) / scale;
        let view = Rect::new(250.0, 112.0, (w - 266.0).max(100.0), (h - 178.0).max(100.0));
        if self.fit_requested {
            self.fit(view);
        }
        show_mouse(true);
        if is_quit_requested() {
            return self.request(Pending::Quit);
        }
        let active = self.dialog.is_none();
        if active {
            if is_key_pressed(KeyCode::Escape) {
                return self.request(Pending::Exit);
            }
            if ctrl() {
                if is_key_pressed(KeyCode::S) {
                    return self.save_request(shift());
                }
                if is_key_pressed(KeyCode::O) {
                    return self.request(Pending::Open);
                }
                if is_key_pressed(KeyCode::N) {
                    return self.request(Pending::New);
                }
                if is_key_pressed(KeyCode::Z) {
                    self.finish_stroke();
                    if shift() {
                        self.doc.redo();
                    } else {
                        self.doc.undo();
                    }
                }
                if is_key_pressed(KeyCode::Y) {
                    self.finish_stroke();
                    self.doc.redo();
                }
            } else {
                if is_key_pressed(KeyCode::B) {
                    self.finish_stroke();
                    self.tool = Tool::Brush;
                }
                if is_key_pressed(KeyCode::R) {
                    self.finish_stroke();
                    self.tool = Tool::Rectangle;
                }
                if is_key_pressed(KeyCode::F) {
                    self.finish_stroke();
                    self.tool = Tool::Fill;
                }
                if is_key_pressed(KeyCode::G) {
                    self.grid_lines = !self.grid_lines;
                }
                if is_key_pressed(KeyCode::Home) {
                    self.fit(view);
                }
                if is_key_pressed(KeyCode::LeftBracket) {
                    self.brush = (self.brush - 2).max(1);
                }
                if is_key_pressed(KeyCode::RightBracket) {
                    self.brush = (self.brush + 2).min(31);
                }
                for (i, key) in [
                    KeyCode::Key1,
                    KeyCode::Key2,
                    KeyCode::Key3,
                    KeyCode::Key4,
                    KeyCode::Key5,
                    KeyCode::Key6,
                    KeyCode::Key7,
                    KeyCode::Key8,
                ]
                .iter()
                .enumerate()
                {
                    if is_key_pressed(*key) {
                        self.tile = PALETTE[i].0;
                    }
                }
            }
            if is_key_pressed(KeyCode::F5) {
                return self.play();
            }
            self.canvas_input(view, mouse);
        }
        draw_rectangle(0.0, 0.0, w, h, INK);
        self.canvas(view, mouse);
        draw_rectangle(0.0, 0.0, w, 104.0, PANEL);
        text("MAP STUDIO", 20.0, 33.0, 26.0, MINT);
        text(
            &format!(
                "{}{}",
                self.title(),
                if self.doc.dirty() {
                    "  /  UNSAVED"
                } else {
                    "  /  SAVED"
                }
            ),
            250.0,
            32.0,
            21.0,
            WHITE,
        );
        let labels = [
            "New",
            "Open",
            "Save",
            "Save as",
            "Resize",
            "Undo",
            "Redo",
            "Validate",
            "Play test",
            "Back",
        ];
        for (i, label) in labels.iter().enumerate() {
            let enabled = active
                && match i {
                    5 => self.doc.can_undo(),
                    6 => self.doc.can_redo(),
                    _ => true,
                };
            if ui_button(
                Rect::new(18.0 + i as f32 * 112.0, 55.0, 104.0, 35.0),
                label,
                mouse,
                enabled,
                i == 8,
            ) {
                match i {
                    0 => return self.request(Pending::New),
                    1 => return self.request(Pending::Open),
                    2 => return self.save_request(false),
                    3 => return self.save_request(true),
                    4 => self.open_dialog(Dialog::Resize),
                    5 => {
                        self.finish_stroke();
                        self.doc.undo();
                    }
                    6 => {
                        self.finish_stroke();
                        self.doc.redo();
                    }
                    7 => {
                        self.finish_stroke();
                        self.status = match self.doc.validate(&self.title()) {
                            Ok(l) => format!(
                                "Ready to play: {} x {} tiles, {} reachable enemy entries.",
                                l.width,
                                l.height,
                                l.portals.len()
                            ),
                            Err(e) => format!("Needs attention: {e}"),
                        };
                    }
                    8 => return self.play(),
                    9 => return self.request(Pending::Exit),
                    _ => {}
                }
            }
        }
        draw_rectangle(12.0, 112.0, 222.0, h - 178.0, PANEL);
        text("TERRAIN & MARKERS", 24.0, 137.0, 16.0, MUTED);
        for (i, &(tile, label, _)) in PALETTE.iter().enumerate() {
            let r = Rect::new(22.0, 150.0 + i as f32 * 31.0, 202.0, 27.0);
            if ui_button(
                r,
                &format!("     {}  {label}", i + 1),
                mouse,
                active,
                self.tile == tile,
            ) {
                self.tile = tile;
            }
            swatch(tile, Rect::new(r.x + 8.0, r.y + 5.0, 17.0, 17.0));
        }
        let hint = PALETTE
            .iter()
            .find(|p| p.0 == self.tile)
            .map_or("Legacy terrain", |p| p.2);
        text(hint, 23.0, 417.0, 14.0, MUTED);
        for (i, (tool, label)) in [
            (Tool::Brush, "Brush"),
            (Tool::Rectangle, "Rect"),
            (Tool::Fill, "Fill"),
        ]
        .iter()
        .enumerate()
        {
            if ui_button(
                Rect::new(22.0 + i as f32 * 69.0, 434.0, 64.0, 31.0),
                label,
                mouse,
                active,
                self.tool == *tool,
            ) {
                self.finish_stroke();
                self.tool = *tool;
            }
        }
        if ui_button(
            Rect::new(22.0, 475.0, 34.0, 28.0),
            "-",
            mouse,
            active,
            false,
        ) {
            self.brush = (self.brush - 2).max(1);
        }
        text(
            &format!(
                "Brush: {} tiles",
                if self.tile == b'P' { 1 } else { self.brush }
            ),
            63.0,
            494.0,
            17.0,
            WHITE,
        );
        if ui_button(
            Rect::new(190.0, 475.0, 34.0, 28.0),
            "+",
            mouse,
            active,
            false,
        ) {
            self.brush = (self.brush + 2).min(31);
        }
        if ui_button(
            Rect::new(22.0, 513.0, 96.0, 28.0),
            "Fit / Home",
            mouse,
            active,
            false,
        ) {
            self.fit(view);
        }
        if ui_button(
            Rect::new(126.0, 513.0, 98.0, 28.0),
            "Grid / G",
            mouse,
            active,
            self.grid_lines,
        ) {
            self.grid_lines = !self.grid_lines;
        }
        let mini = Rect::new(24.0, 552.0, 198.0, (h - 632.0).clamp(50.0, 180.0));
        self.minimap(mini, view, mouse, active);
        draw_rectangle(0.0, h - 58.0, w, 58.0, PANEL);
        text(
            &shorten(&self.status, (w / 8.3) as usize),
            20.0,
            h - 36.0,
            17.0,
            GOLD,
        );
        let coord = self.cell(mouse, view).map_or(String::new(), |(x, y)| {
            format!("  |  Tile {}, {}", x + 1, y + 1)
        });
        text(
            &format!(
                "{} x {}  |  {:.0}%{coord}",
                self.doc.grid.width,
                self.doc.grid.height,
                self.zoom / 32.0 * 100.0
            ),
            20.0,
            h - 13.0,
            15.0,
            MUTED,
        );
        text(
            "Wheel: zoom   |   Middle drag / Space: pan   |   Right drag: erase   |   F5: test",
            390.0,
            h - 13.0,
            15.0,
            MUTED,
        );
        self.previous_mouse = mouse;
        let result = if let Some(dialog) = self.dialog {
            self.draw_dialog(dialog, w, h, mouse)
        } else {
            Action::None
        };
        set_default_camera();
        result
    }
    fn canvas_input(&mut self, view: Rect, mouse: Vec2) {
        let pan_key = is_key_down(KeyCode::Space);
        if view.contains(mouse)
            && (is_mouse_button_pressed(MouseButton::Middle)
                || (pan_key && is_mouse_button_pressed(MouseButton::Left)))
        {
            self.finish_stroke();
            self.panning = true;
        }
        if self.panning {
            if is_mouse_button_down(MouseButton::Middle)
                || (pan_key && is_mouse_button_down(MouseButton::Left))
            {
                self.center -= (mouse - self.previous_mouse) / self.zoom;
            } else {
                self.panning = false;
            }
        }
        if view.contains(mouse) {
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                self.zoom_at(mouse, view, wheel);
            }
        }
        let movement = vec2(
            (is_key_down(KeyCode::Right) as i32 - is_key_down(KeyCode::Left) as i32) as f32,
            (is_key_down(KeyCode::Down) as i32 - is_key_down(KeyCode::Up) as i32) as f32,
        );
        self.center += movement * get_frame_time().min(0.1) * 600.0 / self.zoom;
        self.center = self.center.clamp(
            Vec2::ZERO,
            vec2(self.doc.grid.width as f32, self.doc.grid.height as f32),
        );
        if self.panning || pan_key {
            return;
        }
        let cell = self.cell(mouse, view);
        let pressed = is_mouse_button_pressed(MouseButton::Left)
            || is_mouse_button_pressed(MouseButton::Right);
        let down =
            is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right);
        if let Some(p) = cell {
            if pressed {
                self.stroke_tile = if is_mouse_button_pressed(MouseButton::Right) {
                    b'.'
                } else {
                    self.tile
                };
                if is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt) {
                    self.tile =
                        self.doc.grid.tiles[p.1 as usize * self.doc.grid.width + p.0 as usize];
                    return;
                }
                if self.stroke_tile == b'P' {
                    self.doc.player(p.0, p.1);
                    self.doc.commit();
                } else {
                    match self.tool {
                        Tool::Brush => {
                            self.doc.brush(p.0, p.1, self.brush, self.stroke_tile);
                            self.stroke = Some(p);
                        }
                        Tool::Rectangle => self.rectangle = Some(p),
                        Tool::Fill => {
                            self.doc.fill(p.0, p.1, self.stroke_tile);
                            self.doc.commit();
                        }
                    }
                }
            } else if down {
                if let Some(previous) = self.stroke {
                    self.doc.stroke(previous, p, self.brush, self.stroke_tile);
                    self.stroke = Some(p);
                }
            }
        }
        if !down {
            if let Some(start) = self.rectangle.take() {
                let end = self.world(mouse, view).floor().clamp(
                    Vec2::ZERO,
                    vec2(
                        self.doc.grid.width as f32 - 1.0,
                        self.doc.grid.height as f32 - 1.0,
                    ),
                );
                self.doc
                    .rectangle(start, (end.x as i32, end.y as i32), self.stroke_tile);
            }
            self.doc.commit();
            self.stroke = None;
        }
    }
    fn refresh_overview(&mut self) {
        if self.overview_generation == self.doc.generation && self.overview.is_some() {
            return;
        }
        if get_time() - self.overview_time < 0.15 && self.overview.is_some() {
            return;
        }
        let g = &self.doc.grid;
        let width = g.width.min(256);
        let height = g.height.min(256);
        let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);
        // Keep narrow walls and single-cell markers visible at overview scale.
        let mut priorities = vec![0; width * height];
        for (i, &c) in g.tiles.iter().enumerate() {
            let x = (i % g.width) * width / g.width;
            let y = (i / g.width) * height / g.height;
            let priority = match c {
                b'P' | b'1' => 9,
                b'E' | b'2' => 8,
                b'+' | b'3' => 7,
                b'C' => 6,
                b'#' | b'T' | b'4'..=b'6' => 5,
                b'~' => 4,
                b'B' => 3,
                _ => 1,
            };
            if priority > priorities[y * width + x] {
                priorities[y * width + x] = priority;
                image.set_pixel(x as u32, y as u32, tile_color(c));
            }
        }
        match &self.overview {
            Some(t) if t.width() == width as f32 && t.height() == height as f32 => t.update(&image),
            _ => {
                let t = Texture2D::from_image(&image);
                t.set_filter(FilterMode::Nearest);
                self.overview = Some(t);
            }
        }
        self.overview_generation = self.doc.generation;
        self.overview_time = get_time();
    }
    fn canvas(&mut self, view: Rect, mouse: Vec2) {
        draw_rectangle(view.x, view.y, view.w, view.h, Color::from_hex(0x111c24));
        self.refresh_overview();
        let origin = view.center() - self.center * self.zoom;
        let grid = &self.doc.grid;
        if self.zoom < 3.0 {
            let map = Rect::new(
                origin.x,
                origin.y,
                grid.width as f32 * self.zoom,
                grid.height as f32 * self.zoom,
            );
            if let Some(clip) = map.intersect(view) {
                if let Some(t) = &self.overview {
                    draw_texture_ex(
                        t,
                        clip.x,
                        clip.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(clip.w, clip.h)),
                            source: Some(Rect::new(
                                (clip.x - map.x) / map.w * t.width(),
                                (clip.y - map.y) / map.h * t.height(),
                                clip.w / map.w * t.width(),
                                clip.h / map.h * t.height(),
                            )),
                            ..Default::default()
                        },
                    );
                }
            }
        } else {
            let lo = self
                .world(vec2(view.x, view.y), view)
                .floor()
                .max(Vec2::ZERO);
            let hi = self
                .world(vec2(view.right(), view.bottom()), view)
                .ceil()
                .min(vec2(grid.width as f32, grid.height as f32));
            for y in lo.y as usize..hi.y.max(0.0) as usize {
                for x in lo.x as usize..hi.x.max(0.0) as usize {
                    let r = Rect::new(
                        origin.x + x as f32 * self.zoom,
                        origin.y + y as f32 * self.zoom,
                        self.zoom,
                        self.zoom,
                    );
                    if let Some(clip) = r.intersect(view) {
                        let tile = grid.tiles[y * grid.width + x];
                        draw_rectangle(clip.x, clip.y, clip.w, clip.h, tile_color(tile));
                        if self.zoom >= 15.0
                            && view.contains(vec2(r.x, r.y))
                            && view.contains(vec2(r.right(), r.bottom()))
                        {
                            if tile != b'.' && tile != b'0' && !tile.is_ascii_lowercase() {
                                swatch(tile, r);
                            }
                            if self.grid_lines {
                                draw_rectangle_lines(
                                    r.x,
                                    r.y,
                                    r.w,
                                    r.h,
                                    0.5,
                                    Color::new(0.0, 0.0, 0.0, 0.25),
                                );
                            }
                        }
                    }
                }
            }
        }
        if self.dialog.is_none() {
            if let Some((x, y)) = self.cell(mouse, view) {
                let (a, b) = if let Some(start) = self.rectangle {
                    (
                        vec2(start.0.min(x) as f32, start.1.min(y) as f32),
                        vec2((start.0.max(x) + 1) as f32, (start.1.max(y) + 1) as f32),
                    )
                } else {
                    let r = if self.tile == b'P' || self.tool != Tool::Brush {
                        0
                    } else {
                        self.brush / 2
                    };
                    (
                        vec2((x - r) as f32, (y - r) as f32),
                        vec2((x + r + 1) as f32, (y + r + 1) as f32),
                    )
                };
                let p = origin + a * self.zoom;
                let size = (b - a) * self.zoom;
                if let Some(r) = Rect::new(p.x, p.y, size.x, size.y).intersect(view) {
                    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.5, 1.0, 0.85, 0.18));
                    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, MINT);
                }
            }
        }
        draw_rectangle_lines(view.x, view.y, view.w, view.h, 1.0, EDGE);
        text("CANVAS", view.x + 12.0, view.y + 23.0, 15.0, MUTED);
    }
    fn minimap(&mut self, area: Rect, view: Rect, mouse: Vec2, active: bool) {
        let factor =
            (area.w / self.doc.grid.width as f32).min(area.h / self.doc.grid.height as f32);
        let size = vec2(self.doc.grid.width as f32, self.doc.grid.height as f32) * factor;
        let origin = area.center() - size * 0.5;
        let rect = Rect::new(origin.x, origin.y, size.x, size.y);
        draw_rectangle(area.x, area.y, area.w, area.h, INK);
        if let Some(t) = &self.overview {
            draw_texture_ex(
                t,
                origin.x,
                origin.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(size),
                    ..Default::default()
                },
            );
        }
        let lo = origin + (self.center - vec2(view.w, view.h) * 0.5 / self.zoom) * factor;
        if let Some(r) = Rect::new(
            lo.x,
            lo.y,
            view.w / self.zoom * factor,
            view.h / self.zoom * factor,
        )
        .intersect(rect)
        {
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, WHITE);
        }
        if active
            && rect.contains(mouse)
            && is_mouse_button_down(MouseButton::Left)
            && self.stroke.is_none()
            && self.rectangle.is_none()
        {
            self.center = (mouse - origin) / factor;
        }
        draw_rectangle_lines(area.x, area.y, area.w, area.h, 1.0, EDGE);
    }
    fn draw_dialog(&mut self, dialog: Dialog, w: f32, h: f32, mouse: Vec2) -> Action {
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.78));
        let r = Rect::new(w * 0.5 - 350.0, h * 0.5 - 260.0, 700.0, 520.0);
        draw_rectangle(r.x, r.y, r.w, r.h, INK);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, EDGE);
        let x = r.x + 26.0;
        let y = r.y + 40.0;
        let title = match dialog {
            Dialog::New => "NEW MAP",
            Dialog::Resize => "RESIZE MAP",
            Dialog::Open => "OPEN MAP",
            Dialog::Save => "SAVE MAP AS",
            Dialog::Unsaved(_) => "SAVE YOUR CHANGES?",
            Dialog::Overwrite => "REPLACE EXISTING FILE?",
        };
        text(title, x, y, 27.0, MINT);
        if is_key_pressed(KeyCode::Escape)
            || ui_button(
                Rect::new(r.right() - 116.0, r.bottom() - 52.0, 90.0, 32.0),
                "Cancel",
                mouse,
                true,
                false,
            )
        {
            self.dialog = None;
            self.pending = None;
            return Action::None;
        }
        match dialog {
            Dialog::Unsaved(p) => {
                text("This map has unsaved changes.", x, y + 50.0, 22.0, WHITE);
                text(
                    "Save a copy, discard changes, or cancel to keep editing.",
                    x,
                    y + 84.0,
                    18.0,
                    MUTED,
                );
                if ui_button(
                    Rect::new(x, y + 144.0, 170.0, 40.0),
                    "Save & continue",
                    mouse,
                    true,
                    true,
                ) {
                    self.pending = Some(p);
                    return self.save_request(false);
                }
                if ui_button(
                    Rect::new(x + 190.0, y + 144.0, 170.0, 40.0),
                    "Discard changes",
                    mouse,
                    true,
                    false,
                ) {
                    return self.proceed(p);
                }
            }
            Dialog::Overwrite => {
                text(
                    "A map already exists at this location.",
                    x,
                    y + 50.0,
                    22.0,
                    WHITE,
                );
                if let Some(path) = &self.save_target {
                    text(
                        &shorten(&path.to_string_lossy(), 74),
                        x,
                        y + 86.0,
                        17.0,
                        MUTED,
                    );
                }
                if ui_button(
                    Rect::new(x, y + 144.0, 180.0, 40.0),
                    "Replace file",
                    mouse,
                    true,
                    true,
                ) {
                    if let Some(path) = self.save_target.take() {
                        return self.save(path);
                    }
                }
            }
            Dialog::New | Dialog::Resize => {
                text(
                    "Choose a preset or enter dimensions (5 to 1024 tiles).",
                    x,
                    y + 36.0,
                    18.0,
                    MUTED,
                );
                for (i, n) in [64, 128, 256, 512, 1024].iter().enumerate() {
                    if ui_button(
                        Rect::new(x + i as f32 * 128.0, y + 58.0, 116.0, 35.0),
                        &format!("{n} x {n}"),
                        mouse,
                        true,
                        false,
                    ) {
                        self.fields = [n.to_string(), n.to_string()];
                        self.cursor = self.fields[self.active_field].len();
                        self.select_all = true;
                    }
                }
                text("Width (tiles)", x, y + 135.0, 18.0, WHITE);
                text("Height (tiles)", x + 330.0, y + 135.0, 18.0, WHITE);
                self.field(0, Rect::new(x, y + 148.0, 310.0, 42.0), mouse);
                self.field(1, Rect::new(x + 330.0, y + 148.0, 310.0, 42.0), mouse);
                if is_key_pressed(KeyCode::Tab) {
                    self.active_field = 1 - self.active_field;
                    self.cursor = self.fields[self.active_field].len();
                    self.select_all = true;
                }
                self.type_field(true);
                text(
                    if matches!(dialog, Dialog::Resize) {
                        "Keeps the top-left area. Shrinking crops tiles; Undo restores them."
                    } else {
                        "Starts with grass, a tree border, a player and an enemy entry."
                    },
                    x,
                    y + 230.0,
                    18.0,
                    MUTED,
                );
                text(
                    "1024 x 1024 = 1,048,576 tiles. Zoom in to paint individual tiles.",
                    x,
                    y + 261.0,
                    18.0,
                    MUTED,
                );
                if ui_button(
                    Rect::new(x, y + 305.0, 180.0, 42.0),
                    if matches!(dialog, Dialog::New) {
                        "Create map"
                    } else {
                        "Apply resize"
                    },
                    mouse,
                    true,
                    true,
                ) || is_key_pressed(KeyCode::Enter)
                {
                    let dimensions = self.fields[0]
                        .parse::<usize>()
                        .ok()
                        .zip(self.fields[1].parse::<usize>().ok());
                    let result = if let Some((width, height)) = dimensions {
                        if matches!(dialog, Dialog::New) {
                            Grid::new(width, height).map(|grid| {
                                self.doc = Document::new(grid, false);
                                self.path = None;
                            })
                        } else {
                            self.doc.resize(width, height)
                        }
                    } else {
                        Err("Enter whole numbers between 5 and 1024.".into())
                    };
                    match result {
                        Ok(()) => {
                            self.dialog = None;
                            self.fit_requested = true;
                            self.overview = None;
                            self.status = "Canvas ready. Changes can be undone with Ctrl+Z.".into();
                        }
                        Err(e) => self.status = e,
                    }
                }
            }
            Dialog::Open | Dialog::Save => {
                text(
                    "File path (Ctrl+A selects all; Ctrl+V pastes)",
                    x,
                    y + 36.0,
                    18.0,
                    MUTED,
                );
                self.field(0, Rect::new(x, y + 50.0, 646.0, 42.0), mouse);
                self.type_field(false);
                if matches!(dialog, Dialog::Open) {
                    let entries = crate::catalog();
                    let pages = entries.len().div_ceil(6).max(1);
                    self.browser_page = self.browser_page.min(pages - 1);
                    text("Or choose an existing arena:", x, y + 122.0, 18.0, MUTED);
                    for (i, entry) in entries
                        .iter()
                        .skip(self.browser_page * 6)
                        .take(6)
                        .enumerate()
                    {
                        let label = format!(
                            "{}{}",
                            entry.name,
                            if entry.source.is_none() {
                                "  (built-in copy)"
                            } else {
                                ""
                            }
                        );
                        if ui_button(
                            Rect::new(x, y + 138.0 + i as f32 * 32.0, 646.0, 28.0),
                            &shorten(&label, 68),
                            mouse,
                            true,
                            false,
                        ) {
                            match Self::from_entry(entry) {
                                Ok(editor) => {
                                    *self = editor;
                                    return Action::None;
                                }
                                Err(e) => self.status = e,
                            }
                        }
                    }
                    if ui_button(
                        Rect::new(x, y + 339.0, 110.0, 30.0),
                        "Previous",
                        mouse,
                        true,
                        false,
                    ) {
                        self.browser_page = (self.browser_page + pages - 1) % pages;
                    }
                    text(
                        &format!("{} / {pages}", self.browser_page + 1),
                        x + 130.0,
                        y + 360.0,
                        18.0,
                        MUTED,
                    );
                    if ui_button(
                        Rect::new(x + 210.0, y + 339.0, 110.0, 30.0),
                        "Next",
                        mouse,
                        true,
                        false,
                    ) {
                        self.browser_page = (self.browser_page + 1) % pages;
                    }
                } else {
                    text(
                        "Use .arena or .mmp. Missing folders are created on save.",
                        x,
                        y + 135.0,
                        18.0,
                        MUTED,
                    );
                    text(
                        "Save in levels/ to include the map in the arena selector.",
                        x,
                        y + 167.0,
                        18.0,
                        MUTED,
                    );
                    text(
                        "Unfinished maps can be saved. Validate before playing.",
                        x,
                        y + 199.0,
                        18.0,
                        MUTED,
                    );
                }
                if ui_button(
                    Rect::new(x, r.bottom() - 52.0, 180.0, 32.0),
                    if matches!(dialog, Dialog::Open) {
                        "Open path"
                    } else {
                        "Save map"
                    },
                    mouse,
                    true,
                    true,
                ) || is_key_pressed(KeyCode::Enter)
                {
                    let mut path = PathBuf::from(self.fields[0].trim());
                    if path.as_os_str().is_empty() {
                        self.status = "Enter a file path.".into();
                        return Action::None;
                    }
                    if matches!(dialog, Dialog::Open) {
                        match std::fs::read_to_string(&path)
                            .map_err(|e| e.to_string())
                            .and_then(|s| Grid::parse(&s))
                        {
                            Ok(grid) => *self = Self::new(grid, Some(path)),
                            Err(e) => self.status = format!("Could not open: {e}"),
                        }
                    } else {
                        if path.extension().is_none() {
                            path.set_extension("arena");
                        }
                        if !path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
                            s.eq_ignore_ascii_case("arena") || s.eq_ignore_ascii_case("mmp")
                        }) {
                            self.status = "Use the .arena or .mmp file extension.".into();
                            return Action::None;
                        }
                        if path.exists() {
                            self.save_target = Some(path);
                            self.dialog = Some(Dialog::Overwrite);
                        } else {
                            return self.save(path);
                        }
                    }
                }
            }
        }
        // Keep errors visible above the dimmed editor footer.
        text(&shorten(&self.status, 85), x, r.bottom() - 72.0, 15.0, GOLD);
        Action::None
    }
    fn field(&mut self, index: usize, rect: Rect, mouse: Vec2) {
        if rect.contains(mouse) && is_mouse_button_pressed(MouseButton::Left) {
            self.active_field = index;
            self.cursor = self.fields[index].len();
            self.select_all = true;
        }
        let active = self.active_field == index;
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if active && self.select_all {
                Color::from_hex(0x21483f)
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
            if active { MINT } else { EDGE },
        );
        let value = &self.fields[index];
        let display = if active {
            format!("{}|{}", &value[..self.cursor], &value[self.cursor..])
        } else {
            value.clone()
        };
        let max = (rect.w / 10.0) as usize;
        let tail: String = display
            .chars()
            .rev()
            .take(max)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        text(&tail, rect.x + 10.0, rect.y + 28.0, 20.0, WHITE);
    }
    fn type_field(&mut self, numeric: bool) {
        let value = &mut self.fields[self.active_field];
        if ctrl() && is_key_pressed(KeyCode::A) {
            self.select_all = true;
        }
        if is_key_pressed(KeyCode::Left) {
            self.cursor = value[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(i, _)| i);
            self.select_all = false;
        }
        if is_key_pressed(KeyCode::Right) {
            self.cursor += value[self.cursor..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
            self.select_all = false;
        }
        if is_key_pressed(KeyCode::Home) {
            self.cursor = 0;
            self.select_all = false;
        }
        if is_key_pressed(KeyCode::End) {
            self.cursor = value.len();
            self.select_all = false;
        }
        if is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Delete) {
            if self.select_all {
                value.clear();
                self.cursor = 0;
                self.select_all = false;
            } else if is_key_pressed(KeyCode::Backspace) && self.cursor > 0 {
                let previous = value[..self.cursor].char_indices().next_back().unwrap().0;
                value.drain(previous..self.cursor);
                self.cursor = previous;
            } else if is_key_pressed(KeyCode::Delete) && self.cursor < value.len() {
                value.remove(self.cursor);
            }
        }
        let mut inserted = String::new();
        if ctrl() && is_key_pressed(KeyCode::V) {
            inserted = macroquad::miniquad::window::clipboard_get().unwrap_or_default();
        }
        while let Some(c) = get_char_pressed() {
            if !ctrl() {
                inserted.push(c);
            }
        }
        inserted.retain(|c| !c.is_control() && (!numeric || c.is_ascii_digit()));
        if !inserted.is_empty() {
            if self.select_all {
                value.clear();
                self.cursor = 0;
                self.select_all = false;
            }
            if value.len() + inserted.len() <= 2048 {
                value.insert_str(self.cursor, &inserted);
                self.cursor += inserted.len();
            }
        }
    }
}
fn ctrl() -> bool {
    is_key_down(KeyCode::LeftControl)
        || is_key_down(KeyCode::RightControl)
        || is_key_down(KeyCode::LeftSuper)
        || is_key_down(KeyCode::RightSuper)
}

fn shift() -> bool {
    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)
}
fn shorten(s: &str, limit: usize) -> String {
    if s.chars().count() > limit {
        format!(
            "{}...",
            s.chars().take(limit.saturating_sub(3)).collect::<String>()
        )
    } else {
        s.into()
    }
}
fn ui_button(r: Rect, label: &str, mouse: Vec2, enabled: bool, selected: bool) -> bool {
    let hover = enabled && r.contains(mouse);
    draw_rectangle(
        r.x,
        r.y,
        r.w,
        r.h,
        if selected {
            Color::from_hex(0x21483f)
        } else if hover {
            Color::from_hex(0x253b46)
        } else {
            PANEL
        },
    );
    draw_rectangle_lines(
        r.x,
        r.y,
        r.w,
        r.h,
        1.0,
        if selected || hover { MINT } else { EDGE },
    );
    text(
        label,
        r.x + 9.0,
        r.y + r.h * 0.5 + 6.0,
        17.0,
        if !enabled {
            MUTED
        } else if selected {
            MINT
        } else {
            WHITE
        },
    );
    hover && is_mouse_button_pressed(MouseButton::Left)
}
fn tile_color(c: u8) -> Color {
    Color::from_hex(match c {
        b'#' | b'T' | b'4'..=b'6' => 0x244737,
        b'~' => 0x397c98,
        b'B' => 0x5d7839,
        b'C' => 0xb68a51,
        b'+' | b'3' => 0x69d8bf,
        b'P' | b'1' => 0xf1ce6b,
        b'E' | b'2' => 0xd57489,
        _ => 0x63815a,
    })
}
fn swatch(tile: u8, r: Rect) {
    draw_rectangle(r.x, r.y, r.w, r.h, tile_color(tile));
    let c = r.center();
    let s = r.w.min(r.h);
    match tile {
        b'#' | b'T' | b'4'..=b'6' => {
            draw_circle(c.x, c.y, s * 0.34, Color::from_hex(0x3a654b));
            draw_circle(
                c.x - s * 0.12,
                c.y - s * 0.10,
                s * 0.16,
                Color::from_hex(0x53825b),
            );
        }
        b'C' => {
            draw_rectangle_lines(
                r.x + s * 0.15,
                r.y + s * 0.15,
                r.w - s * 0.3,
                r.h - s * 0.3,
                1.0,
                Color::from_hex(0x765330),
            );
            draw_line(
                r.x + s * 0.15,
                r.y + s * 0.15,
                r.right() - s * 0.15,
                r.bottom() - s * 0.15,
                1.0,
                Color::from_hex(0x765330),
            );
        }
        b'~' => {
            for i in [0.35, 0.65] {
                draw_line(
                    r.x + s * 0.2,
                    r.y + s * i,
                    r.right() - s * 0.2,
                    r.y + s * i,
                    1.0,
                    Color::from_hex(0x73b1c2),
                );
            }
        }
        b'B' => {
            draw_circle(c.x, c.y, s * 0.27, Color::from_hex(0x88a35e));
        }
        b'+' | b'3' => {
            draw_line(
                c.x - s * 0.25,
                c.y,
                c.x + s * 0.25,
                c.y,
                (s * 0.14).max(1.0),
                WHITE,
            );
            draw_line(
                c.x,
                c.y - s * 0.25,
                c.x,
                c.y + s * 0.25,
                (s * 0.14).max(1.0),
                WHITE,
            );
        }
        b'P' | b'1' | b'E' | b'2' => {
            draw_circle(c.x, c.y, s * 0.31, INK);
            text(
                if matches!(tile, b'P' | b'1') {
                    "P"
                } else {
                    "E"
                },
                c.x - s * 0.16,
                c.y + s * 0.22,
                s * 0.63,
                WHITE,
            );
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_zoom_preserves_tile_and_panel_clicks_never_paint() {
        let mut editor = Editor::new(Grid::new(1024, 512).unwrap(), None);
        let view = Rect::new(250.0, 112.0, 1014.0, 542.0);
        editor.fit(view);
        let mouse = view.center() + vec2(40.0, 20.0);
        let before = editor.world(mouse, view);
        let cell = editor.cell(mouse, view);
        for _ in 0..8 {
            editor.zoom_at(mouse, view, 1.0);
        }
        assert!(editor.world(mouse, view).distance(before) < 0.001);
        assert_eq!(editor.cell(mouse, view), cell);
        assert_eq!(editor.cell(vec2(80.0, 200.0), view), None);
        editor.center = Vec2::ZERO;
        assert_eq!(editor.cell(vec2(view.x + 1.0, view.y + 1.0), view), None);
    }
}
