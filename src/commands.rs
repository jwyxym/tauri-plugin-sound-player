use tauri::State;

use crate::{Result, SoundId, SoundPlayerState};

#[tauri::command]
pub fn play_once(
    state: State<'_, SoundPlayerState>,
    path: String,
    volume: Option<f32>,
) -> Result<u64> {
    state
        .play_once(path, volume.unwrap_or(1.0))
        .map(SoundId::get)
}

#[tauri::command]
pub fn play_loop(
    state: State<'_, SoundPlayerState>,
    path: String,
    volume: Option<f32>,
) -> Result<u64> {
    state
        .play_loop(path, volume.unwrap_or(1.0))
        .map(SoundId::get)
}

#[tauri::command]
pub fn stop(state: State<'_, SoundPlayerState>, id: u64) {
    state.stop(id.into());
}

#[tauri::command]
pub fn stop_all(state: State<'_, SoundPlayerState>) {
    state.stop_all();
}

#[tauri::command]
pub fn set_volume(state: State<'_, SoundPlayerState>, id: u64, volume: f32) {
    state.set_volume(id.into(), volume);
}

#[tauri::command]
pub fn pause(state: State<'_, SoundPlayerState>, id: u64) {
    state.pause(id.into());
}

#[tauri::command]
pub fn resume(state: State<'_, SoundPlayerState>, id: u64) {
    state.resume(id.into());
}

#[tauri::command]
pub fn is_finished(state: State<'_, SoundPlayerState>, id: u64) -> bool {
    state.is_finished(id.into())
}
