use crate::renderer::graph::Transform;
use nalgebra::{point, vector};

use crate::renderer::lines::pipeline::LineSegment;
pub use crate::renderer::strokes::{Stroke, Style};

pub mod pipeline;

pub struct Widget {
    styles: Vec<Style>,
    strokes: Vec<(Stroke, usize)>,
}

impl Widget {
    pub fn new() -> Self {
        Widget {
            styles: vec![],
            strokes: vec![],
        }
    }

    pub fn new_with<F>(f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        let mut w = Self::new();

        f(&mut w);

        w
    }

    pub fn set_palette(&mut self, styles: Vec<Style>) {
        self.styles = styles
    }

    pub fn stroke(&mut self, style: usize, stroke: Stroke) {
        self.strokes.push((stroke, style))
    }

    pub fn line_segments(&self, transform: Transform, segments: &mut Vec<LineSegment>) {
        for (stroke, style_idx) in &self.strokes {
            stroke.segments(transform, segments, &self.styles[*style_idx])
        }
    }
}

pub struct Grid(Widget);

pub fn grid(size: f32, step: f32) -> Widget {
    let mut widget = Widget::new();
    widget.set_palette(vec![Style::new(vector![0.0, 0.0, 0.0], 0.1, 0.0)]);

    let start = -(size / 2.0);

    let mut grid_line_position = start;
    while grid_line_position <= -start {
        widget.stroke(
            0,
            Stroke::Line {
                start: point![grid_line_position, 0.0, -start],
                end: point![grid_line_position, 0.0, start],
            },
        );
        widget.stroke(
            0,
            Stroke::Line {
                start: point![-start, 0.0, grid_line_position],
                end: point![start, 0.0, grid_line_position],
            },
        );
        grid_line_position += step
    }

    widget
}

pub fn cardinal_arrows(magnitude: f32) -> Widget {
    let mut widget = Widget::new();
    widget.set_palette(vec![
        Style::new(vector![1.0, 0.0, 0.0], 0.2, 0.0),
        Style::new(vector![0.0, 1.0, 0.0], 0.2, 0.0),
        Style::new(vector![0.0, 0.0, 1.0], 0.2, 0.0),
    ]);

    widget.stroke(
        0,
        Stroke::Arrow {
            direction: vector![1.0, 0.0, 0.0],
            origin: point![0.0, 0.0, 0.0],
            magnitude,
        },
    );

    widget.stroke(
        1,
        Stroke::Arrow {
            direction: vector![0.0, 1.0, 0.0],
            origin: point![0.0, 0.0, 0.0],
            magnitude,
        },
    );

    widget.stroke(
        2,
        Stroke::Arrow {
            direction: vector![0.0, 0.0, 1.0],
            origin: point![0.0, 0.0, 0.0],
            magnitude,
        },
    );

    widget
}
