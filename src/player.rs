use std::{
    collections::HashMap,
    fmt,
    fs::File,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use parking_lot::Mutex;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use serde::Serialize;

use crate::Result;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct SoundId(u64);

impl SoundId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
pub struct SoundHandle {
    id: SoundId,
    player: Arc<Player>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayMode {
    Once,
    Loop,
}

pub struct SoundPlayer {
    stream: MixerDeviceSink,
    next_id: AtomicU64,
    players: Mutex<HashMap<SoundId, SoundHandle>>,
}

pub type SoundPlayerState = SoundPlayer;

impl SoundPlayer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            stream: DeviceSinkBuilder::open_default_sink()?,
            next_id: AtomicU64::new(1),
            players: Mutex::new(HashMap::new()),
        })
    }

    pub fn play_once<P: AsRef<Path>>(&self, path: P, volume: f32) -> Result<SoundId> {
        self.play(path, PlayMode::Once, volume)
    }

    pub fn play_loop<P: AsRef<Path>>(&self, path: P, volume: f32) -> Result<SoundId> {
        self.play(path, PlayMode::Loop, volume)
    }

    pub fn play<P: AsRef<Path>>(&self, path: P, mode: PlayMode, volume: f32) -> Result<SoundId> {
        self.retain_playing();

        let decoder = Decoder::try_from(File::open(path)?)?;
        let player = Arc::new(Player::connect_new(self.stream.mixer()));
        player.set_volume(normalize_volume(volume));

        match mode {
            PlayMode::Once => player.append(decoder),
            PlayMode::Loop => player.append(decoder.repeat_infinite()),
        }

        let id = self.next_sound_id();
        let handle = SoundHandle { id, player };
        self.players.lock().insert(id, handle);

        Ok(id)
    }

    pub fn stop(&self, id: SoundId) {
        if let Some(handle) = self.players.lock().remove(&id) {
            handle.stop();
        }
    }

    pub fn stop_all(&self) {
        let mut players = self.players.lock();
        for handle in players.values() {
            handle.stop();
        }
        players.clear();
    }

    pub fn set_volume(&self, id: SoundId, volume: f32) {
        if let Some(handle) = self.players.lock().get(&id) {
            handle.set_volume(volume);
        }
    }

    pub fn pause(&self, id: SoundId) {
        if let Some(handle) = self.players.lock().get(&id) {
            handle.pause();
        }
    }

    pub fn resume(&self, id: SoundId) {
        if let Some(handle) = self.players.lock().get(&id) {
            handle.resume();
        }
    }

    pub fn is_finished(&self, id: SoundId) -> bool {
        self.players
            .lock()
            .get(&id)
            .map(SoundHandle::is_finished)
            .unwrap_or(true)
    }

    pub fn retain_playing(&self) {
        self.players
            .lock()
            .retain(|_, handle| !handle.is_finished());
    }

    fn next_sound_id(&self) -> SoundId {
        SoundId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

impl SoundHandle {
    pub fn id(&self) -> SoundId {
        self.id
    }

    pub fn set_volume(&self, volume: f32) {
        self.player.set_volume(normalize_volume(volume));
    }

    pub fn volume(&self) -> f32 {
        self.player.volume()
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn is_finished(&self) -> bool {
        self.player.empty()
    }

    pub fn position(&self) -> Duration {
        self.player.get_pos()
    }
}

impl fmt::Debug for SoundHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoundHandle").field("id", &self.id).finish()
    }
}

impl Drop for SoundPlayer {
    fn drop(&mut self) {
        self.stop_all();
    }
}

fn normalize_volume(volume: f32) -> f32 {
    if volume.is_finite() {
        volume.max(0.0)
    } else {
        1.0
    }
}

impl From<u64> for SoundId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
