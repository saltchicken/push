// ‼️ Import new modules and types
mod soundboard_modules;
use crate::soundboard_modules::audio_player::PlaybackSink;
use log::{debug, info};
use push2::{ControlName, EncoderName, PadCoord, Push2, Push2Colors, Push2Event};
use soundboard_modules::{AudioCommand, audio_capture, audio_player, audio_processor};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{error, time};
use tokio::fs as tokio_fs; // ‼️ For async file deletion

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Playback,
    Edit,
}

// ‼️ AppState is now a combination of both projects
struct AppState {
    mode: Mode,
    pad_files: HashMap<u8, PathBuf>,
    playback_sink: PlaybackSink,
    playback_volume: HashMap<u8, f64>,
    pitch_shift_semitones: HashMap<u8, f64>,
    active_recording_key: Option<u8>,
    selected_for_edit: Option<u8>,
    audio_cmd_tx: mpsc::Sender<AudioCommand>,
}

// --- Color Constants for different states ---
const COLOR_OFF: u8 = Push2Colors::BLACK;
const COLOR_HAS_FILE: u8 = Push2Colors::GREEN_PALE;
const COLOR_RECORDING: u8 = Push2Colors::RED;
const COLOR_PLAYING: u8 = Push2Colors::AMBER;
const COLOR_SELECTED: u8 = Push2Colors::WHITE;

const BUTTON_LIGHT_ON: u8 = Push2Colors::GREEN_PALE;

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

