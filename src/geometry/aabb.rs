use crate::{
    types::{prelude::*, Octants},
    utils::{f32, point3f},
};
use std::f32::INFINITY;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Aabb {
    pub center: Point3f,
    /// center + extent = corner (i.e. half side len)
    pub extents: Vector3f,
}

impl Aabb {
    pub fn new(center: Point3f, extents: Vector3f) -> Aabb {
        Aabb { center, extents }
    }

    pub fn new_min_max(min: Point3f, max: Point3f) -> Aabb {
        let center = point3f::midpoint(&min, &max);
        let extents = (point3f::max(&min, &max) - point3f::min(&min, &max)) / 2.0;
        assert!(extents.x >= 0.0 && extents.y >= 0.0 && extents.z >= 0.0);
        Aabb { center, extents }
    }

    pub fn min(&self) -> Point3f {
        self.center - self.extents
    }
    pub fn max(&self) -> Point3f {
        self.center + self.extents
    }

    fn merge_two_aabbs(a: &Aabb, b: &Aabb) -> Aabb {
        Aabb::new_min_max(
            point3f::min(&a.min(), &b.min()),
            point3f::max(&a.max(), &b.max()),
        )
    }

    pub fn merge_aabbs(aabbs: &[Aabb]) -> Aabb {
        if aabbs.len() == 1 {
            aabbs[0]
        } else {
            let mut aabb = aabbs[0];
            for bb in &aabbs[0..] {
                aabb = Aabb::merge_two_aabbs(&aabb, bb);
            }
            aabb
        }
    }

    pub fn new_infinite() -> Aabb {
        Aabb::new_min_max(
            Point3f::new(-INFINITY, -INFINITY, -INFINITY),
            Point3f::new(INFINITY, INFINITY, INFINITY),
        )
    }

    pub fn transform(&self, t: &Transform3f) -> Aabb {
        Aabb {
            center: t * self.center,
            extents: self.extents,
        }
    }

    pub fn face(&self, point: &Point3f) -> Option<Face> {
        let max = self.max();
        let min = self.min();
        if f32::almost_eq(Vector3f::y_axis().dot(&(point - max)), 0.0) {
            Some(Face::Top)
        } else if f32::almost_eq(-Vector3f::y_axis().dot(&(point - min)), 0.0) {
            Some(Face::Bottom)
        } else if f32::almost_eq(Vector3f::z_axis().dot(&(point - max)), 0.0) {
            Some(Face::Front)
        } else if f32::almost_eq(-Vector3f::z_axis().dot(&(point - min)), 0.0) {
            Some(Face::Back)
        } else if f32::almost_eq(-Vector3f::x_axis().dot(&(point - min)), 0.0) {
            Some(Face::Left)
        } else if f32::almost_eq(Vector3f::x_axis().dot(&(point - max)), 0.0) {
            Some(Face::Right)
        } else {
            None
        }
    }

    pub fn points(&self) -> Vec<Point3f> {
        let max = self.max();
        let min = self.min();
        let (min_x, min_y, min_z) = (min.x, min.y, min.z);
        let (max_x, max_y, max_z) = (max.x, max.y, max.z);
        vec![
            min,
            Point3f::new(min_x, min_y, max_z),
            Point3f::new(min_x, max_y, min_z),
            Point3f::new(min_x, max_y, max_z),
            Point3f::new(max_x, min_y, min_z),
            max,
        ]
    }

    pub fn is_infinite(&self) -> bool {
        self.extents.x.is_infinite() || self.extents.y.is_infinite() || self.extents.z.is_infinite()
    }

    pub fn partition(&self) -> Octants<Aabb> {
        if self.is_infinite() {
            panic!("{:?} is infinite", self);
        }
        let center = self.center;
        let min = self.min();
        let max = self.max();
        Octants {
            tfl: Aabb::new_min_max(
                Point3f::new(min.x, center.y, center.z),
                Point3f::new(center.x, max.y, max.z),
            ),
            tfr: Aabb::new_min_max(center, max),
            tbl: Aabb::new_min_max(
                Point3f::new(min.x, center.y, min.z),
                Point3f::new(center.x, max.y, center.z),
            ),
            tbr: Aabb::new_min_max(
                Point3f::new(center.x, center.y, min.z),
                Point3f::new(max.x, max.y, center.z),
            ),
            bfl: Aabb::new_min_max(
                Point3f::new(min.x, min.y, center.z),
                Point3f::new(center.x, center.y, max.z),
            ),
            bfr: Aabb::new_min_max(
                Point3f::new(center.x, min.y, center.z),
                Point3f::new(max.x, center.y, max.z),
            ),
            bbl: Aabb::new_min_max(min, center),
            bbr: Aabb::new_min_max(
                Point3f::new(center.x, min.y, min.z),
                Point3f::new(max.x, center.y, center.z),
            ),
        }
    }

    pub fn contains(&self, point: &Point3f) -> bool {
        let v = point - self.center;
        v.x.abs() <= self.extents.x && v.y.abs() <= self.extents.y && v.z.abs() <= self.extents.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face() {
        let aabb = Aabb::new_min_max(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.face(&Point3f::new(0.0, 1.0, 0.0)), Some(Face::Top));
        assert_eq!(aabb.face(&Point3f::new(0.0, -1.0, 0.0)), Some(Face::Bottom));
        assert_eq!(aabb.face(&Point3f::new(-1.0, 0.5, 0.5)), Some(Face::Left));
        assert_eq!(aabb.face(&Point3f::new(1.0, 0.5, 0.5)), Some(Face::Right));
        assert_eq!(aabb.face(&Point3f::new(0.5, 0.5, 1.0)), Some(Face::Front));
        assert_eq!(aabb.face(&Point3f::new(-0.5, 0.5, -1.0)), Some(Face::Back));
    }

    #[test]
    fn test_partition() {
        let aabb = Aabb::new_min_max(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(1.0, 1.0, 1.0));
        let partition = aabb.partition();

        assert_eq!(
            partition.tfl,
            Aabb::new_min_max(Point3f::new(-1.0, 0.0, 0.0), Point3f::new(0.0, 1.0, 1.0))
        );
        assert_eq!(
            partition.tfr,
            Aabb::new_min_max(Point3f::new(0.0, 0.0, 0.0), Point3f::new(1.0, 1.0, 1.0))
        );
        assert_eq!(
            partition.tbl,
            Aabb::new_min_max(Point3f::new(-1.0, 0.0, -1.0), Point3f::new(0.0, 1.0, 0.0))
        );
        assert_eq!(
            partition.tbr,
            Aabb::new_min_max(Point3f::new(0.0, 0.0, -1.0), Point3f::new(1.0, 1.0, 0.0))
        );

        assert_eq!(
            partition.bfl,
            Aabb::new_min_max(Point3f::new(-1.0, -1.0, 0.0), Point3f::new(0.0, 0.0, 1.0))
        );
        assert_eq!(
            partition.bfr,
            Aabb::new_min_max(Point3f::new(0.0, -1.0, 0.0), Point3f::new(1.0, 0.0, 1.0))
        );
        assert_eq!(
            partition.bbl,
            Aabb::new_min_max(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(0.0, 0.0, 0.0))
        );
        assert_eq!(
            partition.bbr,
            Aabb::new_min_max(Point3f::new(0.0, -1.0, -1.0), Point3f::new(1.0, 0.0, 0.0))
        );
    }
}
