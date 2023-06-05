use web_sys::CanvasRenderingContext2d;

use crate::controls::{Controls, ControlsPtr};

pub type CarPtr = Car;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Car {
    pub x: f64,
    pub y: f64,
    pub width: f32,
    pub height: f32,
    pub speed: f64,
    pub acceleration: f64,
    pub max_speed: f64,
    pub friction: f64,
    pub angle: f64,
    pub controls: ControlsPtr,
}

impl Car {
    pub fn new(x: f64, y: f64, width: f32, height: f32) -> Car {
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
            ..Default::default()
        };

        this.controls = Controls::new();

        this
    }

    pub fn update(&mut self) {
        self.r#move()
    }

    pub fn draw(&mut self, ctx: &CanvasRenderingContext2d) {
        ctx.save();
        ctx.translate(self.x, self.y).unwrap();
        ctx.rotate(-self.angle).unwrap();
        ctx.begin_path();
        ctx.rect(
            -(self.width as f64) / 2.0,
            -(self.height as f64) / 2.0,
            self.width as f64,
            self.height as f64,
        );
        ctx.fill();
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
}
