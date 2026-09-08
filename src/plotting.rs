use spherical_coords::SphericalCoords;
use crate::spherical_coords;
use crate::radians;
use crate::radians::{cos, cot, sin, tan, Radians};

pub use spherical_plotter::SphericalPlotter;
mod spherical_plotter {
    use crate::plotting::canvas_pos::{cpos, CanvasPos};
    use crate::plotting::{angle_to_pos, circle_bezier, pos_to_angle, scale_from_line, spherical_cross_product, CanvasPainter, Color, CubicBezier};
    use crate::radians::Radians;
    use crate::spherical_coords::SphericalCoords;
    use std::mem;

    pub struct SphericalPlotter<P> where P: CanvasPainter {
        painter: P, sphere_radius: f64, focus: SphericalCoords,
    }
    impl<Painter> SphericalPlotter<Painter> where Painter: CanvasPainter {
        pub fn new(painter: Painter, sphere_radius: f64, focus: SphericalCoords) -> Self {
            Self { painter, sphere_radius, focus }
        }
        pub fn destruct(self) -> Painter { self.painter }
        pub fn painter(&self) -> &Painter { &self.painter }
        pub fn sphere_radius(&self) -> f64 { self.sphere_radius }
        pub fn focus(&self) -> SphericalCoords { self.focus }


        ///
        /// Returns the screen position of the given coordinates,
        /// even if the point is hidden by the sphere.
        ///
        /// See also: [`coords_from_pos2`], [`pos2_from_coords`]
        ///
        pub fn pos_from_coords_unchecked(&self, coords: SphericalCoords) -> CanvasPos {
            let (x, _y, z) = coords
                .inverse_translate(self.focus().into())
                .scale(self.sphere_radius() as f64)
                .to_cartesian();

            cpos(z, x)
        }
        ///
        /// Returns the screen position of the given coordinates,
        /// unless the point would be hidden by the sphere.
        ///
        /// See also: [`coords_from_pos2`], [`pos2_from_coords_unchecked`]
        ///
        pub fn pos_from_coords(&self, coords: SphericalCoords) -> Option<CanvasPos> {
            let r = self.sphere_radius() as f64;
            let (x, y, z) = coords
                .inverse_translate(self.focus().into())
                .scale(r)
                .to_cartesian();

            if y < 0.0 { None } else { Some(cpos(z, x)) }
        }


