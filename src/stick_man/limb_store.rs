use nalgebra::{Rotation3, Vector3};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Copy, Clone)]
pub struct Limb {
    pub size: f32,
    pub transform: Rotation3<f32>,
}

impl Limb {
    pub fn new(rotation: Rotation3<f32>, size: f32) -> Self {
        Limb {
            size,
            transform: rotation,
        }
    }
}

pub struct Joint {
    limb: Option<(usize, usize)>, // Limb and next joint
    orientation_basis: Vector3<f32>,
}

impl Joint {
    pub fn new(basis: Vector3<f32>) -> Self {
        Self {
            limb: None,
            orientation_basis: basis,
        }
    }
}

struct _LimbStore {
    limbs: Vec<Limb>,
    joints: Vec<Joint>,
}

impl _LimbStore {
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

pub struct LimbStore {
    _store: Rc<RefCell<_LimbStore>>,
}

impl LimbStore {
    pub fn new() -> Self {
        Self {
            _store: Rc::new(RefCell::new(_LimbStore {
                limbs: vec![],
                joints: vec![],
            })),
        }
    }

    pub fn add_root_joint(&mut self, joint: Joint) -> JointRef {
        let joint_idx = {
            let mut store = self._store.borrow_mut();
            let i = store.joints.len();
            store.joints.push(joint);

            i
        };

        JointRef {
            store: Rc::downgrade(&self._store),
            index: joint_idx,
        }
    }
}

#[derive(Clone)]
pub struct JointRef {
    store: Weak<RefCell<_LimbStore>>,
    index: usize,
}

impl JointRef {
    fn _store(&self) -> Rc<RefCell<_LimbStore>> {
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

    pub fn limb(&self) -> Option<Limb> {
        self._store().borrow().joints[self.index]
            .limb
            .map(|(i, _)| {
                self._store().borrow().limbs[i] // TODO: limb ref
            })
    }

    pub fn basis(&self) -> Vector3<f32> {
        self._store().borrow().joints[self.index].orientation_basis
    }
}
