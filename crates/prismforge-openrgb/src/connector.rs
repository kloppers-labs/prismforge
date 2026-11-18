use std::future::Future;
use std::io;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

pub trait Connector: Send + Sync {
    type Stream: AsyncRead + AsyncWrite + Send + 'static;

    fn connect(&self) -> impl Future<Output = io::Result<Self::Stream>> + Send;
}

impl<F, Fut, S> Connector for F
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = io::Result<S>> + Send,
    S: AsyncRead + AsyncWrite + Send + 'static,
{
    type Stream = S;

    fn connect(&self) -> impl Future<Output = io::Result<S>> + Send {
        self()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpConnector {
    pub host: String,
    pub port: u16,
}

impl TcpConnector {
    pub const DEFAULT_PORT: u16 = 6742;

    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }
}

impl Connector for TcpConnector {
    type Stream = TcpStream;

    async fn connect(&self) -> io::Result<TcpStream> {
        let stream = TcpStream::connect((self.host.as_str(), self.port)).await?;
        stream.set_nodelay(true)?;
        Ok(stream)
    }
}