        ///
        /// Finds the spherical coordinates of the given screen position,
        /// with the point assumed to be on the visible side of the sphere.
        ///
        /// See also: [`pos2_from_coords`]
        ///
        pub fn coords_from_pos(&self, pos: CanvasPos) -> Option<SphericalCoords> {
            let r = self.sphere_radius() as  f64;
            let x = pos.y;
            let z = pos.x;
            let y = (r * r - pos.length_sq()).sqrt();
            if y.is_nan() { return None; }

            let coords = SphericalCoords::from_cartesian((x, y, z))
                .to_unit()
                .translate(self.focus().into());
            Some(coords)
        }
        fn plot_dot_if_visible(&self, coords: SphericalCoords, color: Color) {
            let pos = self.pos_from_coords(coords);
            if let Some(pos) = pos { self.painter().paint_dot(pos, color); }
        }
        ///
        /// Plots a straight line from the origin to the edge of the sphere, at the given angle.
        ///
        fn plot_line_from_angle(&self, theta: Radians, color: Color) {
            let r = self.sphere_radius();
            let pos = angle_to_pos(theta, 1.0);
            self.painter().paint_line(CanvasPos::ORIGIN, pos * r, color);
        }
        ///
        /// Plots an arc between two spherical coordinates.
        ///
        //noinspection DuplicatedCode
        pub fn plot_arc(&self, p: SphericalCoords, q: SphericalCoords, color: Color) {
            let pos_p = self.pos_from_coords_unchecked(p);
            let pos_q = self.pos_from_coords_unchecked(q);
            if pos_p == pos_q { return; }

            let p_visible = self.pos_from_coords(p).is_some();
            let q_visible = self.pos_from_coords(q).is_some();
            if !(p_visible || q_visible) { return; }

            // find normal
            let coords_normal = spherical_cross_product(p, q);

            // find semi-major axis
            let coords_major = spherical_cross_product(coords_normal, self.focus());
            let pos_major = self.pos_from_coords_unchecked(coords_major);

            // find semi-minor axis
            let coords_minor = spherical_cross_product(coords_normal, coords_major);
            let pos_minor = self.pos_from_coords_unchecked(coords_minor);

            // account for arc through origin
            if pos_minor == CanvasPos::ORIGIN { // || pos_p == Pos2::ZERO || pos_q == Pos2::ZERO {
                let mut start = pos_p;
                let mut end = pos_q;
                if !p_visible {
                    let g = if start == CanvasPos::ORIGIN { end } else { start };
                    start = g * g.length().recip();
                }
                if !q_visible {
                    let g = if end == CanvasPos::ORIGIN { start } else { end };
                    end = g * g.length().recip();
                }
                self.painter().paint_line(start, end, color);
                return;
            }

            // find ellipse parameters
            let theta = pos_to_angle(pos_major).unwrap();
            let scale = pos_minor.length() / self.sphere_radius();
            let scale_inv = scale.recip();

            // map points to circle
            let circle_pos_p = scale_from_line(pos_p, theta, scale_inv);
            let circle_pos_q = scale_from_line(pos_q, theta, scale_inv);

            // find arc angles
            let mut start = pos_to_angle(circle_pos_p).unwrap();
            let mut end = pos_to_angle(circle_pos_q).unwrap();
            let mut start_visible = p_visible;
            let mut end_visible = q_visible;
            if (end - start) < Radians::ZERO {
                mem::swap(&mut start, &mut end);
                mem::swap(&mut start_visible, &mut end_visible);
            }

            // clip arc if it passes behind the sphere
            if !(p_visible && q_visible) {
                let mut clip = theta;
                if (theta - start) < Radians::ZERO { clip += Radians::HALF_TAU; }

                if end_visible { start = clip; } else { end = clip; }
            }

            // find circular arc
            let circular_arcs = circle_bezier(self.sphere_radius(), start, end);

            // map arc to ellipse
            let transformation = |pos|
                scale_from_line(pos, theta, scale);
            let elliptical_arcs: Vec<CubicBezier> = circular_arcs
                .iter()
                .map(|arc| arc.transform(transformation))
                .collect::<Vec<_>>();

            // draw arcs
            for arc in elliptical_arcs {
                self.painter.paint_cubic_bezier(arc, color);
            }
        }
        ///
        /// Plots a polygon using spherical coordinates.
        ///
        pub fn plot_polygon(&self, coords: &Vec<SphericalCoords>, color: Color) {
            let len = coords.len();
            for &c in coords { self.plot_dot_if_visible(c, color); }
            if len < 3 { return; }

            for i in 0..len {
                let c1 = coords[i];
                let c2 = coords[(i + 1) % len];
                self.plot_arc(c1, c2, color);
            }
        }

        // TODO: remove this
        pub fn plot_debug(&self, coords: &Vec<SphericalCoords>, color: Color) {
            let len = coords.len();
            for &c in coords { self.plot_dot_if_visible(c, color); }
            if len < 2 { return; }

            self.plot_arc(coords[0], coords[1], color);
        }
    }
}

pub fn pos_to_angle(pos: CanvasPos) -> Result<Radians, String> {
    radians::atan2(pos.y, pos.x)
}
pub fn angle_to_pos(angle: Radians, distance: f64) -> CanvasPos {
    cpos(cos(angle), sin(angle)) * distance
}

