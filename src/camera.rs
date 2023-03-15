use crate::input::WindowResize;
use angle::{Angle, Deg, Rad};
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{ResMut, Resource};
use nalgebra::{
    vector, Matrix, Matrix4, Perspective3, Point3, UnitQuaternion, Vector, Vector3, Vector4,
};

#[derive(Resource)]
pub struct Camera {
    view: Matrix4<f32>,
    proj: Perspective3<f32>,

    pub position: Point3<f32>,
    pub orientation: UnitQuaternion<f32>,
}

impl Camera {
    pub fn new(fov: Deg<f32>, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let position = Point3::new(0.0, 1.0, -6.0);
        let orientation = UnitQuaternion::identity();

        let fov = Rad::from(fov);

        Camera {
            view: calculate_view(position, orientation),
            proj: Perspective3::new(aspect_ratio, fov.value(), near, far),
            position,
            orientation,
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.proj.set_aspect(aspect_ratio);
    }

    pub fn view(&self) -> &Matrix4<f32> {
        &self.view
    }

    pub fn proj(&self) -> Matrix4<f32> {
        // https://johannesugb.github.io/gpu-programming/setting-up-a-proper-vulkan-projection-matrix/
        let flip_yz_matrix: Matrix4<f32> =
            Matrix4::from_diagonal(&Vector4::new(1.0, -1.0, -1.0, 1.0));
        let matrix = self.proj.as_matrix() * flip_yz_matrix;
        matrix
    }

    pub fn update(&mut self) {
        self.view = calculate_view(self.position, self.orientation);
    }

    /// in world-space
    pub const fn forward() -> Vector3<f32> {
        vector![0.0, 0.0, -1.0]
    }

    /// in world-space
    pub const fn right() -> Vector3<f32> {
        vector![1.0, 0.0, 0.0]
    }

    /// in world-space
    pub const fn up() -> Vector3<f32> {
        vector![0.0, 1.0, 0.0]
    }
}

pub fn update_camera(mut camera: ResMut<Camera>) {
    camera.update();
}

pub fn update_camera_aspect_ratio(
    mut camera: ResMut<Camera>,
    mut reader: EventReader<WindowResize>,
) {
    for event in reader.iter() {
        camera.update_aspect_ratio(event.width as f32 / event.height as f32);
    }
}

fn calculate_view(position: Point3<f32>, orientation: UnitQuaternion<f32>) -> Matrix4<f32> {
    // TODO: check if camera forward is correct here
    let cam_direction = orientation * Camera::forward();
    let target = position + cam_direction;

    Matrix::look_at_rh(&position, &target, &Camera::up())
}
