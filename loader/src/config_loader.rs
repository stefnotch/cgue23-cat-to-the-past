use serde::{Deserialize, Serialize};
use std::path::Path;

/// A config that can be loaded from a file.
/// Split into a single separate type, because serde makes compile times annoyingly long.
#[derive(Debug, Deserialize, Serialize)]
pub struct LoadableConfig {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub refresh_rate: u32,
    pub brightness: f32,
    pub mouse_sensitivity: f32,
}

impl LoadableConfig {
    pub fn load<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let config = match std::fs::File::open(path) {
            Ok(file) => serde_json::from_reader(file).unwrap(),
            Err(err) => {
                if path.exists() {
                    panic!("Failed to open {:?}: {}", path, err);
                }

                let config = Self::default();
                let config_file = std::fs::File::create(path)
                    .unwrap_or_else(|_| panic!("Failed to create {:?}", path));
                serde_json::to_writer_pretty(config_file, &config)
                    .unwrap_or_else(|_| panic!("Failed to write to {:?}", path));
                config
            }
        };

        config
    }
}

impl Default for LoadableConfig {
    fn default() -> Self {
        Self {
            resolution: (1280, 720),
            fullscreen: false,
            refresh_rate: 60,
            brightness: 1.0,
            mouse_sensitivity: 1.0,
        }
    }
}
