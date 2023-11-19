use std::f32::consts::PI;

use nalgebra::{Matrix4, point, Point3, Rotation3, Scale3, Translation3, vector, Vector3};

#[derive(Clone, Copy)]
pub struct NodeTransform {
    pub position: Point3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

impl NodeTransform {
    pub fn identity() -> Self {
        Self {
            position: point![0.0, 0.0, 0.0],
            rotation: vector![0.0, 0.0, 0.0],
            scale: vector![1.0, 1.0, 1.0],
        }
    }

    pub fn to_homogeneous(&self) -> Matrix4<f32> {
        let translation =
            Translation3::new(self.position.x, self.position.y, self.position.z).to_homogeneous();
        let rotation = Rotation3::new(self.rotation * (PI / 180.0)).to_homogeneous();
        let scale = Scale3::new(self.scale.x, self.scale.y, self.scale.z).to_homogeneous();

        translation * rotation * scale
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::{point, vector};

    use crate::transform::NodeTransform;

    #[test]
    fn can_invert_identity() {
        let transform = NodeTransform::identity();

        let m = transform.to_homogeneous().try_inverse().unwrap();

        assert_eq!(
            m.transform_point(&point![0.0, 0.0, 0.0]),
            point![0.0, 0.0, 0.0]
        )
    }

    #[test]
    fn transform_position() {
        let mut transform = NodeTransform::identity();

        assert_eq!(transform.position, point![0.0, 0.0, 0.0]);

        transform.position = point![10.0, 0.0, 0.0];

        assert_eq!(transform.position, point![10.0, 0.0, 0.0]);
        assert_eq!(
            transform
                .to_homogeneous()
                .transform_point(&point![0.0, 0.0, 0.0]),
            point![10.0, 0.0, 0.0]
        );
    }

    #[test]
    fn transform_rotation() {
        let mut transform = NodeTransform::identity();

        assert_eq!(transform.rotation, vector![0.0, 0.0, 0.0]);

        transform.rotation = vector![180.0, 0.0, 0.0];

        assert_eq!(transform.rotation, vector![180.0, 0.0, 0.0]);
        assert!(
            (dbg!(transform
                .to_homogeneous()
                .transform_point(&point![0.0, 0.0, 1.0]))
                - point![0.0, 0.0, -1.0])
            .magnitude()
                <= 0.0001
        );
    }
}