///
/// Performs a stretch perpendicular to the specified line.
///
pub fn scale_from_line(point: CanvasPos, line_angle: Radians, scale: f64) -> CanvasPos {
    let theta = line_angle;
    let y_delta = point.y - point.x * tan(theta);
    let pos = angle_to_pos(theta + Radians::QUARTER_TAU, 1.0);
    point + (scale - 1.0) * y_delta * cos(theta) * pos
}
///
/// Approximates a circular arc centered on the origin using a cubic Bézier curve.
///
/// See also: [`circle_bezier_one`]
///
pub fn circle_bezier(r: f64, mut start: Radians, end: Radians) -> Vec<CubicBezier> {
    let mut results = vec![];

    // Splits arc if angle covers more than tau/4
    for _ in 0..4 {
        if end - start <= Radians::QUARTER_TAU { break; }
        let old_start = start;
        start += Radians::QUARTER_TAU;

        results.push(circle_bezier_one(r, old_start, start));
    }

    assert!(end - start <= Radians::QUARTER_TAU);
    results.push(circle_bezier_one(r, start, end));

    results
}
///
/// Approximates a circular arc centered on the origin using a cubic Bézier curve.
/// The arc must span no more than tau/4 radians.
///
/// If you need to make a larger arc, use [`circle_bezier`]
///
//noinspection SpellCheckingInspection
pub fn circle_bezier_one(r: f64, start: Radians, end: Radians) -> CubicBezier {
    // Code snippet by dataphract
    // https://github.com/emilk/egui/issues/4188#issuecomment-2060387851

    let p1 = r * angle_to_pos(start, 1.0);
    let p4 = r * angle_to_pos(end, 1.0);

    let a = p1;
    let b = p4;
    let q1 = a.length_sq();
    let q2 = q1 + a.dot(b);
    let k2 = (4.0 / 3.0) * ((2.0 * q1 * q2).sqrt() - q2) / (a.x * b.y - a.y * b.x);

    let p2 = cpos(a.x - k2 * a.y, a.y + k2 * a.x);
    let p3 = cpos(b.x + k2 * b.y, b.y - k2 * b.x);

    CubicBezier::new([p1, p2, p3, p4])
}
///
/// Calculates the cross product of two spherical coordinates.
///
/// Panics if the coordinates are equal, or exact mirrors of each other.
///
pub fn spherical_cross_product(a: SphericalCoords, b: SphericalCoords) -> SphericalCoords {
    if a.inc() == b.inc() && a.azi() == b.azi() {
        panic!("Cannot calculate the cross product of two equal coordinates!")
    }
    if a.inc() + b.inc() == Radians::HALF_TAU && a.azi() + b.azi() == Radians::ZERO {
        panic!("Cannot calculate the cross product of two mirror coordinates!")
    }

    if a.inc() == Radians::ZERO || a.inc() == Radians::HALF_TAU {
        if a.inc() == b.inc() {
            panic!("Cannot calculate the cross product of two equal coordinates!")
        }
        if a.inc() + b.inc() == Radians::HALF_TAU {
            panic!("Cannot calculate the cross product of two mirror coordinates!")
        }

        return SphericalCoords::unit(Radians::QUARTER_TAU, b.azi() + Radians::QUARTER_TAU);
    }
    if a.inc() == Radians::QUARTER_TAU {
        if b.inc() == Radians::QUARTER_TAU {
            return SphericalCoords::zenith()
        }
        spherical_cross_product(b, a);
    }

    // Simplified from:
    // let spun = b.inverse_translate(a.into());
    // let norm = SphericalCoords::unit(
    //     Radians::QUARTER_TAU, spun.azi() + Radians::QUARTER_TAU);
    // norm.translate(a.into())

    let theta = a.inc();
    let inc = b.inc();
    let azi = b.azi() - a.azi();

    // Make relative to a
    let x_ish = cot(inc) * sin(-theta) + cos(azi) * cos(-theta);
    let y_ish = sin(azi);

    let new_azi = radians::atan2(y_ish, x_ish).unwrap();

    // Return to normal coords
    let x = -sin(new_azi) * cos(theta);
    let y =  cos(new_azi);
    let z =  sin(new_azi) * sin(theta);

    let new_azi = radians::atan2(y, x).unwrap();
    let new_inc = radians::acos(z);

    SphericalCoords::unit(new_inc, new_azi + a.azi())
}


