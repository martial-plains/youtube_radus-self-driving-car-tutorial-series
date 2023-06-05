use gloo::console::console_dbg;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    controls::{ControlKind, Controls, ControlsPtr},
    network::NeuralNetwork,
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
    pub use_brain: bool,
    pub controls: ControlsPtr,
    pub sensor: Option<Sensor>,
    pub brain: Option<NeuralNetwork>,
    pub polygon: Vec<Coord>,
}

impl Car {
    pub fn new(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        control_kind: ControlKind,
        max_speed: Option<f64>,
    ) -> Car {
        let max_speed = max_speed.unwrap_or(3.0);
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
            use_brain: control_kind == ControlKind::AI,
            ..Default::default()
        };

        if !matches!(control_kind, ControlKind::Dummy) {
            this.sensor = Some(Sensor::new());
            if let Some(sensor) = &mut this.sensor {
                this.brain = Some(NeuralNetwork::new(vec![sensor.ray_count, 6, 4]))
            }
        }

        this.controls = Controls::new(control_kind);

        this
    }

    pub fn update(&mut self, road_borders: &[Vec<Coord>], traffic: &[CarPtr]) {
        if !self.damaged {
            self.r#move();
            self.polygon = self.create_polygon();
            self.damaged = self.assess_damage(road_borders, traffic)
        }
        let car = self.clone();
        if let Some(sensor) = &mut self.sensor {
            sensor.update(&car, road_borders, traffic);
            let offsets = sensor
                .readings
                .iter()
                .map(|s| match s {
                    Some(s) => 1.0 - s.offset,
                    None => 0.0,
                })
                .collect::<Vec<f64>>();
            if let Some(brain) = &mut self.brain {
                let outputs = brain.feed_forward(offsets);

                if self.use_brain {
                    self.controls.borrow_mut().forward = outputs[0] == 1.0;
                    self.controls.borrow_mut().left = outputs[1] == 1.0;
                    self.controls.borrow_mut().right = outputs[2] == 1.0;
                    self.controls.borrow_mut().reverse = outputs[3] == 1.0;
                }
            }
        }
    }

    pub fn draw(&mut self, ctx: &CanvasRenderingContext2d, color: &str) {
        if self.damaged {
            ctx.set_fill_style(&JsValue::from_str("gray"));
        } else {
            ctx.set_fill_style(&JsValue::from_str(color));
        }
        ctx.begin_path();
        ctx.move_to(self.polygon[0].x, self.polygon[0].y);
        for i in 1..self.polygon.len() {
            ctx.line_to(self.polygon[i].x, self.polygon[i].y);
        }

        ctx.fill();

        if let Some(sensor) = &mut self.sensor {
            sensor.draw(ctx);
        }
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
        let rad = self.width.hypot(self.height) / 2.0;
        let alpha = self.width.atan2(self.height);
        points.push(Coord {
            x: self.x - (self.angle - alpha).sin() * rad,
            y: self.y - (self.angle - alpha).cos() * rad,
        });
        points.push(Coord {
            x: self.x - (self.angle + alpha).sin() * rad,
            y: self.y - (self.angle + alpha).cos() * rad,
        });
        points.push(Coord {
            x: self.x - (std::f64::consts::PI + self.angle - alpha).sin() * rad,
            y: self.y - (std::f64::consts::PI + self.angle - alpha).cos() * rad,
        });
        points.push(Coord {
            x: self.x - (std::f64::consts::PI + self.angle + alpha).sin() * rad,
            y: self.y - (std::f64::consts::PI + self.angle + alpha).cos() * rad,
        });

        points
    }

    fn assess_damage(&self, road_borders: &[Vec<Coord>], traffic: &[CarPtr]) -> bool {
        for road_border in road_borders {
            if polys_intersect(&self.polygon, road_border) {
                return true;
            }
        }

        for t in traffic {
            if polys_intersect(&self.polygon, &t.polygon) {
                return true;
            }
        }

        false
    }
}
