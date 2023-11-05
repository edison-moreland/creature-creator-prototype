use nalgebra::{Point3, Vector3};

use crate::plane::Plane;
use crate::renderer::widgets::pipeline::LineSegment;

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
    fn segments(&self, segments: &mut Vec<LineSegment>, style: &Style) {
        match *self {
            Self::Line { start, end } => {
                segments.push(LineSegment::new(
                    start,
                    end,
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
                let points =
                    Plane::from_origin_normal(origin, normal).circle_points(segment_count, radius);

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
                let start = origin;
                let end = start + (direction * magnitude);

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

pub struct StrokeSet {
    styles: Vec<Style>,
    strokes: Vec<(Stroke, usize)>,
}

impl StrokeSet {
    pub fn new() -> Self {
        StrokeSet {
            styles: vec![],
            strokes: vec![],
        }
    }

    pub fn set_palette(&mut self, styles: Vec<Style>) {
        self.styles = styles
    }

    pub fn stroke(&mut self, style: usize, stroke: Stroke) {
        self.strokes.push((stroke, style))
    }

    pub fn line_segments(&self, segments: &mut Vec<LineSegment>) {
        for (stroke, style_idx) in &self.strokes {
            stroke.segments(segments, &self.styles[*style_idx])
        }
    }
}
