use super::position::AbsolutePosition;
use super::Radians;
use super::SphericalCoords;

pub struct Polygon {
    coords: Vec<SphericalCoords>,
    is_closed: bool,
}

impl Polygon {
    pub fn new(coords: Vec<SphericalCoords>, is_closed: bool) -> Polygon {
        Polygon { coords, is_closed }
    }

    pub fn coords(&self) -> &[SphericalCoords] { &self.coords }
    pub fn is_closed(&self) -> bool { self.is_closed }

    pub fn closest_coords(&self, coords: SphericalCoords) -> Option<usize> {
        let mut min = Radians::HALF_TAU;
        let mut idx = None;

        for i in 0..self.coords.len() {
            let d = coords.arc_distance(self.coords[i]);
            if d < min { min = d; idx = Some(i); }
        }
        idx
    }
    pub fn closest_arc(&self, coords: SphericalCoords) -> Option<usize> {
        let mut min = Radians::HALF_TAU;
        let mut idx = None;

        let len = self.coords.len();
        let top =  if self.is_closed { len } else { len - 1 };
        for i in 0..top {
            let a = self.coords[i];
            let b = self.coords[(i + 1) % len];
            let d = coords.arc_distance_from_line(a, b);
            if d < min { min = d; idx = Some(i); }
        }
        idx
    }

    pub fn transform(&self, position: AbsolutePosition) -> Polygon {
        let coords = self.coords()
            .iter()
            .map(|&c| position.transform_coords(c))
            .collect();

        Polygon::new(coords, self.is_closed)
    }
}