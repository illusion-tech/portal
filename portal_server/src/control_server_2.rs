use core::{net::SocketAddr, time::Duration};
use std::error::Error;

use portal_lib::{agent::AgentHandshake, ControlPacket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::OwnedWriteHalf, TcpListener, TcpStream},
    sync::mpsc::unbounded_channel,
};
use tracing::{debug, error, info};

use crate::agent_manager::{Agent, AgentManager, Tunnel};

pub async fn spawn<A: Into<SocketAddr>>(addr: A) {
    let listener: TcpListener = TcpListener::bind(addr.into())
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

async fn handle_new_connection(mut socket: TcpStream)
// where
//     R: AsyncReadExt + Unpin,
{
    let handshake: AgentHandshake = try_agent_handshake(&mut socket).await.unwrap();
    let (tx, rx) = unbounded_channel::<ControlPacket>();

    let (_, mut tcp_tx) = socket.into_split();

    let agent = Agent {
        id: handshake.agent_id.clone(),
        tunnel: Tunnel { tx, rx },
    };

    let agent_manager = AgentManager::instance();

    agent_manager.add(agent);

    // agent.tunnel.tx.send(ControlPacket::Ping(None)).unwrap();

    let agent_id = handshake.agent_id.clone();
    info!("spawning tunnel_to_agent task for agent: {}", agent_id);
    tokio::spawn(async move {
        let mut agent = agent_manager.get_mut(&agent_id).unwrap();

        tunnel_to_agent(&mut agent, &mut tcp_tx).await;
    });

    let agent_id = handshake.agent_id.clone();
    info!("spawning ping task for agent: {}", agent_id);
    tokio::spawn(async move {
        loop {
            let agent = agent_manager.get(&agent_id).unwrap();

            info!("sending ping to agent: {}", agent_id);
            agent.tunnel.tx.send(ControlPacket::Ping(None)).unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

async fn try_agent_handshake<R>(socket: &mut R) -> Result<AgentHandshake, Box<dyn Error>>
where
    R: AsyncReadExt + Unpin,
{
    debug!("waiting for agent handshake message");
    // Read message length (u32)
    let len = socket.read_u32().await? as usize;
    let mut buf = vec![0u8; len];

    debug!("got agent handshake message length: {}", len);
    // Read message
    socket.read_exact(&mut buf).await?;

    let handshake = match serde_json::from_slice::<AgentHandshake>(&buf) {
        Ok(ch) => ch,
        Err(error) => {
            error!(?error, "invalid agent handshake message");
            return Err(Box::new(error));
        }
    };

    debug!("got agent handshake message: {:?}", handshake);

    Ok(handshake)
}

/// Send data to agent through tunnel, the data flows from server to the agent.
/// This is used to forward the data received by the server to the agent.
async fn tunnel_to_agent(agent: &mut Agent, tx: &mut OwnedWriteHalf) {
    loop {
        let packet = agent.tunnel.rx.recv().await.unwrap();
        tx.write_all(&packet.serialize()).await.unwrap();
    }
}

/// process the data sent from the agent through the tunnel, the data flows from
/// the agent to the server.
/// This is used to process the data received by the agent and forward it to the
/// client.
fn tunnel_from_agent() {}

#[cfg(test)]
mod tests {

    use portal_lib::agent::AgentHandshake;
    use tokio_test::io::Builder;

    use crate::control_server_2::try_agent_handshake;

    #[tokio::test]
    async fn test_try_agent_handshake() {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let handshake = AgentHandshake::builder()
            .agent_id("agent-123")
            .build()
            .unwrap();

        let json = serde_json::to_string(&handshake).unwrap();
        let len = json.len() as u32;

        println!("json: {}", json);

        let mut mock_stream = Builder::new()
            .read(&len.to_be_bytes())
            .read(json.as_bytes())
            .build();

        _ = try_agent_handshake(&mut mock_stream).await;
    }
}
