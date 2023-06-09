use nalgebra::Vector3;

#[derive(Clone, Debug, PartialEq)]
pub struct BoundingBox<T> {
    /// Minimum value.
    pub min: T,

    /// Maximum value.
    pub max: T,
}

impl<T> BoundingBox<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl BoundingBox<Vector3<f32>> {
    pub fn combine(&self, other: &BoundingBox<Vector3<f32>>) -> BoundingBox<Vector3<f32>> {
        BoundingBox {
            min: self.min.inf(&other.min),
            max: self.max.sup(&other.max),
        }
    }

    pub fn empty() -> BoundingBox<Vector3<f32>> {
        BoundingBox {
            min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vector3::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY),
        }
    }

    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }

    pub fn scale(&self, scaling: &Vector3<f32>) -> BoundingBox<Vector3<f32>> {
        BoundingBox {
            min: self.min.component_mul(scaling),
            max: self.max.component_mul(scaling),
        }
    }

    pub fn bounding_sphere(&self) -> (Vector3<f32>, f32) {
        let center = (self.min + self.max) / 2.0;
        let radius = (self.max - self.min).norm() / 2.0;
        (center, radius)
    }
}
