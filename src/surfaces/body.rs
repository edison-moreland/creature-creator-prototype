use crate::renderer::widgets::strokes::{Stroke, StrokeSet, Style};
use crate::renderer::widgets::Widget;
use crate::surfaces::primitives::ellipsoid;
use crate::surfaces::{gradient, on_surface, Surface};
use nalgebra::{matrix, point, vector, Matrix4, Point3, Rotation3, Scale3, Translation3, Vector3};

// A core is at the center of a body. It's defined by 4 co-planer points
// Limbs attach to the body at it's 4 corners
pub struct Core {
    debug_info: StrokeSet,
    to: Matrix4<f32>,
    a: Vector3<f32>,
    b: Vector3<f32>,
    c: Vector3<f32>,
    d: Vector3<f32>,
}

impl Core {
    pub fn new(a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>, d: Vector3<f32>) -> Self {
        // Expect point order
        // A +----+ D
        //   |    |
        // B +----+ C
        if matrix![a.x, a.y, a.z, 1.0;
                   b.x, b.y, b.z, 1.0;
                   c.x, c.y, c.z, 1.0;
                   d.x, d.y, d.z, 1.0]
        .determinant()
            != 0.0
        {
            panic!("Points are not coplanar!!")
        }

        // This makes sure the points form a rectangle, a requirement i'd like to relax
        let mid_ac = (a + c) / 2.0;
        let mid_bd = (b + d) / 2.0;
        assert_eq!(mid_ac, mid_bd);
        let origin = Point3::from(mid_ac);

        let ab_dist = a.metric_distance(&b);
        let bc_dist = b.metric_distance(&c);

        // Setup transform for children
        // In the core space:
        // (-1,  1, 0)       (1,  1, 0)
        //        A +---------+ D
        //          |         |
        //          |    + (0, 0, 0)
        //          |         |
        //        B +---------+ C
        // (-1, -1, 0)       (1, -1, 0)

        let t = Translation3::new(-origin.x, -origin.y, -origin.z);
        let s = Scale3::new(
            2.0 / bc_dist,
            2.0 / ab_dist,
            2.0 / ((ab_dist + bc_dist) / 4.0),
        );
        let r = Rotation3::rotation_between(&(a - b), &vector![0.0, 1.0, 0.0]).unwrap();

        let to = s.to_homogeneous() * r.to_homogeneous() * t.to_homogeneous();

        assert_eq!(to.transform_point(&origin), point![0.0, 0.0, 0.0]);
        assert_eq!(to.transform_point(&Point3::from(a)), point![-1.0, 1.0, 0.0]);
        assert_eq!(
            to.transform_point(&Point3::from(b)),
            point![-1.0, -1.0, 0.0]
        );
        assert_eq!(to.transform_point(&Point3::from(c)), point![1.0, -1.0, 0.0]);
        assert_eq!(to.transform_point(&Point3::from(d)), point![1.0, 1.0, 0.0]);

        let debug_info = Self::debug_info(a, b, c, d);

        Self {
            debug_info,
            to,
            a,
            b,
            c,
            d,
        }
    }

    fn debug_info(a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>, d: Vector3<f32>) -> StrokeSet {
        let mut debug_info = StrokeSet::new();
        debug_info.set_palette(vec![Style::new(vector![0.0, 0.0, 0.0], 0.1, 0.0)]);

        debug_info.stroke(0, Stroke::Line { start: a, end: b });
        debug_info.stroke(0, Stroke::Line { start: b, end: c });
        debug_info.stroke(0, Stroke::Line { start: c, end: d });
        debug_info.stroke(0, Stroke::Line { start: d, end: a });

        debug_info
    }
}

impl Surface for Core {
    fn at(&self, _: f32, p: Vector3<f32>) -> f32 {
        let surface = ellipsoid(1.0, 1.0, 1.0);

        let tp = self.to.transform_point(&Point3::from(p));

        surface(tp.coords)
    }

    fn sample_point(&self) -> Vector3<f32> {
        let surface = ellipsoid(1.0, 1.0, 1.0);

        let mut point = self.to.transform_point(&point![1.0, 0.0, 0.0]).coords;

        for _ in 0..10 {
            let grad = gradient(self, 0.0, point);
            point -= grad.scale(self.at(0.0, point) / grad.dot(&grad));

            if on_surface(self, 0.0, point) {
                break;
            }
        }

        if !on_surface(self, 0.0, point) {
            panic!("uh oh!")
        }

        point
        // self.to
        //     .try_inverse()
        //     .unwrap()
        //     .transform_point(&Point3::from(point))
        //     .coords
    }
}

impl Widget for Core {
    fn strokes(&self) -> &StrokeSet {
        &self.debug_info
    }
}

#[cfg(test)]
mod test {
    use nalgebra::vector;

    use crate::surfaces::body::Core;

    #[test]
    fn are_coplanar() {
        Core::new(
            vector![-1.0, 1.0, 0.0],
            vector![-1.0, -1.0, 0.0],
            vector![1.0, -1.0, 0.0],
            vector![1.0, 1.0, 0.0],
        );
    }

    #[test]
    #[should_panic]
    fn are_not_coplanar() {
        Core::new(
            vector![-1.0, 1.0, 5.0],
            vector![-1.0, -1.0, -5.0],
            vector![1.0, -1.0, 5.0],
            vector![1.0, 1.0, -5.0],
        );
    }
}
