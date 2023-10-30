use nalgebra::Vector3;

use crate::plane::Plane;
use crate::renderer::widgets::pipeline::LineSegment;

pub mod pipeline;

pub enum Widget {
    Line {
        start: Vector3<f32>,
        end: Vector3<f32>,
        color: Vector3<f32>,
    },
    Circle {
        origin: Vector3<f32>,
        normal: Vector3<f32>,
        radius: f32,
        color: Vector3<f32>,
    },
    Arrow {
        origin: Vector3<f32>,
        direction: Vector3<f32>,
        magnitude: f32,
        color: Vector3<f32>,
    },
}

impl Widget {
    fn segments(&self, segments: &mut Vec<LineSegment>) {
        match *self {
            Widget::Line { start, end, color } => {
                segments.push(LineSegment::new(start, end, color, 0.1, 0.0, 0));
            }
            Widget::Circle {
                origin,
                color,
                normal,
                radius,
            } => {
                let segment_count = 24;
                let points =
                    Plane::from_origin_normal(origin, normal).circle_points(segment_count, radius);

                for i in 0..segment_count {
                    let last_i = if i == 0 { segment_count - 1 } else { i - 1 };

                    segments.push(LineSegment::new(
                        points[last_i],
                        points[i],
                        color,
                        0.1,
                        0.0,
                        0,
                    ));
                }
            }
            Widget::Arrow {
                origin,
                direction,
                magnitude,
                color,
            } => {
                let start = origin;
                let end = start + (direction * magnitude);

                let stem_thickness = 0.2;
                let arrow_thickness = stem_thickness * 4.0;
                let arrow_head_length = arrow_thickness * 1.5;

                if magnitude <= arrow_head_length {
                    segments.push(LineSegment::new(start, end, color, arrow_thickness, 0.0, 1));
                } else {
                    let stem_length = magnitude - arrow_head_length;
                    let stem_end = start + (direction * stem_length);

                    segments.push(LineSegment::new(
                        start,
                        stem_end,
                        color,
                        stem_thickness,
                        0.0,
                        0,
                    ));
                    segments.push(LineSegment::new(
                        stem_end,
                        end,
                        color,
                        arrow_thickness,
                        0.0,
                        1,
                    ));
                }
            }
        }
    }
}
