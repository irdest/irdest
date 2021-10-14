pub(crate) type Result<T> = std::result::Result<T, ChunkError>;

pub(crate) enum ChunkError {
    /// An error occured while serialising data
    Encoding(bincode::Error),
    /// A generic I/O error while dealing with chunks
    Io(std::io::Error),
    /// Chunk overflow error
    SoftOverflow(Vec<u8>),
}

impl From<bincode::Error> for ChunkError {
    fn from(e: bincode::Error) -> Self {
        Self::Encoding(e)
    }
}

impl From<std::io::Error> for ChunkError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
