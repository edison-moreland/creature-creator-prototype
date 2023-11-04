mod limb_store;

use crate::renderer::widgets::strokes::{Stroke, StrokeSet, Style};
use crate::renderer::widgets::Widget;
use crate::stick_man::limb_store::{Joint, JointRef, LimbStore};
use nalgebra::{point, vector, Matrix4, Rotation3, Scale3, Translation3, Vector2, Vector3};

pub use crate::stick_man::limb_store::Limb;
use crate::surfaces::Surface;

pub struct StickMan {
    debug_info: StrokeSet,

    limb_store: LimbStore,

    head_joint: JointRef,
    right_arm_joint: JointRef,
    left_arm_joint: JointRef,
    right_leg_joint: JointRef,
    left_leg_joint: JointRef,

    to: Matrix4<f32>,
    from: Matrix4<f32>,
}

impl StickMan {
    pub fn new(origin: Vector3<f32>, normal: Vector3<f32>, size: Vector2<f32>) -> Self {
        let t = Translation3::new(-origin.x, -origin.y, -origin.z);
        let s = Scale3::new(2.0 / size.x, 2.0 / size.y, 2.0 / ((size.x + size.y) / 4.0));
        let r = Rotation3::rotation_between(&normal, &vector![1.0, 0.0, 0.0]).unwrap();

        let to = s.to_homogeneous() * r.to_homogeneous() * t.to_homogeneous();

        let mut limb_store = LimbStore::new();
        let head_joint = limb_store.add_root_joint(Joint::new(vector![0.0, 1.0, 0.0]));
        let right_arm_joint = limb_store.add_root_joint(Joint::new(vector![0.0, 0.0, 1.0]));
        let left_arm_joint = limb_store.add_root_joint(Joint::new(vector![0.0, 0.0, -1.0]));
        let right_leg_joint = limb_store.add_root_joint(Joint::new(vector![0.0, -1.0, 0.0]));
        let left_leg_joint = limb_store.add_root_joint(Joint::new(vector![0.0, -1.0, 0.0]));

        let mut stroke_set = StrokeSet::new();
        stroke_set.set_palette(vec![Style::new(vector![0.0, 0.0, 0.0], 0.2, 0.0)]);

        Self {
            debug_info: stroke_set,

            limb_store,

            head_joint,
            right_arm_joint,
            left_arm_joint,
            right_leg_joint,
            left_leg_joint,

            to,
            from: to.try_inverse().unwrap(),
        }
    }

    pub fn head(&self) -> &JointRef {
        &self.head_joint
    }

    pub fn head_mut(&mut self) -> &mut JointRef {
        &mut self.head_joint
    }

    pub fn right_arm(&self) -> &JointRef {
        &self.right_arm_joint
    }
    pub fn right_arm_mut(&mut self) -> &mut JointRef {
        &mut self.right_arm_joint
    }

    pub fn left_arm(&self) -> &JointRef {
        &self.left_arm_joint
    }
    pub fn left_arm_mut(&mut self) -> &mut JointRef {
        &mut self.left_arm_joint
    }

    pub fn right_leg(&self) -> &JointRef {
        &self.right_leg_joint
    }
    pub fn right_leg_mut(&mut self) -> &mut JointRef {
        &mut self.right_leg_joint
    }

    pub fn left_leg(&self) -> &JointRef {
        &self.left_leg_joint
    }
    pub fn left_leg_mut(&mut self) -> &mut JointRef {
        &mut self.left_leg_joint
    }
}

// All the stuff for handling debug_info
impl StickMan {
    pub fn update_debug_strokes(&mut self) {
        let a = self.from.transform_point(&point![0.0, 1.0, 1.0]).coords;
        let b = self.from.transform_point(&point![0.0, 1.0, -1.0]).coords;
        let c = self.from.transform_point(&point![0.0, -1.0, -1.0]).coords;
        let d = self.from.transform_point(&point![0.0, -1.0, 1.0]).coords;

        let origin = self.from.transform_point(&point![0.0, 0.0, 0.0]).coords;
        let normal = self
            .from
            .transform_vector(&vector![1.0, 0.0, 0.0])
            .normalize();

        self.debug_info.clear();
        self.stroke_core(a, b, c, d);
        self.stroke_normal(origin, normal);

        let roti = Rotation3::identity().to_homogeneous();
        self.stroke_joint((a + b) / 2.0, roti, &self.head().clone());
        self.stroke_joint(a, roti, &self.right_arm().clone());
        self.stroke_joint(b, roti, &self.left_arm().clone());
        self.stroke_joint(d, roti, &self.right_leg().clone());
        self.stroke_joint(c, roti, &self.left_leg().clone());
    }

    fn stroke_joint(&mut self, origin: Vector3<f32>, transform: Matrix4<f32>, joint: &JointRef) {
        if let Some(limb) = joint.limb() {
            let accum_transform = limb.transform.to_homogeneous() * transform;

            let final_transform = self.from * accum_transform;

            let end =
                origin + (final_transform.transform_vector(&joint.basis()).normalize() * limb.size);

            self.debug_info
                .stroke(0, Stroke::Line { start: origin, end });

            self.stroke_joint(end, accum_transform, &joint.next_joint().unwrap())
        }
    }

    fn stroke_core(&mut self, a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>, d: Vector3<f32>) {
        self.debug_info.stroke(0, Stroke::Line { start: a, end: b });
        self.debug_info.stroke(0, Stroke::Line { start: b, end: c });
        self.debug_info.stroke(0, Stroke::Line { start: c, end: d });
        self.debug_info.stroke(0, Stroke::Line { start: d, end: a });
    }

    fn stroke_normal(&mut self, origin: Vector3<f32>, normal: Vector3<f32>) {
        self.debug_info.stroke(
            0,
            Stroke::Arrow {
                origin,
                direction: normal,
                magnitude: 4.0,
            },
        );
    }
}

impl Widget for StickMan {
    fn strokes(&self) -> &StrokeSet {
        &self.debug_info
    }
}
