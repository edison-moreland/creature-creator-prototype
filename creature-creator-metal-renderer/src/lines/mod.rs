use nalgebra::{point, vector, Matrix4};

use creature_creator_renderer::lines::{Fill, Line, Shape};

use crate::geometry::Plane;
use crate::lines::pipeline::LineSegment;

pub mod pipeline;

pub(crate) fn line_segments(
    line: &Line,
    segments: &mut Vec<LineSegment>,
    transform: &Matrix4<f32>,
) {
    match line.shape {
        Shape::None { length } => shape_none_segments(line, segments, transform, length),
        Shape::Arrow { magnitude } => shape_arrow_segments(line, segments, transform, magnitude),
        Shape::Circle { radius } => shape_circle_segments(line, segments, transform, radius),
    }
}

fn dash_size(fill: &Fill) -> f32 {
    match fill {
        Fill::Solid => 0.0,
        Fill::Dashed(d) => *d,
    }
}

fn shape_none_segments(
    line: &Line,
    segments: &mut Vec<LineSegment>,
    transform: &Matrix4<f32>,
    length: f32,
) {
    let start = point![0.0, length / 2.0, 0.0];
    let end = point![0.0, -(length / 2.0), 0.0];

    segments.push(LineSegment::new(
        transform.transform_point(&start),
        transform.transform_point(&end),
        line.color,
        line.thickness,
        dash_size(&line.fill),
        0,
        0.0,
    ))
}

fn shape_arrow_segments(
    line: &Line,
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

    let stem_thickness = line.thickness;
    let arrow_thickness = stem_thickness * 4.0;
    let arrow_head_length = arrow_thickness * 1.5;

    if magnitude <= arrow_head_length {
        segments.push(LineSegment::new(
            start,
            end,
            line.color,
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
            line.color,
            stem_thickness,
            dash_size(&line.fill),
            0,
            0.0,
        ));
        segments.push(LineSegment::new(
            stem_end,
            end,
            line.color,
            arrow_thickness,
            0.0,
            1,
            0.0,
        ));
    }
}

fn shape_circle_segments(
    line: &Line,
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
            line.color,
            line.thickness,
            dash_size(&line.fill),
            0,
            length,
        ));

        length += (a - b).magnitude();
    }
}
