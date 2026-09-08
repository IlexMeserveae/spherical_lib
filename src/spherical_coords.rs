use super::radians::{Radians, cos, sin};
use std::fmt::{write, Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct SphericalCoords {
    radius: f64,
    inclination: Radians,
    azimuth: Radians,
}
impl Display for SphericalCoords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pre = f.precision().unwrap_or(3);
        if f.alternate() {
            write!(f, "({:.p$}, {:.p$}°, {:.p$}°)", self.rad(),
                   self.inc().to_degrees(), self.azi().to_degrees(), p = pre)
        }
        else {
            write!(f, "({:.p$}, {:.p$}, {:.p$})", self.rad(),
                   self.inc(), self.azi(), p = pre)
        }
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
        let inc = super::radians::acos(y / r);
        let azi = super::radians::atan2(z, x).unwrap_or_default();

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

        let new_azi = super::radians::atan2(y, x);
        let new_inc = super::radians::acos(z);

        if new_azi.is_err() { return Self::with_radius(self.radius, Radians::ZERO, azi) }

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
        super::radians::acos(z)
    }
    pub fn arc_distance_from_line(&self, a: SphericalCoords, b: SphericalCoords) -> Radians {
        let p = *self;

        let trans = a.into();
        let p2 = p.inverse_translate(trans);
        let a2 = a.inverse_translate(trans);
        let b2 = b.inverse_translate(trans);

        let rotation = Radians::QUARTER_TAU - b2.azi();
        let trans2 = SphericalCoords::with_radius(a.rad(),
                                                  Radians::QUARTER_TAU, Radians::ZERO).into();
        let p3 = p2.transform(rotation, trans2);
        let a3 = a2.transform(rotation, trans2);
        let b3 = b2.transform(rotation, trans2);

        // Account for closest point not actually being on the line
        if p3.azi() < Radians::ZERO || p3.azi() > b3.azi() {
            let d_a = p3.arc_distance(a3);
            let d_b = p3.arc_distance(b3);
            Radians::new(f64::min(d_a.value(), d_b.value()))
        }
        else {
            (p3.inc() - Radians::QUARTER_TAU).abs()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SphericalTranslation {
    inclination: Radians,
    azimuth: Radians,
}
impl Display for SphericalTranslation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pre = f.precision().unwrap_or(3);
        if f.alternate() {
            write!(f, "({:.p$}°, {:.p$}°)", 
                   self.inclination.to_degrees(), self.azimuth.to_degrees(), p = pre)
        }
        else {
            write!(f, "({:.p$}, {:.p$})", 
                   self.inclination, self.azimuth, p = pre)
        }
    }
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
