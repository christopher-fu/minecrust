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

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Point3f,
    pub max: Point3f,
}

impl AABB {
    pub fn new(min: Point3f, max: Point3f) -> AABB {
        AABB {
            min: point3f::min(&min, &max),
            max: point3f::max(&min, &max),
        }
    }

    fn _merge_two_aabbs(a: &AABB, b: &AABB) -> AABB {
        AABB {
            min: point3f::min(&a.min, &b.min),
            max: point3f::max(&a.max, &b.max),
        }
    }

    pub fn merge_aabbs(aabbs: &[AABB]) -> AABB {
        if aabbs.len() == 1 {
            aabbs[0]
        } else {
            let mut aabb = aabbs[0];
            for bb in &aabbs[0..] {
                aabb = AABB::_merge_two_aabbs(&aabb, bb);
            }
            aabb
        }
    }

    pub fn new_infinite() -> AABB {
        AABB::new(
            Point3f::new(-INFINITY, -INFINITY, -INFINITY),
            Point3f::new(INFINITY, INFINITY, INFINITY),
        )
    }

    pub fn transform(&self, t: &Transform3f) -> AABB {
        AABB::new(t * self.min, t * self.max)
    }

    pub fn face(&self, point: &Point3f) -> Option<Face> {
        if f32::almost_eq(Vector3f::y_axis().dot(&(point - self.max)), 0.0) {
            Some(Face::Top)
        } else if f32::almost_eq(-Vector3f::y_axis().dot(&(point - self.min)), 0.0) {
            Some(Face::Bottom)
        } else if f32::almost_eq(Vector3f::z_axis().dot(&(point - self.max)), 0.0) {
            Some(Face::Front)
        } else if f32::almost_eq(-Vector3f::z_axis().dot(&(point - self.min)), 0.0) {
            Some(Face::Back)
        } else if f32::almost_eq(-Vector3f::x_axis().dot(&(point - self.min)), 0.0) {
            Some(Face::Left)
        } else if f32::almost_eq(Vector3f::x_axis().dot(&(point - self.max)), 0.0) {
            Some(Face::Right)
        } else {
            None
        }
    }

    pub fn points(&self) -> Vec<Point3f> {
        let (min_x, min_y, min_z) = (self.min.x, self.min.y, self.min.z);
        let (max_x, max_y, max_z) = (self.max.x, self.max.y, self.max.z);
        vec![
            self.min,
            Point3f::new(min_x, min_y, max_z),
            Point3f::new(min_x, max_y, min_z),
            Point3f::new(min_x, max_y, max_z),
            Point3f::new(max_x, min_y, min_z),
            self.max,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face() {
        let aabb = AABB::new(Point3f::new(-1.0, -1.0, -1.0), Point3f::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.face(&Point3f::new(0.0, 1.0, 0.0)), Some(Face::Top));
        assert_eq!(aabb.face(&Point3f::new(0.0, -1.0, 0.0)), Some(Face::Bottom));
        assert_eq!(aabb.face(&Point3f::new(-1.0, 0.5, 0.5)), Some(Face::Left));
        assert_eq!(aabb.face(&Point3f::new(1.0, 0.5, 0.5)), Some(Face::Right));
        assert_eq!(aabb.face(&Point3f::new(0.5, 0.5, 1.0)), Some(Face::Front));
        assert_eq!(aabb.face(&Point3f::new(-0.5, 0.5, -1.0)), Some(Face::Back));
    }
}
