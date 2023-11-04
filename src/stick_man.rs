use crate::renderer::widgets::strokes::{Stroke, StrokeSet, Style};
use crate::renderer::widgets::Widget;
use nalgebra::{point, vector, Matrix4, Rotation3, Scale3, Translation3, Vector2, Vector3};

pub struct StickMan {
    debug_info: StrokeSet,

    to: Matrix4<f32>,
    from: Matrix4<f32>,
}

impl StickMan {
    pub fn new(origin: Vector3<f32>, normal: Vector3<f32>, size: Vector2<f32>) -> Self {
        let t = Translation3::new(-origin.x, -origin.y, -origin.z);
        let s = Scale3::new(2.0 / size.x, 2.0 / size.y, 2.0 / ((size.x + size.y) / 4.0));
        let r = Rotation3::rotation_between(&normal, &vector![1.0, 0.0, 0.0]).unwrap();

        let to = s.to_homogeneous() * r.to_homogeneous() * t.to_homogeneous();

        let mut stroke_set = StrokeSet::new();
        stroke_set.set_palette(vec![Style::new(vector![0.0, 0.0, 0.0], 0.2, 0.0)]);

        let mut sm = Self {
            debug_info: stroke_set,

            to,
            from: to.try_inverse().unwrap(),
        };

        sm.update_strokes();
        sm
    }

    fn update_strokes(&mut self) {
        let a = self.from.transform_point(&point![0.0, 1.0, 1.0]).coords;
        let b = self.from.transform_point(&point![0.0, 1.0, -1.0]).coords;
        let c = self.from.transform_point(&point![0.0, -1.0, -1.0]).coords;
        let d = self.from.transform_point(&point![0.0, -1.0, 1.0]).coords;

        self.debug_info.clear();
        self.stroke_core(a, b, c, d);
        self.debug_info.stroke(
            0,
            Stroke::Arrow {
                origin: self.from.transform_point(&point![0.0, 0.0, 0.0]).coords,
                direction: self
                    .from
                    .transform_vector(&vector![1.0, 0.0, 0.0])
                    .normalize(),
                magnitude: 4.0,
            },
        );
    }

    fn stroke_core(&mut self, a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>, d: Vector3<f32>) {
        self.debug_info.stroke(0, Stroke::Line { start: a, end: b });
        self.debug_info.stroke(0, Stroke::Line { start: b, end: c });
        self.debug_info.stroke(0, Stroke::Line { start: c, end: d });
        self.debug_info.stroke(0, Stroke::Line { start: d, end: a });
    }
}

impl Widget for StickMan {
    fn strokes(&self) -> &StrokeSet {
        &self.debug_info
    }
}
