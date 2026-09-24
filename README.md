# tauri-plugin-sound-player

A Tauri v2 plugin for cross-platform sound playback powered by `rodio`.

It supports one-shot playback, loop playback, live volume changes, pause/resume/stop, concurrent playback through a shared mixer, and mp3/ogg/wav files.

Playback automatically follows changes to the system default output device. While sounds are
playing, the plugin checks every 500 ms and reconnects the existing mixer, preserving sound IDs, playback position,
volume, loop mode, and pause state. Switching may cause a brief audible gap; audio already
buffered by the old device cannot be recovered. If no output is available, playback waits
for a device to return. Failed reconnections are retried automatically. An output device is
still required when the plugin initializes. No frontend API changes are required, and explicit
selection of a non-default device is not currently exposed.

When all sounds have finished, stopped, or paused, periodic checks inspect only in-memory
player state and skip system device queries. Device checks resume within the next 500 ms
after starting or resuming playback; an output changed while idle may therefore briefly
use the previous device. The audio stream remains open while idle.

## Install

Add the Rust plugin crate to your Tauri app:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-sound-player = "0.1.1"
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
