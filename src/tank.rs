use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::thread;
use crate::posixmq::{PMQ, Msg, VEHICLE_QUEUE, MINE_QUEUE, STATION_QUEUE_PREFIX};


const DEFAULT_SCORE: i32 = 100;

#[derive(Copy, Clone, Debug)]
pub enum TankState {
    Refill(f32),
    Supply(f32),
    Load(f32),
    Unload(f32),
}

pub struct Tank {
    supply_target: Arc<Mutex<Option<usize>>>,
    capacity: f32,
    state: Arc<Mutex<TankState>>,
    q: PMQ,
}

impl Tank {
    pub fn new() -> Self {
        let t = Tank {
            supply_target: Arc::new(Mutex::new(None)),
            capacity: 20.0,
            state: Arc::new(Mutex::new(TankState::Refill(0.0))),
            q: PMQ::open(VEHICLE_QUEUE),
        };
        t.spawn_worker();
        t.load();
        t
    }

    fn spawn_worker(&self) {
        let capacity = self.capacity;
        let fourth = capacity * 0.25;
        let state = self.state.clone();
        let target = self.supply_target.clone();

        thread::spawn(move ||{
            info!("Employ vehicle worker");
            let mq_v = PMQ::open(VEHICLE_QUEUE);
            let mq_m = PMQ::open(MINE_QUEUE);

            let mut mq_s: HashMap<usize, PMQ> = HashMap::new();
            let mut idle_stations: HashMap<usize, i32> = HashMap::new();

            let mut fuel = 0.0;

            loop {
                let msg = mq_v.receive();
                trace!("Tank receive msg: {:?}", msg);
                match msg {
                    Ok(Msg::Fuel(amount)) => {
                        fuel += amount;
                        let mut state = state.lock().unwrap();
                        *state = match *state {
                            TankState::Load(_) => TankState::Load(fuel / capacity * 100.0),
                            TankState::Unload(_) => {
                                let t = target.lock().unwrap();
                                info!("Remain fuel {} from station {:?}", amount, *t);
                                // send unload
                                match *t {
                                    Some(idx) => {
                                        let s = idle_stations.remove(&idx);
                                        info!("Remove station {:?} from idle stations", s);
                                        // trigger TankMove
                                        mq_s[&idx].send(Msg::Fuel(0.0))
                                            .expect(format!("Send fuel to station {}", idx).as_ref());
                                    },
                                    _ => unreachable!("Try unload to unknown station!"),
                                };
                                TankState::Unload(fuel / capacity * 100.0)
                            },
                            _ => {
                                error!("Receive fuel from unexpected state: {:?}", *state);
                                *state
                            },
                        }
                    },
                    Ok(Msg::TankLoad) => {
                        mq_m.send(Msg::Fuel(capacity - fuel)).expect("Send fuel request");
                        *state.lock().unwrap() = TankState::Load(fuel / capacity * 100.0);
                    },
                    Ok(Msg::TankUnload) => {
                        let sub = f32::min(fuel, fourth);
                        fuel = f32::max(fuel - sub, 0.0);
                        *state.lock().unwrap() = TankState::Unload(fuel / capacity * 100.0);
                        match *target.lock().unwrap() {
                            Some(idx) => {
                                if let Some(level) = idle_stations.get_mut(&idx) {
                                    *level -= 1;
                                }
                                mq_s[&idx].send(Msg::Fuel(sub))
                                    .expect(format!("Send fuel to station {}", idx).as_ref());
                            },
                            _ => unreachable!("Try unload to unknown station!"),
                        }
                    },
                    Ok(Msg::TankMove) => {
                        let mut s = state.lock().unwrap();
                        let mut t = target.lock().unwrap();
                        match *s {
                            TankState::Load(_) => {
                                if fuel < fourth {
                                    debug!("not enough fuel received: val={}, need={}", fuel, fourth);
                                    mq_m.send(Msg::Fuel(capacity - fuel)).expect("Send fuel request");
                                } else {
                                    *t = match *t {
                                        Some(idx) => {
                                            Some(idx)
                                        },
                                        None => {
                                            let next = idle_stations.iter().max_by(|a, b| {
                                                (*a.1 as u32).cmp(&(*b.1 as u32))
                                            });
                                            if let Some(idx) = next {
                                                Some(*idx.0)
                                            } else {
                                                None
                                            }
                                        },
                                    };
                                    if t.is_some() {
                                        *s = TankState::Supply(fuel / capacity * 100.0);
                                    }
                                    info!("Supply to station {:?}", *t);
                                }
                            },
                            TankState::Unload(_) | TankState::Refill(_) | TankState::Supply(_) => {
                                let next = idle_stations.iter().max_by(|a, b| {
                                    (*a.1 as u32).cmp(&(*b.1 as u32))
                                });
                                if let Some(idx) = next {
                                    *t = Some(*idx.0);
                                    if fuel == 0.0 {
                                        *s = TankState::Refill(fuel / capacity * 100.0);
                                    } else {
                                        *s = TankState::Supply(fuel / capacity * 100.0);
                                    }
                                } else {
                                    *t = None;
                                    *s = TankState::Refill(fuel / capacity * 100.0);
                                }
                                trace!("Set next target to {:?} from {:?}, state={:?}", next, idle_stations, *s);
                            },
                        }
                    },
                    Ok(Msg::IdleStation(idx)) => {
                        if !mq_s.contains_key(&idx) {
                            let q_name = format!("{}{}", STATION_QUEUE_PREFIX, idx);
                            mq_s.insert(idx, PMQ::open(q_name.as_ref()));
                        }
                        if *target.lock().unwrap() == None {
                            *target.lock().unwrap() = Some(idx);
                            // trigger TankMove
                            mq_m.send(Msg::Fuel(0.0)).expect("Send fuel request");
                        }
                        idle_stations.insert(idx, DEFAULT_SCORE);
                    },
                    Err(e) => {
                        error!("Vehicle queue receive error: {:?}", e);
                        break
                    },
                }
            }
        });
    }

    pub fn get_target(&self) -> Option<usize> {
        *self.supply_target.lock().unwrap()
    }

    pub fn get_state(&self) -> TankState {
        *self.state.lock().unwrap()
    }

    pub fn load(&self) {
        let msg = Msg::TankLoad;
        trace!("Send message: {:?}", msg);
        self.q.send(msg).expect("Send tank load message");
    }

    pub fn unload(&self) {
        let msg = Msg::TankUnload;
        trace!("Send message: {:?}", msg);
        self.q.send(msg).expect("Send tank unload message");
    }
}

