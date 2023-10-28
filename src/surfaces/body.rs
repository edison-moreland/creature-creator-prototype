use nalgebra::{matrix, Vector3};

// A core is at the center of a body. It's defined by 4 co-planer points
// Limbs attach to the body at it's 4 corners
struct Core {
    a: Vector3<f32>,
    b: Vector3<f32>,
    c: Vector3<f32>,
    d: Vector3<f32>,
}

impl Core {
    fn new(a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>, d: Vector3<f32>) -> Self {
        // Expect point order
        // A +----+ D
        //   |    |
        // B +----+ C
        if matrix![a.x, a.y, a.z, 1.0;
                   b.x, b.y, b.z, 1.0;
                   c.x, c.y, c.z, 1.0;
                   d.x, d.y, d.z, 1.0]
        .determinant()
            != 0.0
        {
            panic!("Points are not coplanar!!")
        }

        // This makes sure the points form a rectangle, a requirement i'd like to relax
        let mid_ac = (a + c) / 2.0;
        let mid_bd = (b + d) / 2.0;
        assert_eq!(mid_ac, mid_bd);
        let origin = mid_ac;

        // Setup transform for children
        // In the core space:
        // (-1,  1, 0)       (1,  1, 0)
        //        A +---------+ D
        //          |         |
        //          |    + (0, 0, 0)
        //          |         |
        //        B +---------+ C
        // (-1, -1, 0)       (1, -1, 0)

        Self { a, b, c, d }
    }
}

#[cfg(test)]
mod test {
    use crate::surfaces::body::Core;
    use nalgebra::vector;

    #[test]
    fn are_coplanar() {
        Core::new(
            vector![-1.0, 1.0, 0.0],
            vector![-1.0, -1.0, 0.0],
            vector![1.0, -1.0, 0.0],
            vector![1.0, 1.0, 0.0],
        );
    }

    #[test]
    #[should_panic]
    fn are_not_coplanar() {
        Core::new(
            vector![-1.0, 1.0, 5.0],
            vector![-1.0, -1.0, -5.0],
            vector![1.0, -1.0, 5.0],
            vector![1.0, 1.0, -5.0],
        );
    }
}
