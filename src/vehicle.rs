use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou::Draw;
use crate::model::Ids;

pub struct Vehicle {
    mine: widget::Id,
    wh: Point2,
    from: widget::Id,
    to: widget::Id,
    direction: Point2,
    position: Option<Point2>,
}

impl Vehicle {
    pub fn new(ids: Ids) -> Self {
        Vehicle {
            mine: ids.mine,
            wh: pt2(50.0, 20.0),
            from: ids.mine,
            to: ids.stations[0],
            direction: pt2(0.0, 0.0),
            position: None,
        }
    }

    pub fn draw(&self, draw: &Draw) {
        draw.rect()
            .wh(self.wh)
            .xy(self.position.unwrap());
    }

    pub fn update(&mut self, ui: &mut UiCell, speed: f32) {
        let points = [self.from, self.to].iter().map(|id|{
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
            let mut new_p = p + self.direction * speed;
            if new_p.distance(target) > p.distance(target) {
                trace!("Swap target for vehicle");
                std::mem::swap(&mut self.from, &mut self.to);
                new_p = target;
            }
            self.position = Some(new_p);
        } else  {
            self.position = Some(start);
        }
    }
}
