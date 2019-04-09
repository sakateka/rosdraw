use posixmq::{PosixMq, OpenOptions};
use std::str::FromStr;
use std::io;

const MODEL_QUEUE: &str = "/mq";
const MINE_QUEUE: &str = "/mq";
const VEHICLE_QUEUE: &str = "/mq";
const STATION_QUEUE_PREVIX: &str = "/mq";

pub enum Msg {
    StationStop(usize),
    Fuel(f32),
    Speed(f32),
    TankLoad,
    TankUnload,
    TankMove,
}

impl Msg {
    fn encode(self) -> Vec<u8> {
        match self {
            Msg::StationStop(idx) => format!("stop {}", idx).into(),
            Msg::Fuel(amount) => format!("fuel {}", amount).into(),
            Msg::Speed(speed) => format!("speed {}", speed).into(),
            Msg::TankLoad => "load".into(),
            Msg::TankUnload => "unload".into(),
            Msg::TankMove => "move".into(),
        }
    }
    fn decode(data: String) -> Msg {
        let data: Vec<_> = data.split_whitespace().collect();
        let atype = data[0];
        let mut payload = 0.0;
        if data.len() == 2 {
            payload = f32::from_str(data[1]).expect("Parse message payload");
        }
        match atype {
            "stop" => Msg::StationStop(payload as usize),
            "fuel" => Msg::Fuel(payload),
            "speed" => Msg::Speed(payload),
            "load" => Msg::TankLoad,
            "unload "=> Msg::TankUnload,
            "move" => Msg::TankMove,
            _ => unreachable!("Unexpected message type"),
        }
    }
}

pub struct PMQ {
    q: PosixMq,
}

impl PMQ {
    pub fn open(name: &str) -> Self {
        PMQ {
            q: OpenOptions::readwrite().open(name).expect("Create posix message queue"),
        }
    }
    pub fn nonblocking(mut self) -> Self {
        self.q.set_nonblocking(true).expect("Set nonblocking");
        self
    }

    pub fn send(&self, m: Msg) {
        let msg = m.encode();
        self.q.send(0, &msg[..]);
    }

    pub fn receive(&self) -> Result<Msg, io::Error> {
        let mut buf = vec![0; self.q.attributes().max_msg_len];
        let (_, len) = self.q.receive(&mut buf)?;
        let msg = String::from_utf8(buf[..len].to_vec()).expect("Decode message payload");
        Ok(Msg::decode(msg))
    }
}
