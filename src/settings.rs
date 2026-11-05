use std::path::PathBuf;

pub const RESOLUTIONS: [(u32, u32); 5] = [
    (1280, 720),
    (1600, 900),
    (1920, 1080),
    (2560, 1440),
    (3840, 2160),
];
pub struct Settings {
    pub resolution: usize,
    pub fullscreen: bool,
    pub effects: bool,
    pub sound: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: 0,
            fullscreen: false,
            effects: true,
            sound: true,
        }
    }
}
impl Settings {
    fn path() -> PathBuf {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
            .unwrap_or_else(std::env::temp_dir);
        base.join("MGame").join("settings.cfg")
    }
    pub fn load() -> Self {
        let mut s = Self::default();
        if let Ok(text) = std::fs::read_to_string(Self::path()) {
            for line in text.lines() {
                if let Some((key, val)) = line.split_once('=') {
                    match key {
                        "resolution" => s.resolution = val.parse::<usize>().unwrap_or(0).min(4),
                        "fullscreen" => s.fullscreen = val == "true",
                        "effects" => s.effects = val == "true",
                        "sound" => s.sound = val == "true",
                        _ => {}
                    }
                }
            }
        }
        s
    }
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(
            path,
            format!(
                "resolution={}\nfullscreen={}\neffects={}\nsound={}\n",
                self.resolution, self.fullscreen, self.effects, self.sound
            ),
        )
        .map_err(|e| e.to_string())
    }
}
