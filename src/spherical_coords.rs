pub mod radians {
    use std::f64::consts::TAU;
    use std::fmt::Formatter;
    use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

    const fn pos_mod(num: f64, den: f64) -> f64 {
        if num >= 0.0 { num % den }
        else { -((-num) % den) }
    }
    pub const fn normalise(mut value: f64) -> f64 {
        value = pos_mod(value, TAU);
        if value >  std::f64::consts::PI { value -= TAU; }
        if value < -std::f64::consts::PI { value += TAU; }
        value
    }

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
    pub struct Radians { value: f64 }
    impl std::fmt::Display for Radians {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let prec = f.precision().unwrap_or(3);
            write!(f, "{:.p$}r", self.value, p = prec)
        }
    }
    impl Radians {
        pub const ZERO: Radians = Self::new(0.0);
        pub const HALF_TAU: Radians = Self::new(std::f64::consts::PI);
        pub const QUARTER_TAU: Radians = Radians::new(std::f64::consts::FRAC_PI_2);
        pub const TAU: Radians = Self::new(TAU);
        pub const fn new(value: f64) -> Self { Radians { value: normalise(value) } }
        pub const fn tau_fraction(num: i64, den: i64) -> Radians {
            Self::new(TAU * num as f64 / den as f64)
        }

        pub const fn from_degrees(value: f64) -> Self { Self::new(TAU * value * 360_f64.recip()) }
        pub fn to_degrees(&self) -> f64 { self.value * 360_f64 * TAU.recip() }
        pub const fn value(&self) -> f64 { self.value }
        pub fn abs(&self) -> Radians { Radians::new(self.value.abs()) }
    }
    impl Add for Radians {
        type Output = Radians;
        fn add(self, other: Radians) -> Radians { Radians::new(self.value + other.value) }
    }
    impl AddAssign for Radians {
        fn add_assign(&mut self, other: Radians) { *self = *self + other; }
    }
    impl Sub for Radians {
        type Output = Radians;
        fn sub(self, other: Radians) -> Radians { Radians::new(self.value - other.value) }
    }
    impl SubAssign for Radians {
        fn sub_assign(&mut self, other: Radians) { *self = *self - other; }
    }
    impl Neg for Radians {
        type Output = Radians;

        fn neg(self) -> Self::Output { Radians::new(-self.value) }
    }
    impl Mul<f64> for Radians {
        type Output = Radians;
        fn mul(self, rhs: f64) -> Self::Output { Radians::new(self.value * rhs) }
    }
    impl Mul<Radians> for f64 {
        type Output = Radians;
        fn mul(self, rhs: Radians) -> Radians { rhs * self }
    }
    impl MulAssign<f64> for Radians {
        fn mul_assign(&mut self, rhs: f64) {
            *self = *self * rhs;
        }
    }

    pub fn sin(angle: Radians) -> f64 { angle.value.sin() }
    pub fn cos(angle: Radians) -> f64 { angle.value.cos() }
    pub fn tan(angle: Radians) -> f64 { angle.value.tan() }
    pub fn cot(angle: Radians) -> f64 { angle.value.cos() / angle.value.sin() }
    
    pub fn sin32(angle: Radians) -> f32 { sin(angle) as f32 }
    pub fn cos32(angle: Radians) -> f32 { cos(angle) as f32 }
    pub fn tan32(angle: Radians) -> f32 { tan(angle) as f32 }

    pub fn asin(value: f64) -> Radians { Radians::new(value.asin()) }
    pub fn acos(value: f64) -> Radians { Radians::new(value.acos()) }
    pub fn atan(value: f64) -> Radians { Radians::new(value.atan()) }

    pub fn atan2(top: f64, bottom: f64) -> Result<Radians, String> {
        match bottom {
            b if b > 0.0 => Ok(atan(top / b)),
            b if b < 0.0 => Ok(atan(top / b) + Radians::HALF_TAU),
            _ => match top {
                t if t > 0.0 => Ok( Radians::QUARTER_TAU),
                t if t < 0.0 => Ok(-Radians::QUARTER_TAU),
                _ => Err("Cannot find arctan of 0 over 0!".to_string()),
            }
        }
    }
}

