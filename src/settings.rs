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
    pub sound_volume: u8,
    pub music_volume: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: 0,
            fullscreen: false,
            effects: true,
            sound: true,
            sound_volume: 90,
            music_volume: 70,
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
        Self::from_text(&std::fs::read_to_string(Self::path()).unwrap_or_default())
    }
    fn from_text(text: &str) -> Self {
        let mut s = Self::default();
        for line in text.lines() {
            if let Some((key, val)) = line.split_once('=') {
                match key {
                    "resolution" => s.resolution = val.parse::<usize>().unwrap_or(0).min(4),
                    "fullscreen" => s.fullscreen = val == "true",
                    "effects" => s.effects = val == "true",
                    "sound" => s.sound = val == "true",
                    "sound_volume" => s.sound_volume = parse_volume(val, 90),
                    "music_volume" => s.music_volume = parse_volume(val, 70),
                    _ => {}
                }
            }
        }
        s
    }
    fn to_text(&self) -> String {
        format!(
            "resolution={}\nfullscreen={}\neffects={}\nsound={}\nsound_volume={}\nmusic_volume={}\n",
            self.resolution, self.fullscreen, self.effects, self.sound,
            self.sound_volume, self.music_volume
        )
    }
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(path, self.to_text()).map_err(|e| e.to_string())
    }
}

fn parse_volume(value: &str, default: i32) -> u8 {
    value.parse::<i32>().unwrap_or(default).clamp(0, 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_settings_use_default_volumes_and_keep_mute() {
        let settings = Settings::from_text("sound=false\nresolution=2\n");
        assert!(!settings.sound);
        assert_eq!((settings.sound_volume, settings.music_volume), (90, 70));
    }

    #[test]
    fn independent_volumes_round_trip_and_clamp() {
        let settings = Settings::from_text("sound_volume=0\nmusic_volume=65\n");
        let saved = Settings::from_text(&settings.to_text());
        assert_eq!((saved.sound_volume, saved.music_volume), (0, 65));
        let invalid = Settings::from_text("sound_volume=-20\nmusic_volume=150\n");
        assert_eq!((invalid.sound_volume, invalid.music_volume), (0, 100));
        assert_eq!(Settings::from_text("music_volume=invalid").music_volume, 70);
    }
}
