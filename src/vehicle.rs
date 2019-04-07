use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou::state::time;
use nannou::Draw;
use crate::model::Ids;

pub struct Vehicle {
    mine: widget::Id,
    pub wh: Point2,
    from: widget::Id,
    to: widget::Id,
    tank: Tank,
    //direction: Point2,
    position: Option<Point2>,
    distance: f32,
}

enum TankState {
    Load,
    Unload,
    Nop,
}

struct Tank {
    state: TankState,
    fuel: f32,
    capacity: f32,
    labor: time::Duration,
    chunk: f32,
}

impl Tank {
    fn load(&mut self) {
        if self.fuel == self.capacity {
            self.stop_transfer();
            return;
        }
        self.state = TankState::Load;
        if self.labor.ms() > self.percentage() as f64 * 10.0 /*ms*/ {
            self.fuel = f32::min(self.capacity, self.fuel + self.capacity * self.chunk);
            trace!("Load fuel: {:.0}%", self.percentage());
        }
    }

    fn unload(&mut self) {
        if self.fuel == 0.0 {
            self.stop_transfer();
            return;
        }
        self.state = TankState::Unload;
        if self.labor.ms() > (100.0 - self.percentage()) as f64 * 10.0 /*ms*/ {
            self.fuel = (self.fuel -  self.capacity * self.chunk).round();
            trace!("Unload fuel: {:.0}%", self.percentage());
        }
    }

    fn transfer_fuel(&mut self, labor: time::Duration) {
        if self.in_work() {
            *self.labor = *self.labor + *labor;
            match self.state {
                TankState::Load => self.load(),
                TankState::Unload => self.unload(),
                _ => (),
            };
        }
    }

    fn stop_transfer(&mut self) {
        self.state = TankState::Nop;
        self.labor = Self::rested();
    }


    fn in_work(&self) -> bool {
        match self.state {
            TankState::Nop => false,
            _ => true,
        }
    }

    fn percentage(&self) -> f32 {
        (self.fuel / self.capacity * 100.0).round()
    }

    fn rested() -> time::Duration {
        std::time::Duration::new(0,0).into()
    }
}

impl Vehicle {
    pub fn new(ids: Ids) -> Self {
        Vehicle {
            mine: ids.mine,
            wh: pt2(50.0, 20.0),
            from: ids.mine,
            to: ids.stations[0],
            tank: Tank{
                state: TankState::Load,
                fuel: 0.0,
                capacity: 20.0,
                labor: Tank::rested(),
                chunk: 0.10,
            },
            //direction: pt2(0.0, 0.0),
            position: None,
            distance: 0.0,
        }
    }

    fn background(&self) -> Rgba {
        if (self.to == self.mine) && !(self.tank.in_work()) {
            WHITE
        } else {
            GREEN
        }
    }

    pub fn draw(&self, draw: &Draw) {
        draw.rect()
            .wh(self.wh)
            .color(self.background())
            .xy(self.position.unwrap());
        if self.tank.in_work() {
            let transfer_wh = pt2(self.wh.x, self.wh.y * (1.0 - self.tank.percentage()/100.0));
            let pos = self.position.unwrap();
            draw.rect()
                .wh(transfer_wh)
                .color(WHITE)
                .x_y(pos.x, pos.y + (self.wh.y/2.0 - transfer_wh.y/2.0));
        }
    }

    /*
    pub fn update(&mut self, ui: &mut UiCell, speed: f32, since_last: time::Duration) {
        let points = [self.from, self.to].iter_mut().map(|id|{
            let r = ui.rect_of(*id).unwrap();
            if *id == self.mine {
                pt2(r.x.end as f32 + self.wh.x / 2.0, r.y.start as f32 + self.wh.y / 2.0)
            } else {
                pt2(r.x.end as f32 - self.wh.x / 2.0, r.y.start as f32 - self.wh.y)
            }
        }).collect::<Vec<_>>();
        let start = points[0];
        let target = points[1];

        self.direction = target - start;

        if let Some(p) = self.position {
            if self.tank.in_work() {
                self.tank.transfer_fuel(since_last);
                return;
            }
            let mut new_p = p + self.direction * speed;
            if new_p.distance(target) > p.distance(target) {
                trace!("Swap target for vehicle");
                std::mem::swap(&mut self.from, &mut self.to);
                if self.to == self.mine {
                    self.unloading();
                } else {
                    self.loading();
                }
                new_p = target;
            }
            self.position = Some(new_p);
        } else  {
            self.position = Some(start);
        }
    }
    */

    pub fn update(&mut self, ui: &mut UiCell, speed: f32, since_last: time::Duration) {
        // TODO: handle event RESIZE
        let points = [self.from, self.to].iter_mut().map(|id|{
            let r = ui.rect_of(*id).unwrap();
            if *id == self.mine {
                pt2(r.x.end as f32 + self.wh.x / 2.0, r.y.start as f32 + self.wh.y / 2.0)
            } else {
                pt2(r.x.end as f32 - self.wh.x / 2.0, r.y.start as f32 - self.wh.y)
            }
        }).collect::<Vec<_>>();
        let start = points[0];
        let target = points[1];
        self.distance = target.distance(start);
        // END TODO: handle event RESIZE

        if let Some(p) = self.position {
            if self.tank.in_work() {
                self.tank.transfer_fuel(since_last);
                // TODO: handle event RESIZE
                self.position = Some(start);
                return;
            }
            let speedup = speed * (self.distance / p.distance(target)) / 3.0;
            let mut new_p = p.lerp(target, speed + speedup);
            if p.distance(new_p) > p.distance(target) {
                trace!("Swap target for vehicle");
                std::mem::swap(&mut self.from, &mut self.to);
                if self.to == self.mine {
                    self.unloading();
                } else {
                    self.loading();
                }
                new_p = target;
            }
            self.position = Some(new_p);
        } else  {
            self.position = Some(start);
        }
    }

    pub fn loading(&mut self) {
        self.tank.load();
    }
    pub fn unloading(&mut self) {
        self.tank.unload()
    }
}
