#![forbid(unsafe_code)]

pub use radians::Radians;
pub use spherical_coords::SphericalCoords;
pub use spherical_coords::SphericalTranslation;
pub use polygon::Polygon;
pub use object_id::ObjectId;

pub mod radians;
pub mod spherical_coords;
pub mod polygon;
pub mod object_id {
    use std::fmt::Display;
    use std::ops::Deref;

    #[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct ObjectId { id: usize }
    impl ObjectId {
        pub fn new(id: usize) -> Option<ObjectId> {
            if id < 4096 { Some(ObjectId { id }) }
            else { None }
        }
        pub fn force(id: usize) -> ObjectId {
            Self::new(id).expect("PlateId is out of bounds!")
        }
    }
    impl Deref for ObjectId {
        type Target = usize;
        fn deref(&self) -> &usize { &self.id }
    }
    impl Display for ObjectId {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            write!(f, "#{:03}", *self)
        }
    }
}
pub mod position {
    use super::Radians;
    use super::SphericalCoords;
    use super::ObjectId;

    #[derive(Copy, Clone, Debug)]
    pub struct AbsolutePosition {
        coords: SphericalCoords,
        rotation: Radians,
    }
    impl Default for AbsolutePosition {
        fn default() -> Self { AbsolutePosition::new(SphericalCoords::zenith(), Radians::ZERO) }
    }
    impl AbsolutePosition {
        pub fn new(coords: SphericalCoords, rotation: Radians) -> AbsolutePosition {
            AbsolutePosition { coords, rotation }
        }
        pub fn coords(&self) -> SphericalCoords { self.coords }
        pub fn rotation(&self) -> Radians { self.rotation }
        pub fn transform_coords(&self, coords: SphericalCoords) -> SphericalCoords {
            coords.transform(self.rotation(), self.coords().into())
        }
        pub fn inverse_transform_coords(&self, coords: SphericalCoords) -> SphericalCoords {
            coords.inverse_transform(self.rotation(), self.coords().into())
        }
        pub fn to_relative(&self, parent_id: ObjectId, parent: AbsolutePosition) -> RelativePosition {
            let coords = parent.inverse_transform_coords(self.coords);
            RelativePosition::new(parent_id, coords, self.rotation())
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct RelativePosition {
        parent: ObjectId,
        coords: SphericalCoords,
        rotation: Radians,
    }
    impl RelativePosition {
        pub fn new(parent: ObjectId, coords: SphericalCoords, rotation: Radians) -> RelativePosition {
            RelativePosition { parent, coords, rotation }
        }
        pub fn parent(&self) -> ObjectId { self.parent }
        pub fn coords(&self) -> SphericalCoords { self.coords }
        pub fn rotation(&self) -> Radians { self.rotation }
        pub fn to_absolute(&self, parent: AbsolutePosition) -> AbsolutePosition {
            let coords = parent.transform_coords(self.coords());
            AbsolutePosition::new(coords, self.rotation())
        }
    }
    impl Default for RelativePosition {
        fn default() -> Self {
            RelativePosition::new(ObjectId::force(0), SphericalCoords::zenith(), Radians::ZERO)
        }
    }
}

pub mod plotting;
#[cfg(feature = "egui_compat")]
pub mod egui_compat;
#[allow(dead_code)]
pub mod testing {
    use crate::radians::Radians;
    use crate::spherical_coords::SphericalCoords;

    pub fn coords_from_degrees(degrees: (f64, f64)) -> SphericalCoords {
        SphericalCoords::unit(Radians::from_degrees(degrees.0), Radians::from_degrees(degrees.1))
    }
    pub fn degrees_from_coords(coords: SphericalCoords) -> (f64, f64) {
        (coords.inc().to_degrees(), coords.azi().to_degrees())
    }

    pub fn approx_eq_floats(e: f64, a: f64) -> Result<(), String> {
        if (a - e).abs() > 0.001 { Err(format!("Expected {e:.3}, found {a:.3}!")) } else { Ok(()) }
    }
    pub fn approx_eq_param_floats(e: f64, a: f64, param: &str, index: usize) -> Result<(), String> {
        if (a - e).abs() > 0.001 {
            return Err(format!("Expected {param} = {e:.3} at index {index}, found {a:.3}!"))
        }

        Ok(())
    }
    pub fn approx_eq_coords<const N: usize>(expected: [SphericalCoords; N],
                                        actual: [SphericalCoords; N]) -> Result<(), String> {
        for i in 0..N {
            let e = expected[i];
            let a = actual[i];
            approx_eq_param_floats(e.rad(), a.rad(), "rad", i)?;
            approx_eq_param_floats(e.inc().value(), a.inc().value(), "inc", i)?;
            approx_eq_param_floats(e.azi().value(), a.azi().value(), "azi", i)?;
        };

        Ok(())
    }
}

#[cfg(test)]
mod spherical_coords_tests {
    use crate::radians::{normalise, Radians};
    use crate::SphericalCoords;
    use crate::testing::*;

