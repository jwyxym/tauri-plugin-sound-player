use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("sound id `{0}` was not found")]
    NotFound(u64),

    #[error("failed to initialize audio output: {0}")]
    OutputStream(#[from] rodio::DeviceSinkError),

	#[error("audio output worker failed: {0}")]
	OutputWorker(String),

    #[error("failed to open audio file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to decode audio file: {0}")]
    Decode(#[from] rodio::decoder::DecoderError),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
