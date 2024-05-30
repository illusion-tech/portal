use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_hdr_async, WebSocketStream};
use tracing::debug;
use tracing::{error, Instrument};
use tungstenite::handshake::server::{Request, Response};
use url::quirks::host;

/// Response Constants

const HTTP_REDIRECT_RESPONSE:&[u8] = b"HTTP/1.1 301 Moved Permanently\r\nLocation: https://tunnelto.dev/\r\nContent-Length: 20\r\n\r\nhttps://tunnelto.dev";
const HTTP_INVALID_HOST_RESPONSE: &[u8] =
    b"HTTP/1.1 400\r\nContent-Length: 23\r\n\r\nError: Invalid Hostname";
const HTTP_NOT_FOUND_RESPONSE: &[u8] =
    b"HTTP/1.1 404\r\nContent-Length: 23\r\n\r\nError: Tunnel Not Found";
const HTTP_ERROR_LOCATING_HOST_RESPONSE: &[u8] =
    b"HTTP/1.1 500\r\nContent-Length: 27\r\n\r\nError: Error finding tunnel";
const HTTP_TUNNEL_REFUSED_RESPONSE: &[u8] =
    b"HTTP/1.1 500\r\nContent-Length: 32\r\n\r\nTunnel says: connection refused.";
const HTTP_OK_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
const HEALTH_CHECK_PATH: &[u8] = b"/0xDEADBEEF_HEALTH_CHECK";

//异步函数，接收一个 TCP 流 incoming 并尝试连接本地控制服务器。在连接建立后，它会将数据从客户端流转发到控制服务器，并将控制服务器的响应数据转发回客户端
async fn direct_to_control(mut incoming: TcpStream) {
    //使用TcpStream::connect建立到本地服务器的连接，并用format组合给出控制服务器的地质，并使用control_port来获取配置文件的端口，await代表建立成功就进入ok否则进入Error
    let mut control_socket =
        match TcpStream::connect(format!("localhost:{}", get_config().control_port)).await {
            Ok(s) => s,
            Err(error) => {
                tracing::warn!(?error, "failed to connect to local control server");
                return;
            }
        };
    //使用split方法将TcpStream对象分割为只读和只写两部分，来进行后续的读写操作
    let (mut control_r, mut control_w) = control_socket.split();
    let (mut incoming_r, mut incoming_w) = incoming.split();
    // 创建两个异步任务，将数据从控制服务器流复制到客户端流，并且将数据从客户端流复制到服务器流
    let join_1 = tokio::io::copy(&mut control_r, &mut incoming_w);
    let join_2 = tokio::io::copy(&mut incoming_r, &mut control_w);
    //同时运行两个异步任务
    match futures::future::join(join_1, join_2).await {
        (Ok(_), Ok(_)) => {}
        (Err(error), _) | (_, Err(error)) => {
            tracing::error!(?error, "directing stream to control failed");
        }
    }
}

