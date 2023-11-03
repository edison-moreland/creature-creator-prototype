use crate::renderer::widgets::strokes::{Stroke, StrokeSet, Style};
use crate::renderer::widgets::Widget;
use nalgebra::{point, vector, Matrix4, Point, Point3, Rotation3, Scale3, Translation3, Vector3};

use crate::surfaces::primitives::{ellipsoid, translate};
use crate::surfaces::{gradient, on_surface, Surface};

// A limb is a straight line from a to b
// A number of implicit surface live on the limb with a local coordinate system
// origin is at B, with an up vector pointing towards A. The range between A and B is 0.0,1.0
pub struct Limb {
    debug_info: StrokeSet,

    surfaces: Vec<(Vector3<f32>, Vector3<f32>)>,
    // (position, ellipsoid params)
    to: Matrix4<f32>,
}

impl Limb {
    pub fn new(
        a: Vector3<f32>,
        b: Vector3<f32>,
        surfaces: Vec<(Vector3<f32>, Vector3<f32>)>,
    ) -> Self {
        // Limb coordinate system
        // In limb space:
        // origin: between a and b
        // a     : (0.0, 1.0, 0.0)
        // b     : (0.0, -1.0, 0.0)

        let dist = a.metric_distance(&b);
        let origin = Point3::from((a + b) / 2.0);

        let t = Translation3::new(-origin.x, -origin.y, -origin.z);
        let s = Scale3::new(1.0, 2.0 / dist, 1.0);
        let r = Rotation3::rotation_between(&(a - b), &vector![0.0, 1.0, 0.0]).unwrap();

        let to = s.to_homogeneous() * r.to_homogeneous() * t.to_homogeneous();

        assert_eq!(to.transform_point(&origin), point![0.0, 0.0, 0.0]);
        assert_eq!(to.transform_point(&Point3::from(b)), point![0.0, -1.0, 0.0]);
        assert_eq!(to.transform_point(&Point3::from(a)), point![0.0, 1.0, 0.0]);

        let debug_info = Self::debug_info(a, b, origin);

        Limb {
            debug_info,
            to,
            surfaces,
        }
    }

    fn debug_info(a: Vector3<f32>, b: Vector3<f32>, origin: Point<f32, 3>) -> StrokeSet {
        let mut debug_info = StrokeSet::new();
        debug_info.set_palette(vec![
            Style::new(vector![0.0, 0.0, 0.0], 0.4, 0.0),
            Style::new(vector![0.0, 0.0, 0.0], 0.1, 0.0),
        ]);

        debug_info.stroke(0, Stroke::Line { start: a, end: b });
        let normal = (a - b).normalize();
        debug_info.stroke(
            1,
            Stroke::Circle {
                origin: a,
                normal,
                radius: 3.0,
            },
        );
        debug_info.stroke(
            1,
            Stroke::Circle {
                origin: b,
                normal,
                radius: 3.0,
            },
        );
        debug_info.stroke(
            1,
            Stroke::Circle {
                origin: origin.coords,
                normal,
                radius: 3.0,
            },
        );
        debug_info
    }
}

impl Surface for Limb {
    fn at(&self, _: f32, p: Vector3<f32>) -> f32 {
        let tp = self.to.transform_point(&Point3::from(p));

        self.surfaces
            .iter()
            .map(|(position, params)| {
                translate(*position, ellipsoid(params.x, params.y, params.z))(tp.coords)
            })
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap()
    }
}
impl Widget for Limb {
    fn strokes(&self) -> Option<&StrokeSet> {
        Some(&self.debug_info)
    }
}
