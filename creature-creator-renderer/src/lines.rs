use nalgebra::Vector3;

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
    pub shape: Shape,
    pub fill: Fill,
    pub thickness: f32,
    pub color: Vector3<f32>,
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
}
