use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

mod commands;
pub mod error;
mod output;
mod player;

pub use error::{Error, Result};
pub use player::{PlayMode, SoundHandle, SoundId, SoundPlayer, SoundPlayerState};

const PLUGIN_NAME: &str = "sound-player";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![
            commands::play_once,
            commands::play_loop,
            commands::stop,
            commands::stop_all,
            commands::set_volume,
            commands::pause,
            commands::resume,
            commands::is_finished,
        ])
        .setup(|app, _api| {
            app.manage(SoundPlayerState::new()?);
            Ok(())
        })
        .build()
}