#[tracing::instrument(skip(socket))]
//异步函数，接收一个tcp流socket并处理传入的连接请求。
pub async fn accept_connection(mut socket: TcpStream) {
// 通过调用peek_http_request_host获取预读取主机信息的StreamWithPeekedHost既然钩体，读取成功解析并执行否则返回。
    let StreamWithPeekedHost {
        mut socket,
        host,
        forwarded_for,
    } = match peek_http_request_host(socket).await {
        Some(s) => s,
        None => return,
    };

    let config = get_config();
    
    // 健康检查
    tracing::info!(%host, %forwarded_for, "new remote connection");
    tracing::debug!("Allowed hosts: {}", config.allowed_hosts.join(", "));

    // parse the host string and find our client
    // 检查主机列表传入的主机是否被允许
    if !config.allowed_hosts.contains(&host) {
        error!("redirect to homepage");
        let _ = socket.write_all(HTTP_REDIRECT_RESPONSE).await;
        return;
    }
    let host = match validate_host_prefix(&host) {
        Some(sub_domain) => sub_domain,
        None => {
            error!("invalid host specified");
            let _ = socket.write_all(HTTP_INVALID_HOST_RESPONSE).await;
            return;
        }
    };
    // 特殊情况下会重定向到控制服务器
    if host.as_str() == "wormhole" {
        direct_to_control(socket).await;
        return;
    }

    // find the client listening for this host
    //如果都不符合的情况下，尝试寻找监听主机的客户端
    let client = match Connections::find_by_host(&host) {
        Some(client) => client.clone(),
        None => {
            // check other instances that may be serving this host
            match network::instance_for_host(&host).await {
                Ok((instance, _)) => {
                    network::proxy_stream(instance, socket).await;
                    return;
                }
                Err(network::Error::DoesNotServeHost) => {
                    error!(%host, "no tunnel found");
                    let _ = socket.write_all(HTTP_NOT_FOUND_RESPONSE).await;
                    return;
                }
                Err(error) => {
                    error!(%host, ?error, "failed to find instance");
                    let _ = socket.write_all(HTTP_ERROR_LOCATING_HOST_RESPONSE).await;
                    return;
                }
            }
        }
    };

    // allocate a new stream for this request
    //创建新的ActiveStream，返回一个接收端queue_rx，用于从其他任务接收数据。
    let (active_stream, queue_rx) = ActiveStream::new(client.clone());
    //获取新建活动流的ID
    let stream_id = active_stream.id.clone();
    //用于记录调试日志
    tracing::debug!(
        stream_id = %active_stream.id.to_string(),
        "new stream connected"
    );
    //split将socket分为stream和只写的sink两个部分
    let (stream, sink) = tokio::io::split(socket);

    // add our stream
    //将新创建的活动流存储到活动流列表，后续可以根据ID查找和管理
    get_active_streams().insert(stream_id.clone(), active_stream.clone());

    // read from socket, write to client
    //
    let span = observability::remote_trace("process_tcp_stream");
    //创建异步任务，调用pross_tcp_stream来处理stream读取的数据，并将结果发送给对应客户端。
    tokio::spawn(
        async move {
            process_tcp_stream(active_stream, stream).await;
        }
            .instrument(span),
    );

    // read from client, write to socket
    let span = observability::remote_trace("tunnel_to_stream");
    tokio::spawn(
        async move {
            tunnel_to_stream(host, stream_id, sink, queue_rx).await;
        }
            .instrument(span),
    );
}

//验证主机前缀是否符合配置文件中允许的主机前缀，返回一个Option<string>类型的结果
fn validate_host_prefix(host: &str) -> Option<String> {
    //传入的字符串格式化成类似于url个是，后续解析
    let url = format!("http://{}", host);
    debug!(%url, "parsing host");
    //解析url，获取主机部分的信息，并存储在host变量中。
    let host = match url::Url::parse(&url)
        .map(|u| u.host().map(|h| h.to_owned()))
        .unwrap_or(None)
    {
        Some(domain) => domain.to_string(),
        None => {
            error!("invalid host header");
            return None;
        }
    };
//将主机名按照点号拆分成片段，并存储在domain_segments中
    let domain_segments = host.split('.').collect::<Vec<&str>>();
    let prefix = &domain_segments[0];
    let remaining = &domain_segments[1..].join(".");

    let config = get_config();

    debug!(%host, %prefix, "parsed host");
    debug!(%prefix, %remaining, "parsed host");
    debug!(?config.allowed_hosts, "allowed hosts");
//如果允许的主机前缀包含剩部分则返回
    if config.allowed_hosts.contains(remaining) {
        Some(prefix.to_string())
    } else {
        None
    }
}



//存储tcp流，主机名和转发信息
struct StreamWithPeekedHost {
    socket: TcpStream,
    host: String,
    forwarded_for: String,
}

