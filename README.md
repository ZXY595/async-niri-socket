# async-niri-socket

Async non-blocking communication with the niri IPC socket.

## Features

- **Two async runtime backends**: Supports both `async-net` and `tokio` via feature flags
- **Async socket connection**: Connect to the niri socket using `Socket::connect()` (uses `NIRI_SOCKET` env var) or `Socket::connect_to()` with a custom path
- **Request/response**: Send requests to niri and receive replies via the `send()` method
- **Event streaming**: Convert the socket into an infinite [`Event`] stream using `into_event_stream()`

## Example

```rust
use async_niri_socket::AsyncNetSocket;

type NiriSocket = AsyncNetSocket;

async fn run() -> Result<(), std::io::Error> {
    let mut socket = NiriSocket::connect().await?;

    // Send a request and get the reply
    let reply = socket
        .send(niri_ipc::Request::Workspaces)
        .await?;
    println!("Workspaces: {reply:?}");

    // Or convert into an event stream
    let reply = socket.into_event_stream().await;
    if let Ok(event_stream) = reply {
        let read_event = std::pin::pin!(event_stream);
        while let Some(event) = read_event.next().await {
            println!("Received event: {event:?}");
        }
    }

    Ok(())
}
```

## Niri Support Matrix
| async-niri-socket | niri |
|---------|---------|
| 0.0.1..=0.0.2 | 25.11.0 |
