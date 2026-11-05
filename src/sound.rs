use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};
use mgame::simulation::SoundEvent;
use std::{io::Cursor, path::PathBuf};

const MUSIC_INTERVAL: f64 = 20.0;

struct Track {
    sound: Sound,
    duration: f64,
}

#[derive(Default)]
struct Music {
    tracks: Vec<Track>,
    current: Option<usize>,
    last: Option<usize>,
    next_at: f64,
}

impl Music {
    async fn load() -> Self {
        let mut music = Self::default();
        let mut roots = vec![PathBuf::from("Music")];
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                roots.push(parent.join("Music"));
            }
        }
        let mut files = std::collections::BTreeMap::new();
        for root in roots {
            if let Ok(entries) = std::fs::read_dir(root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
                        s.eq_ignore_ascii_case("ogg") || s.eq_ignore_ascii_case("wav")
                    }) {
                        files
                            .entry(entry.file_name().to_string_lossy().to_lowercase())
                            .or_insert(path);
                    }
                }
            }
        }
        for path in files.into_values() {
            let result = async {
                let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                let duration = music_duration(&bytes)?;
                let sound = load_sound_from_bytes(&bytes)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok::<_, String>(Track { sound, duration })
            }
            .await;
            match result {
                Ok(track) => music.tracks.push(track),
                Err(e) => eprintln!("Skipping music {}: {e}", path.display()),
            }
        }
        music
    }

    fn update(&mut self, running: bool, now: f64, volume: f32) {
        if !running {
            if let Some(i) = self.current.take() {
                stop_sound(&self.tracks[i].sound);
            }
            self.next_at = 0.0;
            return;
        }
        if let Some(i) = self.current {
            set_sound_volume(&self.tracks[i].sound, volume);
        }
        if self.tracks.is_empty() || now < self.next_at {
            return;
        }
        // Avoid consecutive repeats when more than one song is available.
        let choices = self.tracks.len() - usize::from(self.last.is_some() && self.tracks.len() > 1);
        let mut i = macroquad::rand::gen_range(0, choices);
        if self.tracks.len() > 1 && self.last.is_some_and(|last| i >= last) {
            i += 1;
        }
        if let Some(previous) = self.current.replace(i) {
            stop_sound(&self.tracks[previous].sound);
        }
        play_sound(
            &self.tracks[i].sound,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
        self.last = Some(i);
        self.next_at = now + self.tracks[i].duration + MUSIC_INTERVAL;
    }
}

// Macroquad does not expose playback completion. Measure decoded samples using
// the same decoder as its audio backend, and schedule against the real clock.
fn music_duration(bytes: &[u8]) -> Result<f64, String> {
    let mut reader = audrey::Reader::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let description = reader.description();
    if !matches!(description.channel_count(), 1 | 2) || description.sample_rate() == 0 {
        return Err("music must have one or two channels and a nonzero sample rate".into());
    }
    let samples = reader.samples::<f32>().try_fold(0u64, |count, sample| {
        sample.map(|_| count + 1).map_err(|e| e.to_string())
    })?;
    if samples == 0 {
        return Err("empty music file".into());
    }
    Ok(samples as f64 / description.channel_count() as f64 / description.sample_rate() as f64)
}

pub struct Audio {
    sounds: Vec<Option<Sound>>,
    music: Music,
}
impl Audio {
    pub async fn new() -> Self {
        let mut sounds = vec![];
        for (frequency, duration, noise) in [
            (160.0, 0.09, 0.65),
            (80.0, 0.20, 0.85),
            (900.0, 0.22, 0.22),
            (500.0, 0.045, 0.6),
            (100.0, 0.18, 0.5),
            (400.0, 0.16, 0.3),
            (620.0, 0.32, 0.0),
            (55.0, 0.55, 0.85),
            (1100.0, 0.3, 0.0),
            (180.0, 1.1, 0.12),
        ] {
            let bytes = wav(frequency, duration, noise);
            sounds.push(load_sound_from_bytes(&bytes).await.ok());
        }
        macroquad::rand::srand(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
        Self {
            sounds,
            music: Music::load().await,
        }
    }
    pub fn update_music(&mut self, running: bool, volume: u8) {
        self.music.update(
            running,
            macroquad::time::get_time(),
            0.3 * f32::from(volume.min(100)) / 100.0,
        );
    }
    pub fn play(&self, events: &[SoundEvent], enabled: bool, volume: u8) {
        if !enabled || volume == 0 {
            return;
        }
        let mut played = [false; 10];
        for event in events {
            let i = match event {
                SoundEvent::Rifle => 0,
                SoundEvent::Shotgun => 1,
                SoundEvent::Rail => 2,
                SoundEvent::Hit => 3,
                SoundEvent::Hurt => 4,
                SoundEvent::Dash => 5,
                SoundEvent::Wave => 6,
                SoundEvent::Explosion => 7,
                SoundEvent::Pickup => 8,
                SoundEvent::Boss => 9,
            };
            if played[i] {
                continue;
            }
            played[i] = true;
            if let Some(sound) = &self.sounds[i] {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: false,
                        volume: (if i == 3 { 0.14 } else { 0.25 }) * f32::from(volume.min(100))
                            / 100.0,
                    },
                );
            }
        }
    }
}
fn wav(frequency: f32, duration: f32, noise: f32) -> Vec<u8> {
    let rate = 22050u32;
    let count = (duration * rate as f32) as u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + count * 2).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * 2).to_le_bytes());
    bytes.extend(2u16.to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend((count * 2).to_le_bytes());
    let mut seed = 12345u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let envelope = (1.0 - t / duration).max(0.0).powi(2);
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let n = (seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let tone = (std::f32::consts::TAU * frequency * (t - t * t / (duration * 2.5))).sin();
        let sample = ((tone * (1.0 - noise) + n * noise) * envelope * 24000.0) as i16;
        bytes.extend(sample.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_uses_decoded_sample_count() {
        assert!((music_duration(&wav(440.0, 1.5, 0.0)).unwrap() - 1.5).abs() < 0.0001);
        assert!(music_duration(b"not audio").is_err());
        assert!(music_duration(&wav(440.0, 0.0, 0.0)).is_err());
    }

    #[test]
    fn supplied_music_decodes() {
        for name in ["Alien", "Dark", "Nothing", "Symphony"] {
            let bytes = std::fs::read(format!("Music/{name}.ogg")).unwrap();
            assert!(music_duration(&bytes).unwrap() > 0.0, "{name}");
        }
    }
}
