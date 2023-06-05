use gloo::console::console_dbg;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::{
    car::Car,
    utils::{get_intersection, lerp, Coord, CoordWithOffset},
};

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Sensor {
    pub ray_count: usize,
    pub ray_length: usize,
    pub ray_spread: f64,
    pub rays: Vec<(Coord, Coord)>,
    pub readings: Vec<Option<CoordWithOffset>>,
}

impl Sensor {
    pub fn new() -> Self {
        Self {
            ray_count: 5,
            ray_length: 150,
            ray_spread: std::f64::consts::PI / 2.0,
            rays: Vec::new(),
            readings: Vec::new(),
        }
    }

    pub fn update(&mut self, car: Car, road_borders: &[Vec<Coord>]) {
        self.cast_rays(car);
        self.readings = Vec::new();
        for i in 0..self.rays.len() {
            self.readings
                .push(Self::get_reading(self.rays[i], road_borders));
        }
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d) {
        for i in 0..self.ray_count {
            let mut end = CoordWithOffset {
                x: self.rays[i].1.x,
                y: self.rays[i].1.y,
                ..Default::default()
            };
            if let Some(reading) = self.readings[i] {
                end = reading
            }

            ctx.begin_path();
            ctx.set_line_width(2.0);
            ctx.set_stroke_style(&JsValue::from_str("yellow"));
            ctx.move_to(self.rays[i].0.x, self.rays[i].0.y);
            ctx.line_to(end.x, end.y);
            ctx.stroke();
            ctx.begin_path();
            ctx.set_line_width(2.0);
            ctx.set_stroke_style(&JsValue::from_str("black"));
            ctx.move_to(self.rays[i].1.x, self.rays[i].1.y);
            ctx.line_to(end.x, end.y);
            ctx.stroke();
        }
    }

    fn get_reading(ray: (Coord, Coord), road_borders: &[Vec<Coord>]) -> Option<CoordWithOffset> {
        let mut touches = Vec::new();

        for border in road_borders {
            let touch = get_intersection(ray.0, ray.1, border[0], border[1]);

            if let Some(touch) = touch {
                touches.push(touch);
            }
        }

        if touches.is_empty() {
            None
        } else {
            let offsets = touches.iter().map(|e| e.offset).collect::<Vec<f64>>();
            let min_offset = offsets
                .iter()
                .min_by(|x, y| x.partial_cmp(y).unwrap())
                .copied()
                .unwrap();
            touches.iter().find(|e| e.offset == min_offset).copied()
        }
    }

    fn cast_rays(&mut self, car: Car) {
        self.rays = Vec::new();

        for i in 0..self.ray_count {
            let ray_angle = lerp(
                self.ray_spread / 2.0,
                -self.ray_spread / 2.0,
                if self.ray_count == 1 {
                    0.5
                } else {
                    i as f64 / (self.ray_count - 1) as f64
                },
            ) + car.angle;

            let start = Coord { x: car.x, y: car.y };
            let end = Coord {
                x: (car.x - ray_angle.sin() * self.ray_length as f64),
                y: (car.y - ray_angle.cos() * self.ray_length as f64),
            };

            self.rays.push((start, end));
        }
    }
}