    mod utils {
        use crate::radians::Radians;
        use crate::testing::{approx_eq_coords, coords_from_degrees};

        pub fn debug_square() -> [(f64, f64); 4] { [(25., 30.), (25., 120.), (25., 210.), (25., 300.)] }

        pub fn inverse_transform_check(label: &str, coords: (f64, f64), rotation: f64,
                                   translation: (f64, f64)) -> bool {
            let rotation = Radians::from_degrees(rotation);
            let translation = coords_from_degrees(translation).into();

            println!("\n\nInverse transform test - {}\n", label);

            let original = coords_from_degrees(coords);
            println!("Original: {:#.4}\n", original);

            let rotated = original.rotate(rotation);
            println!("Rotated: {:#.4}\n", rotated);

            let translated = rotated.translate(translation);
            println!("Translated: {:#.4}\n", translated);

            let de_translated = translated.inverse_translate(translation);
            println!("De-translated: {:#.4}\n", de_translated);

            let de_rotated = de_translated.rotate(-rotation);
            println!("De-rotated: {:#.4}\n", de_rotated);

            if original.inc() == Radians::ZERO || original.inc() == Radians::HALF_TAU {
                println!("Point is at a zenith, so azimuth is arbitrary.\n");
                return true;
            }
            let succ = approx_eq_coords([original], [de_rotated]).is_ok();
            if !succ { println!("Test Failed!!!\n"); }
            succ
        }

    }

    #[test]
    fn normalise_test() {
        approx_eq_floats(normalise(0.75), 0.75).unwrap();
        approx_eq_floats(normalise(-3.10), -3.10).unwrap();
        approx_eq_floats(normalise(4.2), -2.083).unwrap();
        approx_eq_floats(normalise(-37.5), 0.199).unwrap();
    }

    #[test]
    fn translate_test_1() {
        let original = utils::debug_square()
            .map(|deg| coords_from_degrees(deg));

        let translation = coords_from_degrees((0., 0.)).into();
        let translated = original
            .map(|coords| coords.translate(translation));

        let expected = [
            (25.00, 30.00),
            (25.00, 120.00),
            (25.00, 210.00),
            (25.00, 300.00)
        ]
            .map(|deg| coords_from_degrees(deg));
        approx_eq_coords(expected, translated).unwrap();
    }

    #[test]
    fn translate_test_2() {
        let original = utils::debug_square()
            .map(|deg| coords_from_degrees(deg));

        let translation = coords_from_degrees((110., 130.)).into();
        let translated = original
            .map(|coords| coords.translate(translation));

        let expected = [
            (130.84, 146.22),
            ( 96.40, 151.61),
            ( 88.05, 117.79),
            (120.57, 104.85)
        ]
            .map(|deg| coords_from_degrees(deg));
        approx_eq_coords(expected, translated).unwrap();
    }

    #[test] #[ignore]
    fn translate_check() {
        let original = utils::debug_square()
            .map(|deg| coords_from_degrees(deg));

        let translation = coords_from_degrees((110., 130.)).into();
        let translated = original
            .map(|coords| coords.translate(translation));

        println!("Testing square translated by {:#.0}", translation);
        for coord in translated {
            println!("{:#.9}", coord)
        }
    }

    #[test]
    fn rotate_test() {
        let original = utils::debug_square()
            .map(|deg| coords_from_degrees(deg));

        let rotation = Radians::from_degrees(65.0);
        let rotated = original
            .map(|coords| coords.rotate(rotation));

        let expected = [
            (25.00,  95.00),
            (25.00, 185.00),
            (25.00, -85.00),
            (25.00,   5.00),
        ]
            .map(|deg| coords_from_degrees(deg));
        approx_eq_coords(expected, rotated).unwrap();
    }

    #[test]
    fn transform_test() {
        let original = utils::debug_square()
            .map(|deg| coords_from_degrees(deg));

        let rotation = Radians::from_degrees(-37.0);
        let translation = coords_from_degrees((74.0, -103.0)).into();
        let transformed = original
            .map(|coords| coords.transform(rotation, translation));

        let expected = [
            (98.82, -105.99),
            (78.45,  -77.65),
            (49.23,  -99.10),
            (72.58, -129.08),
        ]
            .map(|deg| coords_from_degrees(deg));
        approx_eq_coords(expected, transformed).unwrap();
    }

