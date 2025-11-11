// ‼️ This file contains the shared definitions from your soundboard's src/lib.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub enum AudioCommand {
    Start(PathBuf),
    Stop,
}

// ‼️ Note: This function is duplicated from the example,
// ‼️ but in a real app you'd share this.
pub fn get_audio_storage_path() -> std::io::Result<PathBuf> {
    match dirs::audio_dir() {
        Some(mut path) => {
            path.push("soundboard-recordings");
            std::fs::create_dir_all(&path)?;
            Ok(path)
        }
        None => Err(std::io::Error::other("Could not find audio directory")),
    }
}

// ‼️ We also declare the other files in this module
pub mod audio_capture;
pub mod audio_player;
pub mod audio_processor;