/// Filter incoming remote streams
#[tracing::instrument(skip(socket))]
async fn peek_http_request_host(mut socket: TcpStream) -> Option<StreamWithPeekedHost> {
    /// Note we return out if the host header is not found
    /// within the first 4kb of the request.
    const MAX_HEADER_PEAK: usize = 4096;//最大预读4kb
    let mut buf = vec![0; MAX_HEADER_PEAK]; //1kb的缓冲区

    tracing::debug!("checking stream headers");
//通过socket.peek方法异步读取tcp流到缓冲区buf，返回读取的字节数，并使用match处理异步操作的结果
    let n = match socket.peek(&mut buf).await {
        Ok(n) => n,
        Err(e) => {
            error!("failed to read from tcp socket to determine host: {:?}", e);
            return None;
        }
    };

    // make sure we're not peeking the same header bytes
    //无法预读头部信息，直接返回None
    if n == 0 {
        tracing::debug!("unable to peek header bytes");
        return None;
    }

    tracing::debug!("peeked {} stream bytes ", n);
//headers数组存储HTTP头部信息，长度为64,req来解析HTTP请求的头部信息，并传入之前创建的headers数组

    let host = "test.portal.illusiontech.cn";
    let forwarded_for = "";

    return Some(StreamWithPeekedHost {
        socket,
        host: host.to_string(),
        forwarded_for:forwarded_for.to_string(),
    });
    tracing::info!("found no host header, dropping connection.");
    None
}

/// Process Messages from the control path in & out of the remote stream
#[tracing::instrument(skip(tunnel_stream, tcp_stream))]
//处理路径消息，远程的进出流
async fn process_tcp_stream(mut tunnel_stream: ActiveStream, mut tcp_stream: ReadHalf<TcpStream>) {
    // send initial control stream init to client
    control_server::send_client_stream_init(tunnel_stream.clone()).await;

    // now read from stream and forward to clients
    let mut buf = [0; 1024];

    loop {
        // client is no longer connected
        if Connections::get(&tunnel_stream.client.id).is_none() {
            debug!("client disconnected, closing stream");
            let _ = tunnel_stream.tx.send(StreamMessage::NoClientTunnel).await;
            tunnel_stream.tx.close_channel();
            return;
        }

        // read from stream
        let n = match tcp_stream.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                error!("failed to read from tcp socket: {:?}", e);
                return;
            }
        };

        if n == 0 {
            debug!("stream ended");
            let _ = tunnel_stream
                .client
                .tx
                .send(ControlPacket::End(tunnel_stream.id.clone()))
                .await
                .map_err(|e| {
                    error!("failed to send end signal: {:?}", e);
                });
            return;
        }

        debug!("read {} bytes", n);

        let data = &buf[..n];
        let packet = ControlPacket::Data(tunnel_stream.id.clone(), data.to_vec());

        match tunnel_stream.client.tx.send(packet.clone()).await {
            Ok(_) => debug!(client_id = %tunnel_stream.client.id, "sent data packet to client"),
            Err(_) => {
                error!("failed to forward tcp packets to disconnected client. dropping client.");
                Connections::remove(&tunnel_stream.client);
            }
        }
    }
}

#[tracing::instrument(skip(sink, stream_id, queue))]
//从接收器队列获取数据，然后将数据写入到TCP流中，指导队列结束或者错误
async fn tunnel_to_stream(
    subdomain: String,
    stream_id: StreamId,
    mut sink: WriteHalf<TcpStream>,
    mut queue: UnboundedReceiver<StreamMessage>,
) {
    loop {
        //从队列异步获取下一个消息
        let result = queue.next().await;
        //匹配处理从接收器队列获取的消息，根据消息内容做出相应的处理。没有消息就result设置为None
        let result = if let Some(message) = result {
            match message {
                StreamMessage::Data(data) => Some(data),
                StreamMessage::TunnelRefused => {
                    tracing::debug!(?stream_id, "tunnel refused");
                    let _ = sink.write_all(HTTP_TUNNEL_REFUSED_RESPONSE).await;
                    None
                }
                StreamMessage::NoClientTunnel => {
                    tracing::info!(%subdomain, ?stream_id, "client tunnel not found");
                    let _ = sink.write_all(HTTP_NOT_FOUND_RESPONSE).await;
                    None
                }
            }
        } else {
            None
        };
    //根据处理的结果决定是否继续进行写入
        let data = match result {
            Some(data) => data,
            None => {
                tracing::debug!("done tunneling to sink");
                let _ = sink.shutdown().await.map_err(|_e| {
                    error!("error shutting down tcp stream");
                });

                get_active_streams().remove(&stream_id);
                return;
            }
        };
//数据写入tcp流，等待写入操作完成。
        let result = sink.write_all(&data).await;
//检查写入是否有错
        if let Some(error) = result.err() {
            tracing::warn!(?error, "stream closed, disconnecting");
            return;
        }
    }
}