pub use canvas_pos::{cpos, CanvasPos};
mod canvas_pos {
    use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

    pub const fn cpos(x: f64, y: f64) -> CanvasPos { CanvasPos::new(x, y) }
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct CanvasPos {
        pub x: f64,
        pub y: f64
    }
    impl CanvasPos {
        pub const ORIGIN: Self = Self::new(0.0, 0.0);
        pub const fn new(x: f64, y: f64) -> Self { Self { x, y, } }
        pub fn length(&self) -> f64 { f64::hypot(self.x, self.y) }
        pub fn length_sq(&self) -> f64 { self.x * self.x + self.y * self.y }
        pub fn distance(&self, other: Self) -> f64 { (other - *self).length()  }
        pub fn dot(&self, other: Self) -> f64 {
            self.x * other.x + self.y * other.y
        }
    }
    impl Add for CanvasPos {
        type Output = CanvasPos;
        fn add(self, other: CanvasPos) -> CanvasPos {
            Self::new(self.x + other.x, self.y + other.y)
        }
    }
    impl AddAssign for CanvasPos {
        fn add_assign(&mut self, other: CanvasPos) { *self = *self + other; }
    }
    impl Sub for CanvasPos {
        type Output = CanvasPos;
        fn sub(self, other: CanvasPos) -> CanvasPos {
            Self::new(self.x - other.x, self.y - other.y)
        }
    }
    impl SubAssign for CanvasPos {
        fn sub_assign(&mut self, other: CanvasPos) { *self = *self - other; }
    }
    impl Mul<f64> for CanvasPos {
        type Output = CanvasPos;
        fn mul(self, other: f64) -> CanvasPos {
            Self::new(self.x * other, self.y * other)
        }
    }
    impl Mul<CanvasPos> for f64 {
        type Output = CanvasPos;
        fn mul(self, other: CanvasPos) -> CanvasPos { other * self }
    }
    impl Neg for CanvasPos {
        type Output = CanvasPos;
        fn neg(self) -> CanvasPos { CanvasPos::new(-self.x, -self.y) }
    }
}

pub type QuadraticBezier = BezierCurve<3>;
pub type CubicBezier = BezierCurve<4>;
#[derive(Copy, Clone, Debug)]
pub struct BezierCurve<const TERMS: usize> {
    points: [CanvasPos; TERMS]
}
impl<const TERMS: usize> BezierCurve<TERMS> {
    pub fn new(points: [CanvasPos; TERMS]) -> Self { Self { points } }
    pub fn destruct(self) -> [CanvasPos; TERMS] { self.points }
    pub fn transform<T>(&self, transformation: T) -> Self
    where T: Fn(CanvasPos) -> CanvasPos {
        Self::new(self.points.map(transformation))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    values: [u8; 4]
}
impl Color {
    pub fn new(values: [u8; 4]) -> Self { Self { values } }
    pub fn from_rbg(r: u8, g: u8, b: u8) -> Self { Self::new([r, g, b, 0xFF]) }
    pub fn values(&self) -> &[u8; 4] { &self.values }
    pub fn r(&self) -> u8 { self.values[0] }
    pub fn g(&self) -> u8 { self.values[1] }
    pub fn b(&self) -> u8 { self.values[2] }
    pub fn a(&self) -> u8 { self.values[3] }
}

pub trait CanvasPainter {
    fn paint_dot(&self, pos: CanvasPos, color: Color);
    fn paint_line(&self, start: CanvasPos, end: CanvasPos, color: Color);
    fn paint_quadratic_bezier(&self, bezier: QuadraticBezier, color32: Color);
    fn paint_cubic_bezier(&self, bezier: CubicBezier, color32: Color);
}
