use crate::{ReconnectToken, StreamId};

#[derive(Debug, Clone)]
pub enum ControlPacket {
    Init(StreamId),
    Data(StreamId, Vec<u8>),
    Refused(StreamId),
    End(StreamId),
    Ping(Option<ReconnectToken>),
}

pub const PING_INTERVAL: u64 = 30;

const EMPTY_STREAM: StreamId = StreamId([0xF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
const TOKEN_STREAM: StreamId = StreamId([0xF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);

impl ControlPacket {
    pub fn serialize(self) -> Vec<u8> {
        match self {
            ControlPacket::Init(sid) => [vec![0x01], sid.0.to_vec()].concat(),
            ControlPacket::Data(sid, data) => [vec![0x02], sid.0.to_vec(), data].concat(),
            ControlPacket::Refused(sid) => [vec![0x03], sid.0.to_vec()].concat(),
            ControlPacket::End(sid) => [vec![0x04], sid.0.to_vec()].concat(),
            ControlPacket::Ping(tok) => {
                let data = tok.map_or(EMPTY_STREAM.0.to_vec(), |t| {
                    [TOKEN_STREAM.0.to_vec(), t.0.into_bytes()].concat()
                });
                [vec![0x05], data].concat()
            }
        }
    }

    pub fn packet_type(&self) -> &str {
        match &self {
            ControlPacket::Ping(_) => "PING",
            ControlPacket::Init(_) => "INIT STREAM",
            ControlPacket::Data(_, _) => "STREAM DATA",
            ControlPacket::Refused(_) => "REFUSED",
            ControlPacket::End(_) => "END STREAM",
        }
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if data.len() < 9 {
            return Err("invalid DataPacket, missing stream id".into());
        }

        let mut stream_id = [0u8; 8];
        stream_id.clone_from_slice(&data[1..9]);
        let stream_id = StreamId(stream_id);

        let packet = match data[0] {
            0x01 => ControlPacket::Init(stream_id),
            0x02 => ControlPacket::Data(stream_id, data[9..].to_vec()),
            0x03 => ControlPacket::Refused(stream_id),
            0x04 => ControlPacket::End(stream_id),
            0x05 => {
                if stream_id == EMPTY_STREAM {
                    ControlPacket::Ping(None)
                } else {
                    ControlPacket::Ping(Some(ReconnectToken(
                        String::from_utf8_lossy(&data[9..]).to_string(),
                    )))
                }
            }
            _ => return Err("invalid control byte in DataPacket".into()),
        };

        Ok(packet)
    }
}
