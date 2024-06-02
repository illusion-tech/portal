use core::net::SocketAddr;

use portal_lib::{ClientHello, ClientId};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, info};

pub async fn spawn<A: Into<SocketAddr>>(addr: A) {
    let listener = TcpListener::bind(addr.into())
        .await
        .expect("failed to bind control server address");

    info!(
        "started control server on: {}",
        listener.local_addr().unwrap()
    );

    tokio::spawn(async move {
        loop {
            let socket = match listener.accept().await {
                Ok((socket, _)) => socket,
                Err(e) => {
                    error!("failed to accept socket: {:?}", e);
                    continue;
                }
            };

            info!("accepted connection from: {}", socket.peer_addr().unwrap());
            handle_new_connection(socket).await;
        }
    });
}

async fn handle_new_connection(socket: TcpStream) {
    try_client_handshake(socket).await;
}

async fn try_client_handshake<R>(mut socket: R) -> Option<()>
where
    R: AsyncReadExt + Unpin,
{
    // Read message length (u32)
    let len = socket
        .read_u32()
        .await
        .expect("failed to read client hello length") as usize;
    let mut buf = vec![0u8; len];

    // Read message
    socket
        .read_exact(&mut buf)
        .await
        .expect("failed to read client hello");

    let client_hello: ClientHello = match serde_json::from_slice(&buf) {
        Ok(ch) => ch,
        Err(error) => {
            error!(?error, "invalid client hello");
            return None;
        }
    };

    debug!("got client hello: {:?}", client_hello);

    None
}

#[cfg(test)]
mod tests {
    use portal_lib::ClientHello;
    use tokio_test::io::Builder;

    use crate::control_server_2::try_client_handshake;

    #[test]
    fn test_serde_json() {
        let client_hello = ClientHello::generate(None, portal_lib::ClientType::Anonymous);

        let json = serde_json::to_string(&client_hello).unwrap();
        let client_hello: ClientHello = serde_json::from_str(&json).unwrap();

        println!("{} - {}", json.len(), json);
        println!("{:?}", client_hello);
    }

    #[tokio::test]
    async fn test_try_client_handshake() {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let client_hello = ClientHello::generate(None, portal_lib::ClientType::Anonymous);
        let json = serde_json::to_string(&client_hello).unwrap();
        let len = json.len() as u32;

        let mock_stream = Builder::new()
            .read(&len.to_be_bytes())
            .read(json.as_bytes())
            .build();

        let result = try_client_handshake(mock_stream).await;
    }
}
