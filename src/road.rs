use std::vec;

use js_sys::Array;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::utils::{lerp, Coord};

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Road {
    pub x: f64,
    pub width: f64,
    pub lane_count: usize,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    pub borders: Vec<Vec<Coord>>,
}

impl Road {
    pub fn new(x: f64, width: f64, lane_count: Option<usize>) -> Self {
        let lane_count = lane_count.unwrap_or(3);
        let left = x - width / 2.0;
        let right = x + width / 2.0;
        let infinity = 1_000_000.0;
        let top = -infinity;
        let bottom = infinity;

        let top_left = Coord { x: left, y: top };
        let top_right = Coord { x: right, y: top };
        let bottom_left = Coord { x: left, y: bottom };
        let bottom_right = Coord {
            x: right,
            y: bottom,
        };

        let borders = vec![vec![top_left, bottom_left], vec![top_right, bottom_right]];

        Self {
            x,
            width,
            lane_count,
            left,
            right,
            top,
            bottom,
            borders,
        }
    }

    pub fn get_late_center(&self, lane_index: usize) -> f64 {
        let lane_width = self.width / self.lane_count as f64;
        self.left + lane_width / 2.0 + lane_index.min(self.lane_count - 1) as f64 * lane_width
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d) {
        ctx.set_line_width(5.0);
        ctx.set_stroke_style(&JsValue::from_str("white"));

        for i in 1..=self.lane_count - 1 {
            let x = lerp(self.left, self.right, i as f64 / self.lane_count as f64);

            let array = Array::new();
            array.push(&JsValue::from(20));
            array.push(&JsValue::from(20));

            ctx.set_line_dash(&array).unwrap();
            ctx.begin_path();
            ctx.move_to(x, self.top);
            ctx.line_to(x, self.bottom);
            ctx.stroke();
        }

        ctx.set_line_dash(&Array::new()).unwrap();
        for border in &self.borders {
            ctx.begin_path();
            ctx.move_to(border[0].x, border[0].y);
            ctx.line_to(border[1].x, border[1].y);
            ctx.stroke();
        }
    }
}
