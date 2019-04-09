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
    mine: widget::Id,
    pub wh: Point2,
    from: RoutePoint, 
    to:  RoutePoint,
    tank: Tank,
    position: Option<Point2>,
    distance: f32,
    need_resize: bool,
}

impl Vehicle {
    pub fn new(ids: Ids) -> Self {
        Vehicle {
            mine: ids.mine,
            wh: pt2(50.0, 20.0),
            from: RoutePoint::new(ids.mine),
            to: RoutePoint::new(ids.stations[0]),
            tank: Tank::new(),
            position: None,
            distance: 0.0,
            need_resize: true,
        }
    }

    fn background(&self) -> Rgba {
        if (self.to.id == self.mine) && !(self.tank.in_work()) {
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

    fn route_to(&self, id: widget::Id, r: ui::Rect) -> Point2 {
        if id == self.mine {
            pt2(r.x.end as f32 + self.wh.x / 2.0, r.y.start as f32 + self.wh.y / 2.0)
        } else {
            pt2(r.x.end as f32 - self.wh.x / 2.0, r.y.start as f32 - self.wh.y)
        }
    }

    fn update_route(&mut self, ui: &mut UiCell) {
        let r = ui.rect_of(self.from.id).unwrap();
        self.from.p = self.route_to(self.from.id, r);

        let r = ui.rect_of(self.to.id).unwrap();
        self.to.p = self.route_to(self.to.id, r);

        self.distance = self.from.p.distance(self.to.p);
        if self.position.is_some() && self.tank.in_work() {
            self.position = Some(self.from.p);
        }
        self.need_resize = false;
    }

    pub fn update(&mut self, ui: &mut UiCell, speed: f32, since_last: f64) {
        if self.need_resize {
            self.update_route(ui);
        }

        if let Some(p) = self.position {
            if self.tank.in_work() {
                self.tank.transfer_fuel(since_last);
                return;
            }
            let speedup = speed * (self.distance / p.distance(self.to.p)) / 3.0;
            let mut new_p = p.lerp(self.to.p, speed + speedup);
            if p.distance(new_p) > p.distance(self.to.p) {
                trace!("Swap target for vehicle");
                new_p = self.to.p; // complete move
                std::mem::swap(&mut self.from, &mut self.to);
                if self.to.id == self.mine {
                    self.tank.unload()
                } else {
                    self.tank.load();
                }
            }
            self.position = Some(new_p);
        } else  {
            self.position = Some(self.from.p);
        }
    }

    pub fn resize(&mut self) {
        self.need_resize = true;
    }
}
