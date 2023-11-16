use nalgebra::{point, vector, Matrix4, Vector3};

use crate::geometry::Plane;
use crate::renderer::graph::NodeTransform;
use crate::renderer::lines::pipeline::LineSegment;

pub mod pipeline;

pub enum Shape {
    // A regular line with it's origin in the middle
    None { length: f32 },
    // A line with an arrow cap, it's origin is at the non-arrow end
    Arrow { magnitude: f32 },
    // A circle with it's origin in the center
    Circle { radius: f32 },
}

pub enum Fill {
    Solid,
    Dashed(f32),
}

pub struct Line {
    shape: Shape,
    fill: Fill,
    thickness: f32,
    color: Vector3<f32>,
}

impl Line {
    pub fn new(length: f32) -> Self {
        Self {
            shape: Shape::None { length },
            fill: Fill::Solid,
            thickness: 0.1,
            color: Default::default(),
        }
    }
    pub fn new_arrow(magnitude: f32) -> Self {
        Self {
            shape: Shape::Arrow { magnitude },
            fill: Fill::Solid,
            thickness: 0.1,
            color: Default::default(),
        }
    }

    pub fn new_circle(radius: f32) -> Self {
        Self {
            shape: Shape::Circle { radius },
            fill: Fill::Solid,
            thickness: 0.1,
            color: Default::default(),
        }
    }

    pub fn fill(mut self, fill: Fill) -> Self {
        self.fill = fill;
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn color(mut self, color: Vector3<f32>) -> Self {
        self.color = color;
        self
    }

    pub(crate) fn line_segments(&self, segments: &mut Vec<LineSegment>, transform: &Matrix4<f32>) {
        match self.shape {
            Shape::None { length } => self.shape_none_segments(segments, transform, length),
            Shape::Arrow { magnitude } => self.shape_arrow_segments(segments, transform, magnitude),
            Shape::Circle { radius } => self.shape_circle_segments(segments, transform, radius),
        }
    }

    fn dash_size(&self) -> f32 {
        match self.fill {
            Fill::Solid => 0.0,
            Fill::Dashed(d) => d,
        }
    }

    fn shape_none_segments(
        &self,
        segments: &mut Vec<LineSegment>,
        transform: &Matrix4<f32>,
        length: f32,
    ) {
        let start = point![0.0, length / 2.0, 0.0];
        let end = point![0.0, -(length / 2.0), 0.0];

        segments.push(LineSegment::new(
            transform.transform_point(&start),
            transform.transform_point(&end),
            self.color,
            self.thickness,
            self.dash_size(),
            0,
            0.0,
        ))
    }

    fn shape_arrow_segments(
        &self,
        segments: &mut Vec<LineSegment>,
        transform: &Matrix4<f32>,
        magnitude: f32,
    ) {
        let direction = transform
            .transform_vector(&vector![0.0, 1.0, 0.0])
            .normalize();
        let origin = transform.transform_point(&point![0.0, 0.0, 0.0]);

        let start = origin;
        let end = start + (direction * magnitude);

        let stem_thickness = self.thickness;
        let arrow_thickness = stem_thickness * 4.0;
        let arrow_head_length = arrow_thickness * 1.5;

        if magnitude <= arrow_head_length {
            segments.push(LineSegment::new(
                start,
                end,
                self.color,
                arrow_thickness,
                0.0,
                1,
                0.0,
            ));
        } else {
            let stem_length = magnitude - arrow_head_length;
            let stem_end = start + (direction * stem_length);

            segments.push(LineSegment::new(
                start,
                stem_end,
                self.color,
                stem_thickness,
                self.dash_size(),
                0,
                0.0,
            ));
            segments.push(LineSegment::new(
                stem_end,
                end,
                self.color,
                arrow_thickness,
                0.0,
                1,
                0.0,
            ));
        }
    }

    fn shape_circle_segments(
        &self,
        segments: &mut Vec<LineSegment>,
        transform: &Matrix4<f32>,
        radius: f32,
    ) {
        let segment_count = 24 * 2; // TODO: Scale segment_count based on final radius/dash size
        let points = Plane::from_origin_normal(point![0.0, 0.0, 0.0], vector![0.0, 1.0, 0.0])
            .circle_points(segment_count, radius);

        let mut length = 0.0;
        for i in 0..segment_count {
            let last_i = if i == 0 { segment_count - 1 } else { i - 1 };

            let a = transform.transform_point(&points[i]);
            let b = transform.transform_point(&points[last_i]);

            segments.push(LineSegment::new(
                a,
                b,
                self.color,
                self.thickness,
                self.dash_size(),
                0,
                length,
            ));

            length += (a - b).magnitude();
        }
    }
}
