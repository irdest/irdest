use async_std::net::TcpStream;

/// I/O socket abstraction for a client application
///
/// We use this indirection here to allow future sockets to use
/// different formats (for example seqpack unix sockets).  Use
/// `[as_io()](Self::as_io) to get access to the underlying `Read +
/// Write` stream.
#[derive(Clone)]
pub(crate) enum Io {
    Tcp(TcpStream),
}

impl Io {
    pub(crate) fn as_io(&mut self) -> &mut (impl async_std::io::Write + async_std::io::Read) {
        match self {
            Self::Tcp(ref mut stream) => stream,
        }
    }
}
