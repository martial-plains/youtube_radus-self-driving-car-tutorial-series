use web_sys::CanvasRenderingContext2d;

use crate::{
    controls::{Controls, ControlsPtr},
    sensor::Sensor,
    utils::{polys_intersect, Coord},
};

pub type CarPtr = Car;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Car {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub speed: f64,
    pub acceleration: f64,
    pub max_speed: f64,
    pub friction: f64,
    pub angle: f64,
    pub damaged: bool,
    pub controls: ControlsPtr,
    pub sensor: Sensor,
    pub polygon: Vec<Coord>,
}

impl Car {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Car {
        let max_speed = 3.0;
        let mut this = Car {
            x,
            y,
            width,
            height,
            speed: 0.0,
            acceleration: 0.2,
            max_speed,
            friction: 0.05,
            angle: 0.0,
            sensor: Sensor::new(),
            ..Default::default()
        };

        this.controls = Controls::new();

        this
    }

    pub fn update(&mut self, road_borders: &[Vec<Coord>]) {
        if !self.damaged {
            self.r#move();
            self.polygon = self.create_polygon();
            self.damaged = self.assess_damage(road_borders)
        }

        self.sensor.update(self.clone(), road_borders)
    }

    pub fn draw(&mut self, ctx: &CanvasRenderingContext2d) {
        ctx.save();
        ctx.translate(self.x, self.y).unwrap();
        ctx.rotate(-self.angle).unwrap();
        ctx.begin_path();
        ctx.rect(
            -self.width / 2.0,
            -self.height / 2.0,
            self.width,
            self.height,
        );
        ctx.fill();
        ctx.restore();
        self.sensor.draw(ctx);
    }

    fn r#move(&mut self) {
        if self.controls.borrow_mut().forward {
            self.speed += self.acceleration;
        }

        if self.controls.borrow_mut().reverse {
            self.speed -= self.acceleration;
        }

        if self.speed > self.max_speed {
            self.speed = self.max_speed;
        }

        if self.speed < -self.max_speed / 2.0 {
            self.speed = -self.max_speed / 2.0;
        }

        if self.speed > 0.0 {
            self.speed -= self.friction;
        }

        if self.speed < 0.0 {
            self.speed += self.friction;
        }

        if self.speed.abs() < self.friction {
            self.speed = 0.0;
        }

        if self.speed != 0.0 {
            let flip = if self.speed > 0.0 { 1.0 } else { -1.0 };

            if self.controls.borrow_mut().left {
                self.angle += 0.03 * flip;
            }
            if self.controls.borrow_mut().right {
                self.angle -= 0.03 * flip;
            }
        }

        self.x -= self.angle.sin() * self.speed;
        self.y -= self.angle.cos() * self.speed;
    }

    fn create_polygon(&self) -> Vec<Coord> {
        let mut points = Vec::new();
        let width = self.width;
        let height = self.height;
        let rad = width.hypot(height) / 2.0;
        let alpha = width.atan2(height);
        points.push(Coord {
            x: self.x - (self.angle - alpha).sin() * rad,
            y: self.y - (self.angle - alpha).cos() * rad,
        });
        points.push(Coord {
            x: self.x - (self.angle + alpha).sin() * rad,
            y: self.y - (self.angle + alpha).cos() * rad,
        });
        points.push(Coord {
            x: self.x - (self.angle + alpha).sin() * rad,
            y: self.y - (self.angle + alpha).cos() * rad,
        });

        points
    }

    fn assess_damage(&self, road_borders: &[Vec<Coord>]) -> bool {
        for road_border in road_borders {
            if polys_intersect(&self.polygon, road_border) {
                return true;
            }
        }

        false
    }
}
