use crate::{
    types::prelude::*,
    utils::{f32, point3f},
};
use std::f32::INFINITY;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Face {
    Top,
    Front,
    Left,
    Right,
    Back,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct Octants {
    pub tfr: AABB,
    pub tfl: AABB,
    pub tbr: AABB,
    pub tbl: AABB,
    pub bfr: AABB,
    pub bfl: AABB,
    pub bbr: AABB,
    pub bbl: AABB,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct AABB {
    pub center: Point3f,
    /// center + extent = corner (i.e. half side len)
    pub extents: Vector3f,
}

impl AABB {
    pub fn new_min_max(min: Point3f, max: Point3f) -> AABB {
        let center = point3f::midpoint(&min, &max);
        let extents = (point3f::max(&min, &max) - point3f::min(&min, &max)) / 2.0;
        assert!(extents.x >= 0.0 && extents.y >= 0.0 && extents.z >= 0.0);
        AABB { center, extents }
    }

    pub fn min(&self) -> Point3f {
        self.center - self.extents
    }
    pub fn max(&self) -> Point3f {
        self.center + self.extents
    }

    fn merge_two_aabbs(a: &AABB, b: &AABB) -> AABB {
        AABB::new_min_max(
            point3f::min(&a.min(), &b.min()),
            point3f::max(&a.max(), &b.max()),
        )
    }

    pub fn merge_aabbs(aabbs: &[AABB]) -> AABB {
        if aabbs.len() == 1 {
            aabbs[0]
        } else {
            let mut aabb = aabbs[0];
            for bb in &aabbs[0..] {
                aabb = AABB::merge_two_aabbs(&aabb, bb);
            }
            aabb
        }
    }

    pub fn new_infinite() -> AABB {
        AABB::new_min_max(
            Point3f::new(-INFINITY, -INFINITY, -INFINITY),
            Point3f::new(INFINITY, INFINITY, INFINITY),
        )
    }

    pub fn transform(&self, t: &Transform3f) -> AABB {
        AABB {
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

    pub fn partition(&self) -> Octants {
        if self.is_infinite() {
            panic!("{:?} is infinite", self);
        }
        let center = self.center;
        let min = self.min();
        let max = self.max();
        Octants {
            tfl: AABB::new_min_max(
                Point3f::new(min.x, center.y, center.z),
                Point3f::new(center.x, max.y, max.z),
            ),
            tfr: AABB::new_min_max(center, max),
            tbl: AABB::new_min_max(
                Point3f::new(min.x, center.y, min.z),
                Point3f::new(center.x, max.y, center.z),
            ),
            tbr: AABB::new_min_max(
                Point3f::new(center.x, center.y, min.z),
                Point3f::new(max.x, max.y, center.z),
            ),
            bfl: AABB::new_min_max(
                Point3f::new(min.x, min.y, center.z),
                Point3f::new(center.x, center.y, max.z),
            ),
            bfr: AABB::new_min_max(
                Point3f::new(center.x, min.y, center.z),
                Point3f::new(max.x, center.y, max.z),
            ),
            bbl: AABB::new_min_max(min, center),
            bbr: AABB::new_min_max(
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
        let aabb = AABB::new_min_max(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.face(&Point3f::new(0.0, 1.0, 0.0)), Some(Face::Top));
        assert_eq!(aabb.face(&Point3f::new(0.0, -1.0, 0.0)), Some(Face::Bottom));
        assert_eq!(aabb.face(&Point3f::new(-1.0, 0.5, 0.5)), Some(Face::Left));
        assert_eq!(aabb.face(&Point3f::new(1.0, 0.5, 0.5)), Some(Face::Right));
        assert_eq!(aabb.face(&Point3f::new(0.5, 0.5, 1.0)), Some(Face::Front));
        assert_eq!(aabb.face(&Point3f::new(-0.5, 0.5, -1.0)), Some(Face::Back));
    }

    #[test]
    fn test_partition() {
        let aabb = AABB::new_min_max(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(1.0, 1.0, 1.0));
        let partition = aabb.partition();

        assert_eq!(
            partition.tfl,
            AABB::new_min_max(Point3f::new(-1.0, 0.0, 0.0), Point3f::new(0.0, 1.0, 1.0))
        );
        assert_eq!(
            partition.tfr,
            AABB::new_min_max(Point3f::new(0.0, 0.0, 0.0), Point3f::new(1.0, 1.0, 1.0))
        );
        assert_eq!(
            partition.tbl,
            AABB::new_min_max(Point3f::new(-1.0, 0.0, -1.0), Point3f::new(0.0, 1.0, 0.0))
        );
        assert_eq!(
            partition.tbr,
            AABB::new_min_max(Point3f::new(0.0, 0.0, -1.0), Point3f::new(1.0, 1.0, 0.0))
        );

        assert_eq!(
            partition.bfl,
            AABB::new_min_max(Point3f::new(-1.0, -1.0, 0.0), Point3f::new(0.0, 0.0, 1.0))
        );
        assert_eq!(
            partition.bfr,
            AABB::new_min_max(Point3f::new(0.0, -1.0, 0.0), Point3f::new(1.0, 0.0, 1.0))
        );
        assert_eq!(
            partition.bbl,
            AABB::new_min_max(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(0.0, 0.0, 0.0))
        );
        assert_eq!(
            partition.bbr,
            AABB::new_min_max(Point3f::new(0.0, -1.0, -1.0), Point3f::new(1.0, 0.0, 0.0))
        );
    }
}
