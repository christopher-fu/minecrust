use crate::geometry::{rectangle::Rectangle, BoundingBox, PrimitiveGeometry};
use crate::types::{Point3f, Transform3f};

pub struct Square {
    rect: Rectangle,
}

impl Square {
    pub fn new(side: f32) -> Square {
        Square {
            rect: Rectangle::new(side, side),
        }
    }
}

impl PrimitiveGeometry for Square {
    fn vtx_data(&mut self, transform: &Transform3f) -> Vec<f32> {
        self.rect.vtx_data(transform)
    }

    fn vertices(&mut self, transform: &Transform3f) -> Vec<Point3f> {
        self.rect.vertices(transform)
    }

    fn bounding_box(&self, transform: &Transform3f) -> BoundingBox {
        self.rect.bounding_box(transform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alga::general::SubsetOf;
    use crate::na::geometry::Isometry;
    use crate::na::{Rotation3, Translation3};
    use crate::types::{Point3f, Vector3f};
    use crate::utils;

    #[test]
    fn test_transform1() {
        let mut s = Square::new(1.0);
        let t = Translation3::from_vector(Vector3f::new(0.0, 2.0, 0.0));
        let vertices = s.vertices(&t.to_superset());
        assert!(utils::pt3f_almost_eq(
            &vertices[0],
            &Point3f::new(-0.5, 2.0, -0.5)
        ));
        assert!(utils::pt3f_almost_eq(
            &vertices[1],
            &Point3f::new(-0.5, 2.0, 0.5)
        ));
        assert!(utils::pt3f_almost_eq(
            &vertices[2],
            &Point3f::new(0.5, 2.0, 0.5)
        ));
        assert!(utils::pt3f_almost_eq(
            &vertices[3],
            &Point3f::new(0.5, 2.0, -0.5)
        ));
    }

    #[test]
    fn test_transform2() {
        let mut s = Square::new(1.0);
        let t = Isometry::from_parts(
            Translation3::from_vector(Vector3f::new(0.0, 0.0, 0.5)),
            Rotation3::from_axis_angle(&Vector3f::x_axis(), ::std::f32::consts::FRAC_PI_2),
        );
        let vertices = s.vertices(&t.to_superset());
        assert!(utils::pt3f_almost_eq(
            &vertices[0],
            &Point3f::new(-0.5, 0.5, 0.5)
        ));
        assert!(utils::pt3f_almost_eq(
            &vertices[1],
            &Point3f::new(-0.5, -0.5, 0.5)
        ));
        assert!(utils::pt3f_almost_eq(
            &vertices[2],
            &Point3f::new(0.5, -0.5, 0.5)
        ));
        assert!(utils::pt3f_almost_eq(
            &vertices[3],
            &Point3f::new(0.5, 0.5, 0.5)
        ));
    }
}
