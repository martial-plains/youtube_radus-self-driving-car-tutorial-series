use std::{
    format,
    ops::{Add, Mul, Sub},
};

use rand::{thread_rng, Rng};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CoordWithOffset {
    pub x: f64,
    pub y: f64,
    pub offset: f64,
}

pub fn lerp<T>(a: T, b: T, t: T) -> T
where
    T: Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Clone + Copy,
{
    a + (b - a) * t
}

pub fn get_intersection(a: Coord, b: Coord, c: Coord, d: Coord) -> Option<CoordWithOffset> {
    let t_top = (d.x - c.x) * (a.y - c.y) - (d.y - c.y) * (a.x - c.x);
    let u_top = (c.y - a.y) * (a.x - b.x) - (c.x - a.x) * (a.y - b.y);
    let bottom = (d.y - c.y) * (b.x - a.x) - (d.x - c.x) * (b.y - a.y);

    if bottom != 0.0 {
        let t = t_top / bottom;
        let u = u_top / bottom;

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            return Some(CoordWithOffset {
                x: lerp(a.x, b.x, t),
                y: lerp(a.y, b.y, t),
                offset: t,
            });
        }
    }

    None
}

pub fn get_rgba(value: i32) -> String {
    let alpha: i32 = value.abs();
    let r = if value < 0 { 0 } else { 255 };
    let g = r;
    let b = if value > 0 { 0 } else { 255 };

    format!("rgba({r},{g},{b},{alpha})")
}

pub fn get_random_color() -> String {
    let mut rng = thread_rng();
    let hue = 290 + rng.gen_range(0..260);
    format!("hsl({hue}, 100%, 60%)")
}

pub fn polys_intersect(poly1: &[Coord], poly2: &[Coord]) -> bool {
    for i in 0..poly1.len() {
        for j in 0..poly2.len() {
            let touch = get_intersection(
                poly1[i],
                poly1[(i + 1) % poly1.len()],
                poly2[j],
                poly2[(j + 1) % poly2.len()],
            );

            if touch.is_some() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::lerp;

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_lerp() {
        let expected = 0.7853981633974483;
        let actual = lerp(
            1.5707963267948966 / 2.0,
            -1.5707963267948966 / 2.0,
            0.0 / (5.0 - 1.0),
        ) + 0.0;

        assert_eq!(expected, actual)
    }
}
