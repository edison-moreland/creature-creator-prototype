use crate::surfaces::primitives::{ellipsoid, translate};
use crate::surfaces::Surface;
use nalgebra::{
    point, vector, Affine3, Isometry3, Matrix, Matrix4, Point3, Rotation3, Scale3, Similarity3,
    TAffine, Transform, Transform3, Translation3, Vector3,
};

// A limb is a straight line from a to b
// A number of implicit surface live on the limb with a local coordinate system
// origin is at B, with an up vector pointing towards A. The range between A and B is 0.0,1.0
pub struct Limb {
    a: Vector3<f32>,
    b: Vector3<f32>,
    surfaces: Vec<(Vector3<f32>, Vector3<f32>)>, // (position, ellipsoid params)
    to: Matrix4<f32>,
    from: Matrix4<f32>,
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
        let dir = (a - b).normalize();
        let origin = Point3::from(b + dir.scale(dist / 2.0));

        dbg!(origin);

        let t = Translation3::new(-origin.x, -origin.y, -origin.z);
        let s = Scale3::new(1.0, 2.0 / dist, 1.0);
        let r = Rotation3::rotation_between(&(a - b), &vector![0.0, 1.0, 0.0]).unwrap();

        let to = s.to_homogeneous() * r.to_homogeneous() * t.to_homogeneous();

        // assert_eq!(to.transform_point(&origin), point![0.0, 0.0, 0.0]);
        // assert_eq!(to.transform_point(&Point3::from(b)), point![0.0, -1.0, 0.0]);
        // assert_eq!(to.transform_point(&Point3::from(a)), point![0.0, 1.0, 0.0]);

        Limb {
            to,
            from: to.try_inverse().unwrap(),
            a,
            b,
            surfaces,
        }
    }
}

impl Surface for Limb {
    fn at(&self, _: f32, p: Vector3<f32>) -> f32 {
        let tp = self.to.transform_vector(&p);

        // dbg!(tp, p);

        self.surfaces
            .iter()
            .map(|(position, params)| {
                translate(*position, ellipsoid(params.x, params.y, params.z))(tp)
            })
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap()
    }

    fn sample_point(&self) -> Vector3<f32> {
        let first = self.surfaces.first().unwrap();

        self.from
            .transform_vector(&(first.0 + vector![first.1.x, 0.0, 0.0]))
    }
}
