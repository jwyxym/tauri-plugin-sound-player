import { invoke } from '@tauri-apps/api/core';

export type SoundId = number;

export interface PlayOptions {
  path: string;
  volume?: number;
}

export async function playOnce(options: PlayOptions): Promise<SoundId> {
  return await invoke<SoundId>('plugin:sound-player|play_once', {
    path: options.path,
    volume: options.volume,
  });
}

export async function playLoop(options: PlayOptions): Promise<SoundId> {
  return await invoke<SoundId>('plugin:sound-player|play_loop', {
    path: options.path,
    volume: options.volume,
  });
}

export async function stop(id: SoundId) {
  await invoke('plugin:sound-player|stop', { id });
}

export async function stopAll() {
  await invoke('plugin:sound-player|stop_all');
}

export async function setVolume(id: SoundId, volume: number) {
  await invoke('plugin:sound-player|set_volume', { id, volume });
}

export async function pause(id: SoundId) {
  await invoke('plugin:sound-player|pause', { id });
}

export async function resume(id: SoundId) {
  await invoke('plugin:sound-player|resume', { id });
}

export async function isFinished(id: SoundId): Promise<boolean> {
  return await invoke<boolean>('plugin:sound-player|is_finished', { id });
}