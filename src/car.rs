use std::f64::consts::PI;

use gloo::utils::document;
use js_sys::Function;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement, HtmlImageElement};

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
    pub sensor: Option<Sensor>,
    pub brain: Option<NeuralNetwork>,
    pub controls: ControlsPtr,
    pub img: Option<HtmlImageElement>,
    pub mask: Option<Element>,
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
        color: Option<&str>,
    ) -> Car {
        let color = color.unwrap_or("blue");
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

        this.img = HtmlImageElement::new().ok();
        this.img.as_mut().unwrap().set_src("car.png");

        this.mask = document().create_element("canvas").ok();
        let mask_elmt = {
            let elmt = this.mask.as_ref().unwrap();

            elmt.dyn_ref::<HtmlCanvasElement>().unwrap()
        };

        mask_elmt.set_width(width as u32);
        mask_elmt.set_height(height as u32);

        let mask_ctx = mask_elmt
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let onload = {
            let color = color.to_string();
            let img = this.img.clone().unwrap();
            let width = this.width;
            let height = this.height;

            Closure::<dyn Fn()>::new(move || {
                mask_ctx.set_fill_style(&JsValue::from_str(&color));
                mask_ctx.rect(0.0, 0.0, width, height);
                mask_ctx.fill();

                mask_ctx
                    .set_global_composite_operation("destination-atop")
                    .unwrap();
                mask_ctx
                    .draw_image_with_html_image_element_and_dw_and_dh(&img, 0.0, 0.0, width, height)
                    .unwrap();
            })
            .into_js_value()
            .dyn_into::<Function>()
            .unwrap()
        };

        if let Some(img) = &mut this.img {
            img.set_onload(Some(&onload))
        }

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

    pub fn draw(&mut self, ctx: &CanvasRenderingContext2d, draw_sensor: Option<bool>) {
        let draw_sensor = draw_sensor.unwrap_or_default();

        if let Some(sensor) = &self.sensor {
            if draw_sensor {
                sensor.draw(ctx);
            }
        }

        ctx.save();
        ctx.translate(self.x, self.y).unwrap();
        ctx.rotate(-self.angle).unwrap();
        if !self.damaged {
            let mask = self
                .mask
                .as_ref()
                .unwrap()
                .dyn_ref::<HtmlCanvasElement>()
                .unwrap();
            ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                mask,
                -self.width / 2.0,
                -self.height / 2.0,
                self.width,
                self.height,
            )
            .unwrap();
            ctx.set_global_composite_operation("multiply").unwrap();
        }
        ctx.draw_image_with_html_image_element_and_dw_and_dh(
            &self.img.clone().unwrap(),
            -self.width / 2.0,
            -self.height / 2.0,
            self.width,
            self.height,
        )
        .unwrap();

        ctx.restore();
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
            x: self.x - (PI + self.angle - alpha).sin() * rad,
            y: self.y - (PI + self.angle - alpha).cos() * rad,
        });
        points.push(Coord {
            x: self.x - (PI + self.angle + alpha).sin() * rad,
            y: self.y - (PI + self.angle + alpha).cos() * rad,
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
