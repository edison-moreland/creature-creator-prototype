use crate::renderer::widgets::strokes::{Stroke, StrokeSet, Style};
use crate::renderer::widgets::Widget;
use nalgebra::{point, vector, Matrix4, Rotation3, Scale3, Translation3, Unit, Vector2, Vector3};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct LimbStore {
    limbs: Vec<Limb>,
    joints: Vec<Joint>,
}

impl LimbStore {
    fn new() -> Self {
        Self {
            limbs: vec![],
            joints: vec![],
        }
    }

    fn add_root_joint(&mut self, joint: Joint) -> usize {
        let joint_idx = self.joints.len();
        self.joints.push(joint);

        joint_idx
    }
    fn attach_limb(&mut self, root_idx: usize, limb: Limb) -> (usize, usize) {
        let next_joint_idx = self.joints.len();
        let joint_basis = self.joints[root_idx].orientation_basis;
        self.joints.push(Joint {
            limb: None,
            orientation_basis: joint_basis,
        });

        let joint = &mut self.joints[root_idx];
        if joint.limb.is_some() {
            panic!("Limb already attached!")
        }

        let limb_idx = self.limbs.len();
        self.limbs.push(limb);

        joint.limb = Some((limb_idx, next_joint_idx));

        (limb_idx, next_joint_idx)
    }
}

#[derive(Copy, Clone)]
pub struct Limb {
    size: f32,
    transform: Rotation3<f32>,
}

impl Limb {
    pub fn new(rotation: Rotation3<f32>, size: f32) -> Self {
        Limb {
            size,
            transform: rotation,
        }
    }
}

struct Joint {
    limb: Option<(usize, usize)>, // Limb and next joint
    orientation_basis: Vector3<f32>,
}

impl Joint {
    fn new(basis: Vector3<f32>) -> Self {
        Self {
            limb: None,
            orientation_basis: basis,
        }
    }
}

pub struct JointRef {
    store: Weak<RefCell<LimbStore>>,
    index: usize,
}

impl JointRef {
    fn _store(&self) -> Rc<RefCell<LimbStore>> {
        self.store.upgrade().expect("Joint set not to be dropped")
    }

    pub fn attach(&mut self, limb: Limb) -> JointRef {
        let (_, next_joint_idx) = self._store().borrow_mut().attach_limb(self.index, limb);

        JointRef {
            store: self.store.clone(),
            index: next_joint_idx,
        }
    }

    pub fn next_joint(&self) -> Option<JointRef> {
        self._store().borrow().joints[self.index]
            .limb
            .map(|(_, i)| JointRef {
                store: self.store.clone(),
                index: i,
            })
    }

    fn limb(&self) -> Option<Limb> {
        self._store().borrow().joints[self.index]
            .limb
            .map(|(i, _)| {
                self._store().borrow().limbs[i] // TODO: limb ref
            })
    }

    fn basis(&self) -> Vector3<f32> {
        self._store().borrow().joints[self.index].orientation_basis
    }
}

pub struct StickMan {
    debug_info: StrokeSet,

    limb_store: Rc<RefCell<LimbStore>>,

    head_joint: usize,
    right_arm_joint: usize,
    left_arm_joint: usize,
    right_leg_joint: usize,
    left_leg_joint: usize,

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
        let head_joint = limb_store.add_root_joint(Joint::new(vector![0.0, 0.0, 0.0]));
        let right_arm_joint = limb_store.add_root_joint(Joint::new(vector![0.0, 0.0, 1.0]));
        let left_arm_joint = limb_store.add_root_joint(Joint::new(vector![0.0, 0.0, -1.0]));
        let right_leg_joint = limb_store.add_root_joint(Joint::new(vector![0.0, -1.0, 0.0]));
        let left_leg_joint = limb_store.add_root_joint(Joint::new(vector![0.0, -1.0, 0.0]));

        let mut stroke_set = StrokeSet::new();
        stroke_set.set_palette(vec![Style::new(vector![0.0, 0.0, 0.0], 0.2, 0.0)]);

        Self {
            debug_info: stroke_set,

            limb_store: Rc::new(RefCell::new(limb_store)),

            head_joint,
            right_arm_joint,
            left_arm_joint,
            right_leg_joint,
            left_leg_joint,

            to,
            from: to.try_inverse().unwrap(),
        }
    }

    pub fn head_joint(&self) -> JointRef {
        JointRef {
            store: Rc::downgrade(&self.limb_store),
            index: self.head_joint,
        }
    }

    pub fn right_arm_joint(&self) -> JointRef {
        JointRef {
            store: Rc::downgrade(&self.limb_store),
            index: self.right_arm_joint,
        }
    }

    pub fn left_arm_joint(&self) -> JointRef {
        JointRef {
            store: Rc::downgrade(&self.limb_store),
            index: self.left_arm_joint,
        }
    }

    pub fn right_leg_joint(&self) -> JointRef {
        JointRef {
            store: Rc::downgrade(&self.limb_store),
            index: self.right_leg_joint,
        }
    }

    pub fn left_leg_joint(&self) -> JointRef {
        JointRef {
            store: Rc::downgrade(&self.limb_store),
            index: self.left_leg_joint,
        }
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

        self.stroke_joint((a + b) / 2.0, self.from, self.head_joint());
        self.stroke_joint(a, self.from, self.right_arm_joint());
        self.stroke_joint(b, self.from, self.left_arm_joint());
        self.stroke_joint(d, self.from, self.right_leg_joint());
        self.stroke_joint(c, self.from, self.left_leg_joint());
    }

    fn stroke_joint(&mut self, origin: Vector3<f32>, transform: Matrix4<f32>, joint: JointRef) {
        if let Some(limb) = joint.limb() {
            let accum_transform = limb.transform.to_homogeneous() * transform;

            let end =
                origin + (accum_transform.transform_vector(&joint.basis()).normalize() * limb.size);

            self.debug_info.stroke(
                0,
                Stroke::Line {
                    start: origin,
                    end: end,
                },
            );

            self.stroke_joint(end, accum_transform, joint.next_joint().unwrap())
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
