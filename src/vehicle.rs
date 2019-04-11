use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou::Draw;
use crate::model::Ids;
use crate::tank::*;

#[derive(PartialEq)]
pub struct RoutePoint {
    id: widget::Id,
    p: Point2,
}

impl RoutePoint {
    fn new(id: widget::Id) -> Self {
        RoutePoint{
            id,
            p: pt2(0.0, 0.0),
        }
    }
}

pub struct Vehicle {
    pub wh: Point2,
    mine: RoutePoint,
    station:  RoutePoint,
    ids: Ids,
    tank: Tank,
    position: Option<Point2>,
    pub need_resize: bool,
}

impl Vehicle {
    pub fn new(ids: Ids) -> Self {
        Vehicle {
            wh: pt2(50.0, 20.0),
            mine: RoutePoint::new(ids.mine),
            station: RoutePoint::new(ids.stations[0]),
            ids: ids,
            tank: Tank::new(),
            position: None,
            need_resize: true,
        }
    }

    pub fn draw(&self, draw: &Draw) {
        if let Some(pos) = self.position {
            draw.rect()
                .wh(self.wh)
                .color(GREEN)
                .xy(pos);
                let transfer_wh = pt2(self.wh.x, self.wh.y * (1.0 - self.fuel_percent() / 100.0));
                draw.rect()
                    .wh(transfer_wh)
                    .color(WHITE)
                    .x_y(pos.x, pos.y + (self.wh.y/2.0 - transfer_wh.y/2.0));

        }
    }

    pub fn update_route(&mut self, ui: &mut UiCell, state: TankState) {
        if let Some(r) = ui.rect_of(self.ids.mine) {
            self.mine.p = pt2(r.x.end as f32 + self.wh.x / 2.0,
                              r.y.start as f32 + self.wh.y / 2.0)
        }

        if let Some(r) = ui.rect_of(self.station.id) {
            self.station.p = pt2(r.x.end as f32 - self.wh.x / 2.0,
                                 r.y.start as f32 - self.wh.y)
        }

        match state {
            TankState::Load(_) => {
                self.position = Some(self.mine.p);
            },
            TankState::Unload(_) => {
                self.position = Some(self.station.p);
            }
            _ => (),
        }
        self.need_resize = false;
    }

    pub fn update(&mut self, ui: &mut UiCell, speed: f32) {
        if let Some(id) = self.current_station() {
            self.station.id = id;
        }
        let state = self.tank.get_state();
        self.update_route(ui, state);

        if let Some(p) = self.position {
            let (from, to) = match state {
                TankState::Load(_) | TankState::Unload(_) => return,
                TankState::Supply(_) => (&self.mine, &self.station),
                TankState::Refill(_) => (&self.station, &self.mine),
            };
            let dist = from.p.distance(to.p);

            let speedup = speed * (dist / p.distance(to.p)) / 3.0;
            let mut new_p = p.lerp(to.p, speed + speedup);
            if p.distance(new_p) >= p.distance(to.p) {
                new_p = to.p; // complete move
                match state {
                    TankState::Refill(_) => self.tank.load(),
                    TankState::Supply(_) => self.tank.unload(),
                    _ => (),
                };
            }
            self.position = Some(new_p);
        } else  {
            self.position = Some(self.mine.p);
        }
    }

    fn fuel_percent(&self) -> f32 {
        match self.tank.get_state() {
            TankState::Load(l) |
                TankState::Unload(l) |
                TankState::Supply(l) |
                TankState::Refill(l) => l,
        }
    }

    fn current_station(&self) -> Option<widget::Id> {
        match self.tank.get_target() {
            Some(idx) => Some(self.ids.stations[idx]),
            None => None,
        }
    }

    pub fn resize(&mut self) {
        self.need_resize = true;
    }
}
