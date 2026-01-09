#![cfg_attr(docsrs, feature(doc_cfg))]
//! Non-blocking communication over the niri socket.

use futures_lite::{Stream, stream};
use niri_ipc::{Event, Reply, Request, Response, socket::SOCKET_PATH_ENV};
use std::{env, io, path::Path};

pub use error::NiriReplyError;
mod error;

#[cfg(feature = "async-net")]
mod async_net_executor;

#[cfg(feature = "async-net")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-net")))]
pub type AsyncNetSocket = Socket<async_net_executor::AsyncNetStream>;

#[cfg(feature = "tokio")]
mod tokio_executor;

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub type TokioSocket = Socket<tokio_executor::TokioStream>;

pub struct Socket<S> {
    stream: S,
}

trait SocketStream: Sized {
    async fn connect_to(path: impl AsRef<Path>) -> Result<Self, io::Error>;
    async fn read_line(&mut self, buf: &mut String) -> Result<(), io::Error>;
    async fn write_all(&mut self, data: &[u8]) -> Result<(), io::Error>;
    async fn shutdown_write(&mut self);
}

#[expect(private_bounds)]
impl<S: SocketStream> Socket<S> {
    fn from_stream(stream: S) -> Self {
        Self { stream }
    }

    /// Connects to the default niri IPC socket.
    ///
    /// This is the async version of [Socket::connect](niri_ipc::socket::Socket::connect)
    pub async fn connect() -> Result<Self, io::Error> {
        let socket_path = env::var_os(SOCKET_PATH_ENV).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("{SOCKET_PATH_ENV} is not set, are you running this within niri?"),
            )
        })?;
        Self::connect_to(socket_path).await
    }
    /// Connects to the niri IPC socket at the given path.
    ///
    /// This is the async version of [Socket::connect_to](niri_ipc::socket::Socket::connect_to)
    pub async fn connect_to(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        S::connect_to(path).await.map(Self::from_stream)
    }

    /// Sends a request to niri and returns the response.
    ///
    /// This is the async version of [Socket::send](niri_ipc::socket::Socket::send), but with a
    /// flatten error type [NiriReplyError].
    pub async fn send(&mut self, request: Request) -> Result<Response, NiriReplyError> {
        let mut buf = serde_json::to_string(&request).unwrap();
        buf.push('\n');

        self.stream.write_all(buf.as_bytes()).await?;

        buf.clear();
        self.stream.read_line(&mut buf).await?;

        serde_json::from_str::<'_, Reply>(&buf)
            .map_err(io::Error::from)?
            .map_err(NiriReplyError::Niri)
    }

    /// Send request and reading event stream [`Event`]s from the socket.
    ///
    /// Note that unlike the [`Socket::read_events`](niri_ipc::socket::Socket::read_events),
    /// this method will send an [`EventStream`][Request::EventStream] request.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use niri_ipc::{Request, Response};
    /// use niri_ipc::socket::Socket;
    ///
    /// async fn print_events() -> Result<(), std::io::Error> {
    ///     let mut socket = Socket::connect().await?;
    ///
    ///     let reply = socket.into_event_stream().await;
    ///     if let Ok(event_stream) = reply {
    ///         let read_event = std::pin::pin!(event_stream);
    ///         while let Some(event) = read_event.next().await {
    ///             println!("Received event: {event:?}");
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn into_event_stream(
        mut self,
    ) -> Result<impl Stream<Item = Result<Event, io::Error>>, NiriReplyError> {
        self.send(Request::EventStream).await?;
        let mut stream = self.stream;
        stream.shutdown_write().await;
        Ok(Self::get_event_stream(stream))
    }

    fn get_event_stream(stream: S) -> impl Stream<Item = Result<Event, io::Error>> {
        stream::unfold(
            (stream, String::new()),
            |(mut stream, mut buf)| async move {
                buf.clear();

                let event = stream
                    .read_line(&mut buf)
                    .await
                    .and_then(|_| serde_json::from_str(&buf).map_err(From::from));
                Some((event, (stream, buf)))
            },
        )
    }
}
