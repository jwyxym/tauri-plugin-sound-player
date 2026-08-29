const COMMANDS: &[&str] = &[
    "play_once",
    "play_loop",
    "stop",
    "stop_all",
    "set_volume",
    "pause",
    "resume",
    "is_finished",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
