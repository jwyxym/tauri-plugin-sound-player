# tauri-plugin-sound-player

A Tauri v2 plugin for cross-platform sound playback powered by `rodio`.

It supports one-shot playback, loop playback, live volume changes, pause/resume/stop, concurrent playback through a shared mixer, and mp3/ogg/wav files.

## Install

Add the Rust plugin crate to your Tauri app:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-sound-player = { path = "../tauri-plugin-sound-player" }
```

Register the plugin:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_sound_player::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

Allow the commands in your Tauri capability file:

```json
{
  "permissions": ["sound-player:default"]
}
```

## Frontend

The frontend API follows the same layout as `tauri-plugin-dglab-ws-server`:

- source: `webview-src/index.ts`
- build output: `webview-dist/index.js`
- types: `webview-dist/index.d.ts`

Use it from the host app:

```ts
import {
  playLoop,
  playOnce,
  setVolume,
  stop,
  stopAll,
  pause,
  resume,
  isFinished,
} from 'tauri-plugin-sound-player';

const bgm = await playLoop({
  path: 'C:/sounds/bgm.ogg',
  volume: 0.6,
});

await playOnce({
  path: 'C:/sounds/click.wav',
  volume: 1.0,
});

await setVolume(bgm, 0.3);
await pause(bgm);
await resume(bgm);

const finished = await isFinished(bgm);

await stop(bgm);
await stopAll();
```

You can also call commands directly:

```ts
import { invoke } from '@tauri-apps/api/core';

const id = await invoke<number>('plugin:sound-player|play_loop', {
  path: 'C:/sounds/bgm.mp3',
  volume: 0.5,
});

await invoke('plugin:sound-player|set_volume', { id, volume: 0.25 });
await invoke('plugin:sound-player|stop', { id });
```

## Commands

| Command | Arguments | Returns |
| --- | --- | --- |
| `play_once` | `path: string`, `volume?: number` | `number` sound id |
| `play_loop` | `path: string`, `volume?: number` | `number` sound id |
| `set_volume` | `id: number`, `volume: number` | `void` |
| `pause` | `id: number` | `void` |
| `resume` | `id: number` | `void` |
| `stop` | `id: number` | `void` |
| `stop_all` | none | `void` |
| `is_finished` | `id: number` | `boolean` |