use radians::{cos, sin, Radians};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct SphericalCoords {
    radius: f64,
    inclination: Radians,
    azimuth: Radians,
}
impl Display for SphericalCoords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prec = f.precision().unwrap_or(3);
        write!(f, "({:.p$}, {:.p$}, {:.p$})", self.rad(), self.inc(), self.azi(), p = prec)
    }
}
impl SphericalCoords {
    pub fn with_radius(radius: f64, mut inclination: Radians, mut azimuth: Radians) -> Self {
        if inclination < Radians::ZERO { inclination = -inclination; azimuth += Radians::HALF_TAU }
        Self { radius, inclination, azimuth }
    }
    pub fn unit(inclination: Radians, azimuth: Radians) -> Self {
        Self::with_radius(1., inclination, azimuth)
    }
    pub fn zenith() -> Self { Self::unit(Radians::ZERO, Radians::ZERO) }
    
    pub fn degree_string(&self, dp: usize) -> String {
        let inc = self.inclination.to_degrees();
        let azi = self.azimuth.to_degrees();
        format!("({inc:.0$}, {azi:.0$})", dp)
    }

    pub fn rad(&self) -> f64 { self.radius }
    pub fn inc(&self) -> Radians { self.inclination }
    pub fn azi(&self) -> Radians { self.azimuth }

    pub fn to_cartesian(&self) -> (f64, f64, f64) {
        let x = self.rad() * cos(self.azimuth) * sin(self.inclination);
        let y = self.rad() * cos(self.inclination);
        let z = self.rad() * sin(self.azimuth) * sin(self.inclination);
        (x, y, z)
    }
    pub fn from_cartesian(coords: (f64, f64, f64)) -> Self {
        let (x, y, z) = coords;
        let r = (x * x + y * y + z * z).sqrt();
        let inc = radians::acos(y / r);
        let azi = radians::atan2(z, x).unwrap_or_default();

        Self::with_radius(r, inc, azi)
    }
    pub fn scale(&self, scale: f64) -> SphericalCoords {
        SphericalCoords { radius: self.radius * scale, ..*self }
    }
    pub fn to_unit(&self) -> SphericalCoords { Self::unit(self.inclination, self.azimuth) }

    pub fn translate_down(&self, theta: Radians) -> Self {
        let azi = self.azimuth;
        let inc = self.inclination;

        let cos_sin = cos(azi) * sin(inc);

        let x = cos(inc) * sin(theta) + cos_sin * cos(theta);
        let y = sin(inc) * sin(azi);
        let z = cos(inc) * cos(theta) - cos_sin * sin(theta);

        let new_azi = radians::atan2(y, x);
        let new_inc = radians::acos(z);

        if new_azi.is_err() { return Self::with_radius(self.radius, Radians::ZERO, Radians::ZERO) }

        Self::with_radius(self.radius, new_inc, new_azi.unwrap())
    }
    pub fn translate_across(&self, phi: Radians) -> Self {
        Self::with_radius(self.radius, self.inclination, self.azimuth + phi)
    }
    pub fn translate(&self, translation: SphericalTranslation) -> Self {
        let theta = translation.inclination;
        let phi = translation.azimuth;
        self.translate_down(theta).translate_across(phi)
    }
    pub fn inverse_translate(&self, translation: SphericalTranslation) -> Self {
        let theta = translation.inclination;
        let phi = translation.azimuth;
        self.translate_across(-phi).translate_down(-theta)
    }
    pub fn rotate(&self, rotation: Radians) -> Self {
        Self::with_radius(self.radius, self.inclination, self.azimuth + rotation)
    }
    pub fn transform(&self, rotation: Radians, translation: SphericalTranslation) -> Self {
        self.rotate(rotation).translate(translation)
    }
    pub fn inverse_transform(&self, rotation: Radians, translation: SphericalTranslation) -> Self {
        self.inverse_translate(translation).rotate(-rotation)
    }
    pub fn mirror(&self) -> Self { SphericalCoords::with_radius(
        self.radius, self.inclination +  Radians::HALF_TAU, self.azimuth) }
    
    pub fn arc_distance(&self, other: SphericalCoords) -> Radians {
        // Simplified from:
        // self.inverse_translate(other.into()).inclination
        
        let theta = -other.inc();
        let azi = self.azi() - other.azi();
        let inc = self.inc();
        
        let z = cos(inc) * cos(theta) - cos(azi) * sin(inc) * sin(theta);
        radians::acos(z)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SphericalTranslation {
    inclination: Radians,
    azimuth: Radians,
}
impl SphericalTranslation {
    pub fn new(inclination: Radians, azimuth: Radians) -> Self { Self { inclination, azimuth } }
    pub fn to_unit_coords(&self) -> SphericalCoords {
        SphericalCoords::unit(self.inclination, self.azimuth)
    }
}
impl From<SphericalCoords> for SphericalTranslation {
    fn from(value: SphericalCoords) -> Self { Self::new(value.inclination, value.azimuth) }
}
