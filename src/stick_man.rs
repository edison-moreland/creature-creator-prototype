use crate::plane::plane_uv;
use crate::renderer::widgets::strokes::{Stroke, StrokeSet, Style};
use crate::renderer::widgets::Widget;
use nalgebra::{vector, Vector2, Vector3};

pub struct LimbSection {
    direction: Vector3<f32>,
    length: f32,
}

impl LimbSection {
    pub fn new(direction: Vector3<f32>, length: f32) -> Self {
        LimbSection { direction, length }
    }
}

pub struct StickMan {
    debug_info: StrokeSet,

    limbs: Vec<(LimbSection, Option<usize>)>,
    head: Option<usize>,
    right_arm: Option<usize>,
    left_arm: Option<usize>,
    right_leg: Option<usize>,
    left_leg: Option<usize>,

    origin: Vector3<f32>,
    normal: Vector3<f32>,

    u: Vector3<f32>,
    v: Vector3<f32>,
    half_size: Vector2<f32>,
}

impl StickMan {
    pub fn new(origin: Vector3<f32>, normal: Vector3<f32>, size: Vector2<f32>) -> Self {
        let (u, v) = plane_uv(origin, normal);

        let mut stroke_set = StrokeSet::new();
        stroke_set.set_palette(vec![Style::new(vector![0.0, 0.0, 0.0], 0.2, 0.0)]);

        let mut sm = Self {
            debug_info: stroke_set,
            limbs: vec![],
            head: None,
            left_arm: None,
            right_arm: None,
            left_leg: None,
            right_leg: None,

            origin,
            normal,

            u,
            v,
            half_size: size / 2.0,
        };

        sm.update_strokes();

        sm
    }

    pub fn attach_head(&mut self, l: LimbSection) {
        self.head = Some(self.new_limb(l));
        self.update_strokes()
    }

    pub fn attach_right_arm(&mut self, l: LimbSection) {
        self.right_arm = Some(self.new_limb(l));
        self.update_strokes()
    }
    pub fn attach_left_arm(&mut self, l: LimbSection) {
        self.left_arm = Some(self.new_limb(l));
        self.update_strokes()
    }
    pub fn attach_right_leg(&mut self, l: LimbSection) {
        self.right_leg = Some(self.new_limb(l));
        self.update_strokes()
    }
    pub fn attach_left_leg(&mut self, l: LimbSection) {
        self.left_leg = Some(self.new_limb(l));
        self.update_strokes()
    }

    fn new_limb(&mut self, limb: LimbSection) -> usize {
        let limb_idx = self.limbs.len();
        self.limbs.push((limb, None));

        limb_idx
    }

    fn stroke_limb(&mut self, attachment: Vector3<f32>, limb: Option<usize>) {
        if let Some(i) = limb {
            let (l, _) = &self.limbs[i];

            self.debug_info.stroke(
                0,
                Stroke::Line {
                    start: attachment,
                    end: attachment + (l.direction * l.length),
                },
            )
        }
    }

    fn update_strokes(&mut self) {
        let a = (self.v * self.half_size.x) + (self.u * self.half_size.y);
        let b = (self.v * -self.half_size.x) + (self.u * self.half_size.y);
        let c = (self.v * -self.half_size.x) + (self.u * -self.half_size.y);
        let d = (self.v * self.half_size.x) + (self.u * -self.half_size.y);

        self.debug_info.clear();
        self.debug_info.stroke(0, Stroke::Line { start: a, end: b });
        self.debug_info.stroke(0, Stroke::Line { start: b, end: c });
        self.debug_info.stroke(0, Stroke::Line { start: c, end: d });
        self.debug_info.stroke(0, Stroke::Line { start: d, end: a });

        self.debug_info.stroke(
            0,
            Stroke::Arrow {
                origin: self.origin,
                direction: self.normal,
                magnitude: 5.0,
            },
        );

        self.stroke_limb((a + b) / 2.0, self.head);
        self.stroke_limb(a, self.right_arm);
        self.stroke_limb(b, self.left_arm);
        self.stroke_limb(d, self.right_leg);
        self.stroke_limb(c, self.left_leg);
    }
}

impl Widget for StickMan {
    fn strokes(&self) -> &StrokeSet {
        &self.debug_info
    }
}