// ‼️ Main function is now `async` and uses `tokio::main`
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::init();

    // ‼️ --- Spawn Audio Capture Thread ---
    // ‼️ This thread will block on the pipewire mainloop, which is perfect.
    let (audio_tx, audio_rx) = mpsc::channel();
    std::thread::spawn(move || {
        println!("Audio capture thread started...");
        if let Err(e) = audio_capture::run_capture_loop(audio_rx) {
            eprintln!("Audio capture thread failed: {}", e);
        } else {
            println!("Audio capture thread exited cleanly.");
        }
    });

    // --- Config Loading ---
    let mut push2 = Push2::new()?;
    let audio_storage_path = get_audio_storage_path()?;
    println!("Audio storage path: {}", audio_storage_path.display());

    // ‼️ --- Initialize Full AppState ---
    let mut app_state = AppState {
        mode: Mode::Playback,
        pad_files: HashMap::new(),
        playback_sink: PlaybackSink::Default,
        playback_volume: HashMap::new(),
        pitch_shift_semitones: HashMap::new(),
        active_recording_key: None,
        selected_for_edit: None,
        audio_cmd_tx: audio_tx,
    };

    info!("\nConnection open. Soundboard example running.");
    info!(
        "Mode: {:?} | Sink: {:?}",
        app_state.mode, app_state.playback_sink
    );

    // ‼️ --- Initialize Pads ---
    for y in 0..8 {
        for x in 0..8 {
            let coord = PadCoord { x, y };
            let mut color = COLOR_OFF;

            if let Some(address) = push2.button_map.get_note_address(coord) {
                // ‼️ Assign a file path to every pad
                let file_name = format!("pad_{}_{}.wav", x, y);
                let file_path = audio_storage_path.join(file_name);

                // ‼️ Check if the file *actually* exists
                if file_path.exists() {
                    color = COLOR_HAS_FILE;
                }
                app_state.pad_files.insert(address, file_path);
            }
            push2.set_pad_color(coord, color)?;
        }
    }

    // --- Main Loop ---
    loop {
        while let Some(event) = push2.poll_event() {
            debug!("Received event: {:?}", event);
            match event {
                // ‼️ --- PAD PRESSED ---
                Push2Event::PadPressed { coord, .. } => {
                    let Some(address) = push2.button_map.get_note_address(coord) else {
                        continue;
                    };
                    let Some(path) = app_state.pad_files.get(&address) else {
                        continue;
                    };

                    match app_state.mode {
                        Mode::Playback => {
                            if path.exists() {
                                // ‼️ File exists: Set color to "playing"
                                push2.set_pad_color(coord, COLOR_PLAYING)?;
                            } else {
                                // ‼️ No file: Start recording
                                info!("START recording to {}", path.display());
                                let cmd = AudioCommand::Start(path.clone());
                                if let Err(e) = app_state.audio_cmd_tx.send(cmd) {
                                    eprintln!("Failed to send START command: {}", e);
                                } else {
                                    app_state.active_recording_key = Some(address);
                                    push2.set_pad_color(coord, COLOR_RECORDING)?;
                                }
                            }
                        }
                        Mode::Edit => {
                            if !path.exists() {
                                continue;
                            } // ‼️ Can't edit a non-existent file

                            if let Some(prev_selected_key) = app_state.selected_for_edit {
                                if prev_selected_key == address {
                                    // ‼️ Deselecting current pad
                                    app_state.selected_for_edit = None;
                                    push2.set_pad_color(coord, COLOR_HAS_FILE)?;
                                } else {
                                    // ‼️ Deselect old pad
                                    if let Some(old_coord) =
                                        push2.button_map.get_note(prev_selected_key)
                                    {
                                        push2.set_pad_color(old_coord, COLOR_HAS_FILE)?;
                                    }
                                    // ‼️ Select new pad
                                    app_state.selected_for_edit = Some(address);
                                    push2.set_pad_color(coord, COLOR_SELECTED)?;
                                }
                            } else {
                                // ‼️ Nothing selected, select this pad
                                app_state.selected_for_edit = Some(address);
                                push2.set_pad_color(coord, COLOR_SELECTED)?;
                            }
                        }
                    }
                }

                // ‼️ --- PAD RELEASED ---
                Push2Event::PadReleased { coord } => {
                    let Some(address) = push2.button_map.get_note_address(coord) else {
                        continue;
                    };
                    let Some(path) = app_state.pad_files.get(&address) else {
                        continue;
                    };

                    match app_state.mode {
                        Mode::Playback => {
                            if app_state.active_recording_key == Some(address) {
                                // ‼️ --- Stop Recording ---
                                info!("STOP recording.");
                                if let Err(e) = app_state.audio_cmd_tx.send(AudioCommand::Stop) {
                                    eprintln!("Failed to send STOP command: {}", e);
                                }
                                app_state.active_recording_key = None;
                                // ‼️ Set color to "has_file" (it should exist now)
                                push2.set_pad_color(coord, COLOR_HAS_FILE)?;
                            } else if path.exists() {
                                // ‼️ --- Trigger Playback ---
                                info!("Triggering playback for pad ({}, {}).", coord.x, coord.y);
                                let pitch_shift = app_state
                                    .pitch_shift_semitones
                                    .get(&address)
                                    .cloned()
                                    .unwrap_or(0.0);
                                let path_clone = path.clone();
                                let sink_clone = app_state.playback_sink;
                                let volume_clone = app_state
                                    .playback_volume
                                    .get(&address)
                                    .cloned()
                                    .unwrap_or(1.0);

                                // ‼️ Spawn a new async task to handle playback
                                tokio::spawn(async move {
                                    let mut temp_path: Option<PathBuf> = None;
                                    let path_to_play = if pitch_shift.abs() > 0.01 {
                                        let path_for_blocking = path_clone.clone();
                                        match tokio::task::spawn_blocking(move || {
                                            audio_processor::create_pitched_copy_sync(
                                                &path_for_blocking,
                                                pitch_shift,
                                            )
                                        })
                                        .await
                                        {
                                            Ok(Ok(new_path)) => {
                                                temp_path = Some(new_path.clone());
                                                new_path
                                            }
                                            Ok(Err(e)) => {
                                                eprintln!(
                                                    "Failed to create pitched copy: {}. Playing original.",
                                                    e
                                                );
                                                path_clone
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "Task join error: {}. Playing original.",
                                                    e
                                                );
                                                path_clone
                                            }
                                        }
                                    } else {
                                        path_clone
                                    };

                                    if let Err(e) = audio_player::play_audio_file(
                                        &path_to_play,
                                        sink_clone,
                                        volume_clone,
                                    )
                                    .await
                                    {
                                        eprintln!("Playback failed: {}", e);
                                    }

                                    if let Some(p) = temp_path {
                                        if let Err(e) = tokio_fs::remove_file(&p).await {
                                            eprintln!(
                                                "Failed to clean up temp file {}: {}",
                                                p.display(),
                                                e
                                            );
                                        }
                                    }
                                });
                                // ‼️ Set color back to "has_file"
                                push2.set_pad_color(coord, COLOR_HAS_FILE)?;
                            } else {
                                // ‼️ Released a pad that has no file and wasn't recording
                                push2.set_pad_color(coord, COLOR_OFF)?;
                            }
                        }
                        Mode::Edit => {
                            // ‼️ In edit mode, releasing a pad does nothing.
                            // ‼️ It stays selected or deselected.
                        }
                    }
                }

                // ‼️ --- BUTTON PRESSED (for Mode controls) ---
                Push2Event::ButtonPressed { name, .. } => {
                    match name {
                        // ‼️ Map Master button to cycling the playback sink
                        ControlName::Master => {
                            app_state.playback_sink = match app_state.playback_sink {
                                PlaybackSink::Default => PlaybackSink::Mixer,
                                PlaybackSink::Mixer => PlaybackSink::Both,
                                PlaybackSink::Both => PlaybackSink::Default,
                            };
                            info!("Playback sink set to: {:?}", app_state.playback_sink);
                        }
                        // ‼️ Map Delete button
                        ControlName::Delete => {
                            if app_state.mode == Mode::Edit {
                                if let Some(key_to_delete) = app_state.selected_for_edit.take() {
                                    info!("DELETE button pressed. Deleting selected sample.");
                                    if let (Some(path), Some(coord)) = (
                                        app_state.pad_files.get(&key_to_delete),
                                        push2.button_map.get_note(key_to_delete),
                                    ) {
                                        match tokio_fs::remove_file(path).await {
                                            Ok(_) => {
                                                info!("...File {} deleted.", path.display());
                                                app_state
                                                    .pitch_shift_semitones
                                                    .remove(&key_to_delete);
                                                app_state.playback_volume.remove(&key_to_delete);
                                                push2.set_pad_color(coord, COLOR_OFF)?;
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "...Failed to delete file {}: {}",
                                                    path.display(),
                                                    e
                                                );
                                                // ‼️ Set back to "has file" color if delete failed
                                                push2.set_pad_color(coord, COLOR_HAS_FILE)?;
                                            }
                                        }
                                    }
                                } else {
                                    info!("DELETE pressed in Edit mode, but no sample selected.");
                                }
                            }
                        }
                        _ => {
                            debug!("--- Button {:?} PRESSED ---", name);
                            push2.set_button_light(name, BUTTON_LIGHT_ON)?;
                        }
                    }
                }
                Push2Event::ButtonReleased { name } => {
                    debug!("--- Button {:?} RELEASED ---", name);
                    // ‼️ Don't turn off Delete button if we are in Edit mode
                    if !(name == ControlName::Delete && app_state.mode == Mode::Edit) {
                        push2.set_button_light(name, 0)?;
                    }
                }

                // ‼️ --- ENCODER TWISTED (for Mode/Param controls) ---
                Push2Event::EncoderTwisted {
                    name, raw_delta, ..
                } => {
                    let delta = if raw_delta > 64 {
                        -((128 - raw_delta) as i32)
                    } else {
                        raw_delta as i32
                    };

                    match name {
                        // ‼️ Map Tempo knob to Mode switch
                        EncoderName::Tempo => {
                            app_state.mode = match app_state.mode {
                                Mode::Playback => Mode::Edit,
                                Mode::Edit => Mode::Playback,
                            };
                            info!("Mode switched to: {:?}", app_state.mode);

                            // ‼️ If switching away from Edit, deselect pad
                            if app_state.mode == Mode::Playback {
                                if let Some(selected_key) = app_state.selected_for_edit.take() {
                                    if let Some(coord) = push2.button_map.get_note(selected_key) {
                                        push2.set_pad_color(coord, COLOR_HAS_FILE)?;
                                    }
                                }
                                push2.set_button_light(ControlName::Delete, 0)?;
                            } else {
                                // ‼️ Switched *to* Edit mode
                                push2
                                    .set_button_light(ControlName::Delete, Push2Colors::RED_LED)?;
                            }
                        }
                        // ‼️ Map Track1 knob to Volume
                        EncoderName::Track1 => {
                            if app_state.mode == Mode::Edit {
                                if let Some(key) = app_state.selected_for_edit {
                                    let current_volume =
                                        app_state.playback_volume.entry(key).or_insert(1.0);
                                    *current_volume += delta as f64 * 0.01; // 1% per tick
                                    *current_volume = current_volume.clamp(0.0, 1.5); // 0% to 150%
                                    info!(
                                        "Set volume for selected pad to {:.0}%",
                                        *current_volume * 100.0
                                    );
                                }
                            }
                        }
                        // ‼️ Map Track2 knob to Pitch
                        EncoderName::Track2 => {
                            if app_state.mode == Mode::Edit {
                                if let Some(key) = app_state.selected_for_edit {
                                    let current_pitch =
                                        app_state.pitch_shift_semitones.entry(key).or_insert(0.0);
                                    *current_pitch += delta as f64 * 0.1; // 0.1 semitones per tick
                                    info!(
                                        "Set pitch for selected pad to {:.2} semitones",
                                        *current_pitch
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // ‼️ Yield control back to the tokio runtime
        tokio::time::sleep(time::Duration::from_millis(1000 / 60)).await;
    }
}
