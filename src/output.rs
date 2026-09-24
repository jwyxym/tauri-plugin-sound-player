use std::{
	sync::{
		Arc, Weak,
		atomic::{AtomicBool, Ordering},
		mpsc::{self, Receiver, Sender},
	},
	thread::{self, JoinHandle},
	time::Duration,
};

use parking_lot::Mutex;
use rodio::{
	ChannelCount, Device, DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink, Player, SampleRate, Source,
	cpal::{self, DeviceId, traits::{DeviceTrait, HostTrait}},
	mixer::{self, Mixer, MixerSource},
};

use crate::{Error, Result};

const DEVICE_POLL_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct AudioOutput {
	mixer: Mixer,
	players: Arc<Mutex<Vec<Weak<Player>>>>,
	shutdown: Sender<()>,
	worker: Option<JoinHandle<()>>,
}

impl AudioOutput {
	pub(crate) fn new() -> Result<Self> {
		// Keep the logical mixer independent of the physical output format/device.
		let (mixer, source) = mixer::mixer(rodio::nz!(2), rodio::nz!(48000));
		let source = SharedSource(Arc::new(Mutex::new(source)));
		let (shutdown, receiver) = mpsc::channel();
		let (ready, initialized) = mpsc::sync_channel(1);
		let players = Arc::new(Mutex::new(Vec::new()));
		let worker_players = players.clone();
		let worker = thread::Builder::new()
			.name("sound-player-output".into())
			.spawn(move || run_output(source, worker_players, receiver, ready))
			.map_err(|error| Error::OutputWorker(error.to_string()))?;
		let output = Self { mixer, players, shutdown, worker: Some(worker) };
		initialized.recv()
			.map_err(|error| Error::OutputWorker(error.to_string()))??;
		Ok(output)
	}

	pub(crate) fn mixer(&self) -> &Mixer {
		&self.mixer
	}

	pub(crate) fn track_player(&self, player: &Arc<Player>) {
		self.players.lock().push(Arc::downgrade(player));
	}
}

impl Drop for AudioOutput {
	fn drop(&mut self) {
		let _ = self.shutdown.send(());
		if let Some(worker) = self.worker.take() {
			let _ = worker.join();
		}
	}
}

struct DeviceOutput {
	stream: MixerDeviceSink,
	id: Option<DeviceId>,
	failed: Arc<AtomicBool>,
}

impl DeviceOutput {
	fn open(device: Device) -> Result<Self> {
		let id = device.id().ok();
		let failed = Arc::new(AtomicBool::new(false));
		let callback_failed = failed.clone();
		let mut stream = DeviceSinkBuilder::from_device(device)?
			.with_error_callback(move |_| {
				callback_failed.store(true, Ordering::Release);
			})
			.open_sink_or_fallback()?;
		stream.log_on_drop(false);
		Ok(Self { stream, id, failed })
	}
}

fn run_output(
	source: SharedSource,
	players: Arc<Mutex<Vec<Weak<Player>>>>,
	shutdown: Receiver<()>,
	ready: mpsc::SyncSender<Result<()>>,
) {
	let host = cpal::default_host();
	let initial = host.default_output_device()
		.ok_or_else(|| Error::from(DeviceSinkError::NoDevice))
		.and_then(DeviceOutput::open);
	let mut output = match initial {
		Ok(output) => {
			output.stream.mixer().add(source.clone());
			if ready.send(Ok(())).is_err() {
				return;
			}
			Some(output)
		}
		Err(error) => {
			let _ = ready.send(Err(error));
			return;
		}
	};

	while let Err(mpsc::RecvTimeoutError::Timeout) = shutdown.recv_timeout(DEVICE_POLL_INTERVAL) {
		// Inspect only in-memory player state while idle; do not query the audio backend.
		// Retain paused players so resuming them enables device checks again.
		let mut playing = false;
		players.lock().retain(|player| {
			let Some(player) = player.upgrade() else {
				return false;
			};
			if player.empty() {
				return false;
			}
			playing |= !player.is_paused();
			true
		});
		if !playing {
			continue;
		}
		let Some(device) = host.default_output_device() else {
			// With no output, retain the source without advancing its playback position.
			output.take();
			continue;
		};
		if output.as_ref().is_some_and(|output| output.failed.load(Ordering::Acquire)) {
			output.take();
		}
		let id = device.id().ok();
		if output.as_ref().is_some_and(|output| output.id == id) {
			continue;
		}

		match DeviceOutput::open(device) {
			Ok(next) => {
				// Stop the previous consumer before attaching the same source to the new sink.
				// The sink adapts sample rate/channels; Players and decoders remain intact.
				output.take();
				next.stream.mixer().add(source.clone());
				output = Some(next);
			}
			Err(_) => {
				// Keep a working old output if possible, and retry on the next poll.
			}
		}
	}
}

#[derive(Clone)]
struct SharedSource(Arc<Mutex<MixerSource>>);

impl Iterator for SharedSource {
	type Item = f32;

	fn next(&mut self) -> Option<Self::Item> {
		// An idle mixer must stay attached so future play calls are audible.
		Some(self.0.lock().next().unwrap_or(0.0))
	}
}

impl Source for SharedSource {
	fn current_span_len(&self) -> Option<usize> {
		None
	}

	fn channels(&self) -> ChannelCount {
		rodio::nz!(2)
	}

	fn sample_rate(&self) -> SampleRate {
		rodio::nz!(48000)
	}

	fn total_duration(&self) -> Option<Duration> {
		None
	}
}
