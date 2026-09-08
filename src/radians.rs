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
