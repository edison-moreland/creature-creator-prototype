use nalgebra::{point, vector, Point3, Vector3};

use crate::renderer::widgets::strokes::{Stroke, StrokeSet, Style};

pub mod pipeline;
pub mod strokes;

pub trait Widget {
    fn strokes(&self) -> Option<&StrokeSet>;
}

pub struct Grid(StrokeSet);

impl Grid {
    pub fn new(size: f32, step: f32) -> Self {
        let mut stroke_set = StrokeSet::new();
        stroke_set.set_palette(vec![Style::new(vector![0.0, 0.0, 0.0], 0.1, 0.0)]);

        let start = -(size / 2.0);

        let mut grid_line_position = start;
        while grid_line_position <= -start {
            stroke_set.stroke(
                0,
                Stroke::Line {
                    start: point![grid_line_position, 0.0, -start],
                    end: point![grid_line_position, 0.0, start],
                },
            );
            stroke_set.stroke(
                0,
                Stroke::Line {
                    start: point![-start, 0.0, grid_line_position],
                    end: point![start, 0.0, grid_line_position],
                },
            );
            grid_line_position += step
        }

        Self(stroke_set)
    }
}

impl Widget for Grid {
    fn strokes(&self) -> Option<&StrokeSet> {
        Some(&self.0)
    }
}

pub struct CardinalArrows(StrokeSet);

impl CardinalArrows {
    pub fn new(origin: Point3<f32>, magnitude: f32) -> Self {
        let mut stroke_set = StrokeSet::new();
        stroke_set.set_palette(vec![
            Style::new(vector![1.0, 0.0, 0.0], 0.2, 0.0),
            Style::new(vector![0.0, 1.0, 0.0], 0.2, 0.0),
            Style::new(vector![0.0, 0.0, 1.0], 0.2, 0.0),
        ]);

        stroke_set.stroke(
            0,
            Stroke::Arrow {
                direction: vector![1.0, 0.0, 0.0],
                origin,
                magnitude,
            },
        );

        stroke_set.stroke(
            1,
            Stroke::Arrow {
                direction: vector![0.0, 1.0, 0.0],
                origin,
                magnitude,
            },
        );

        stroke_set.stroke(
            2,
            Stroke::Arrow {
                direction: vector![0.0, 0.0, 1.0],
                origin,
                magnitude,
            },
        );

        Self(stroke_set)
    }
}

impl Widget for CardinalArrows {
    fn strokes(&self) -> Option<&StrokeSet> {
        Some(&self.0)
    }
}
