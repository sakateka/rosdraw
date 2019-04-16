use crate::model::NUM_STATIONS;
use posixmq::{unlink, OpenOptions, PosixMq};
use std::io;
use std::str::FromStr;

pub const MINE_QUEUE: &str = "/mq-m";
pub const VEHICLE_QUEUE: &str = "/mq-v";
pub const STATION_QUEUE_PREFIX: &str = "/mq-s";

pub struct PMQ {
    q: PosixMq,
}

impl PMQ {
    pub fn open(name: &str) -> Self {
        PMQ {
            q: OpenOptions::readwrite()
                .create()
                .open(name)
                .expect("Create posix message queue"),
        }
    }

    pub fn nonblocking(self) -> Self {
        self.q.set_nonblocking(true).expect("Set nonblocking");
        self
    }

    pub fn set_nonblocking(&self, b: bool) {
        self.q.set_nonblocking(b).expect("set nonblocking");
    }

    pub fn send(&self, m: Msg) -> Result<(), io::Error> {
        let msg = m.encode();
        self.q.send(0, &msg[..])
    }

    pub fn receive(&self) -> Result<Msg, io::Error> {
        let mut buf = vec![0; self.q.attributes().max_msg_len];
        let (_, len) = self.q.receive(&mut buf)?;
        let msg = String::from_utf8(buf[..len].to_vec()).expect("Decode message payload");
        Ok(Msg::decode(msg))
    }
}

#[derive(Debug)]
pub enum Msg {
    IdleStation(usize),
    Fuel(f32),
    TankLoad,
    TankUnload,
    TankMove,
}

impl Msg {
    fn encode(self) -> Vec<u8> {
        match self {
            Msg::IdleStation(idx) => format!("idle {}", idx).into(),
            Msg::Fuel(amount) => format!("fuel {}", amount).into(),
            Msg::TankLoad => "load".into(),
            Msg::TankUnload => "unload".into(),
            Msg::TankMove => "move".into(),
        }
    }
    fn decode(data: String) -> Msg {
        let data: Vec<_> = data.split_whitespace().collect();
        let atype = data[0];
        let mut payload = 0.0;
        if data.len() > 1 {
            payload = f32::from_str(data[1]).expect("Parse message payload");
        }
        match atype {
            "idle" => Msg::IdleStation(payload as usize),
            "fuel" => Msg::Fuel(payload),
            "load" => Msg::TankLoad,
            "unload" => Msg::TankUnload,
            "move" => Msg::TankMove,
            _ => unreachable!("Unexpected message type {}", atype),
        }
    }
}

pub fn cleanup_posix_queues() {
    info!("Unlink posix message queues");
    info!("unlink {}: {:?}", MINE_QUEUE, unlink(MINE_QUEUE));
    info!("unlink {}: {:?}", VEHICLE_QUEUE, unlink(VEHICLE_QUEUE));
    for i in 0..NUM_STATIONS {
        let q_name = format!("{}{}", STATION_QUEUE_PREFIX, i);
        info!("unlink {}: {:?}", q_name, unlink(&q_name));
    }
}
