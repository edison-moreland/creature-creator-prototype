use nalgebra::{Point3, Transform3, Vector3};

use crate::geometry::Plane;
use crate::renderer::graph::Transform;
use crate::renderer::lines::pipeline::LineSegment;

pub struct Style {
    dash_size: f32,
    // 0 for no dashes
    thickness: f32,
    color: Vector3<f32>,
}

impl Style {
    pub(crate) fn new(color: Vector3<f32>, thickness: f32, dash_size: f32) -> Self {
        Style {
            color,
            thickness,
            dash_size,
        }
    }
}

pub enum Stroke {
    Line {
        start: Point3<f32>,
        end: Point3<f32>,
    },
    Arrow {
        origin: Point3<f32>,
        direction: Vector3<f32>,
        magnitude: f32,
    },
    Circle {
        origin: Point3<f32>,
        normal: Vector3<f32>,
        radius: f32,
    },
}

impl Stroke {
    pub(crate) fn segments(
        &self,
        transform: Transform,
        segments: &mut Vec<LineSegment>,
        style: &Style,
    ) {
        match *self {
            Self::Line { start, end } => {
                segments.push(LineSegment::new(
                    transform.apply_point(&start),
                    transform.apply_point(&end),
                    style.color,
                    style.thickness,
                    style.dash_size,
                    0,
                ));
            }
            Self::Circle {
                origin,
                normal,
                radius,
            } => {
                let segment_count = 24;
                let points = Plane::from_origin_normal(
                    transform.apply_point(&origin),
                    transform.apply_vector(&normal),
                )
                .circle_points(segment_count, radius);

                for i in 0..segment_count {
                    let last_i = if i == 0 { segment_count - 1 } else { i - 1 };

                    segments.push(LineSegment::new(
                        points[last_i],
                        points[i],
                        style.color,
                        style.thickness,
                        style.dash_size,
                        0,
                    ));
                }
            }
            Self::Arrow {
                origin,
                direction,
                magnitude,
            } => {
                let start = transform.apply_point(&origin);
                let end = start + (transform.apply_vector(&direction) * magnitude);

                let stem_thickness = style.thickness;
                let arrow_thickness = stem_thickness * 4.0;
                let arrow_head_length = arrow_thickness * 1.5;

                if magnitude <= arrow_head_length {
                    segments.push(LineSegment::new(
                        start,
                        end,
                        style.color,
                        arrow_thickness,
                        0.0,
                        1,
                    ));
                } else {
                    let stem_length = magnitude - arrow_head_length;
                    let stem_end = start + (direction * stem_length);

                    segments.push(LineSegment::new(
                        start,
                        stem_end,
                        style.color,
                        stem_thickness,
                        style.dash_size,
                        0,
                    ));
                    segments.push(LineSegment::new(
                        stem_end,
                        end,
                        style.color,
                        arrow_thickness,
                        0.0,
                        1,
                    ));
                }
            }
        }
    }
}