    #[test] #[ignore]
    fn inverse_transform_check_many() {
        let checks = [
            ("1", ( 35.,   60.),  25., (200.,  0.) ),
            ("2", (-10.,  -60.), 400., ( 15., -40.) ),
            ("3", (125.,  117.),  30., (  0.,  35.) ),
            ("4", (180., -819.), -37., (  7., -10.) ),
        ];

        if checks.map(|c| utils::inverse_transform_check(c.0, c.1, c.2, c.3))
            .contains(&false) {
            panic!("Inverse transform check failed.");
        }
    }

    #[test]
    fn inverse_transform_test() {
        let original = utils::debug_square()
            .map(|deg| coords_from_degrees(deg));

        let rotation = Radians::from_degrees(-54.0);
        let translation = coords_from_degrees((105.0, 33.0)).into();
        let transformed = original
            .map(|coords| coords.transform(rotation, translation));

        let expected = [
            (127.41, 20.50),
            (113.62, 57.92),
            ( 82.05, 43.00),
            ( 93.93, 10.23),
        ]
            .map(|deg| coords_from_degrees(deg));
        approx_eq_coords(expected, transformed).unwrap();

        let re_transformed = transformed
            .map(|coords| coords.inverse_transform(rotation, translation));

        let expected = original;
        approx_eq_coords(expected, re_transformed).unwrap()
    }

    #[test]
    fn transform_associativity_test() {
        let parent = (
            Radians::from_degrees(0.0),
            coords_from_degrees((40.0, -105.0)),
        );
        let child = (
            Radians::from_degrees(20.0),
            coords_from_degrees((10.0, 75.0)),
        );
        let point = coords_from_degrees((45.0, 20.0));

        let transform: fn(SphericalCoords, (_, SphericalCoords)) -> _ =
            |coords, (rot, map)|
                coords.transform(rot-map.azi(), map.into());

        let desc = {
            let new_child = (
                child.0,
                transform(child.1, parent),
            );
            transform(point, new_child)
        };
        let asc = {
            let new_point = transform(point, child);
            transform(new_point, parent)
        };

        println!("asc: {asc:#}");
        println!("desc: {desc:#}");
        approx_eq_coords([asc], [desc]).unwrap();
    }

    #[test]
    fn arc_distance_from_line_test() {
        // Expected values from: https://www.desmos.com/3d/fpxedu7aso

        let a = vec![(50.0, 0.0), (90.0, -90.0), (165.0, 20.0), (0.0, 0.0)];
        let b = vec![(0.0, -25.0), (15.0, 45.0), (90.0, 20.0), (135.0, -60.0)];
        let p = vec![(30.0, 40.0), (125.0, -150.0), (105.0, 20.0), (65.0, -35.0)];
        let expected = vec![18.747, 65.822, 0.000, 22.521];

        // Case No.2 tests if the bounds check is working

        for i in 0..4 {
            let a = coords_from_degrees(a[i]);
            let b = coords_from_degrees(b[i]);
            let p = coords_from_degrees(p[i]);

            let d = p.arc_distance_from_line(a, b);
            approx_eq_floats(d.to_degrees(), expected[i]).unwrap();
        }
    }
}

#[cfg(test)]
mod plotting_tests {
    use crate::plotting::spherical_cross_product;
    use crate::radians::Radians;
    use crate::spherical_coords::SphericalCoords;
    use crate::testing::*;

    #[test]
    fn spherical_cross_product_test() {
        const N: usize = 5;
        let a: [SphericalCoords; N] = [
            ( 30.0,   0.0),
            ( 40.0, -40.0),
            ( 90.0, 170.0),
            (  0.0,   0.0),
            ( 60.0,   0.0),
        ].map(|degrees| coords_from_degrees(degrees));

        let b: [SphericalCoords; N] = [
            ( 50.0,   0.0),
            (120.0, -40.0),
            ( 90.0, -25.0),
            ( 25.0, -60.0),
            (110.0,  30.0),
        ].map(|degrees| coords_from_degrees(degrees));

        let mut n: [SphericalCoords; N] = [SphericalCoords::unit(Radians::ZERO, Radians::ZERO); N];
        for i in 0..n.len() { n[i] = spherical_cross_product(a[i], b[i]); }

        let expected = [
            (90.00,  90.00),
            (90.00,  50.00),
            ( 0.00,   0.00),
            (90.00,  30.00),
            (61.24, 108.48),
        ].map(|degrees| coords_from_degrees(degrees));

        approx_eq_coords(expected, n).unwrap();
    }
}