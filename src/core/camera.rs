use crate::input::events::WindowResize;
use angle::{Angle, Deg, Rad};
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use nalgebra::{vector, Matrix, Matrix4, Point3, UnitQuaternion, UnitVector3};

// TODO: look up how to get the euler yaw and pitch angles from a quaternion
#[derive(Resource)]
pub struct Camera {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,

    pub position: Point3<f32>,
    pub orientation: UnitQuaternion<f32>,
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        orientation: UnitQuaternion<f32>,
        aspect_ratio: f32,
        fov: Deg<f32>,
        near: f32,
        far: f32,
    ) -> Self {
        let fov = Rad::from(fov);

        Camera {
            proj: calculate_projection(aspect_ratio, fov, near, far),
            view: calculate_view(position, orientation),

            position,
            orientation,
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.proj[(0, 0)] = -self.proj[(1, 1)].clone() / aspect_ratio;
    }

    pub fn view(&self) -> &Matrix4<f32> {
        &self.view
    }

    pub fn proj(&self) -> &Matrix4<f32> {
        &self.proj
    }

    pub fn update(&mut self) {
        self.view = calculate_view(self.position, self.orientation);
    }

    /// in world-space
    pub const fn forward() -> UnitVector3<f32> {
        UnitVector3::new_unchecked(vector![0.0, 0.0, -1.0])
    }

    /// in world-space
    pub const fn right() -> UnitVector3<f32> {
        UnitVector3::new_unchecked(vector![1.0, 0.0, 0.0])
    }

    /// in world-space
    pub const fn up() -> UnitVector3<f32> {
        UnitVector3::new_unchecked(vector![0.0, 1.0, 0.0])
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

fn calculate_projection(aspect_ratio: f32, fov: Rad<f32>, near: f32, far: f32) -> Matrix4<f32> {
    // https://johannesugb.github.io/gpu-programming/setting-up-a-proper-vulkan-projection-matrix/
    // Note that this projection matrix is already multiplied by the X matrix
    let mut projection = Matrix4::identity();

    let tan_half_fov = (fov.value() / 2.0).tan();

    projection[(0, 0)] = 1.0 / (tan_half_fov * aspect_ratio);
    projection[(1, 1)] = -1.0 / tan_half_fov;

    projection[(2, 2)] = far / (near - far);
    projection[(2, 3)] = (near * far) / (near - far);

    projection[(3, 3)] = 0.0;
    projection[(3, 2)] = -1.0;

    projection
}

fn calculate_view(position: Point3<f32>, orientation: UnitQuaternion<f32>) -> Matrix4<f32> {
    let cam_direction = orientation * Camera::forward();
    let target = position + cam_direction.into_inner();

    Matrix::look_at_rh(&position, &target, &Camera::up())
}
