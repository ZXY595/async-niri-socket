pub use async_net::unix::UnixStream;
pub use futures_lite::{AsyncBufReadExt, AsyncWriteExt, io::BufReader};
pub use std::net::Shutdown;

use super::*;

pub struct AsyncNetStream(BufReader<UnixStream>);

impl SocketStream for AsyncNetStream {
    async fn connect_to(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let stream = UnixStream::connect(path.as_ref()).await?;
        let stream = BufReader::new(stream);
        Ok(Self(stream))
    }

    async fn read_line(&mut self, buf: &mut String) -> Result<(), io::Error> {
        self.0.read_line(buf).await.map(drop)
    }

    async fn write_all(&mut self, data: &[u8]) -> Result<(), io::Error> {
        self.0.write_all(data).await
    }

    async fn shutdown_write(&mut self) {
        let _ = self.0.get_mut().shutdown(Shutdown::Write);
    }
}
