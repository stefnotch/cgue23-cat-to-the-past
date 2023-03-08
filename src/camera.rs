use crate::input::WindowResize;
use angle::{Angle, Deg, Rad};
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{ResMut, Resource};
use nalgebra::{Matrix, Matrix4, Perspective3, Point3, Vector3};

#[derive(Resource)]
pub struct Camera {
    view: Matrix4<f32>,
    proj: Perspective3<f32>,

    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl Camera {
    pub fn new(fov: Deg<f32>, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let position = Point3::new(0.0, 0.0, -6.0);

        let fov = Rad::from(fov);

        let yaw = Rad(0.0);
        let pitch = Rad(0.0);

        Camera {
            view: calculate_view(position, yaw, pitch),
            proj: Perspective3::new(aspect_ratio, fov.value(), near, far),
            position,
            yaw,
            pitch,
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.proj.set_aspect(aspect_ratio);
    }

    pub fn view(&self) -> &Matrix4<f32> {
        &self.view
    }

    pub fn proj(&self) -> &Matrix4<f32> {
        &self.proj.as_matrix()
    }

    pub fn update(&mut self) {
        self.view = calculate_view(self.position, self.yaw, self.pitch);
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

fn calculate_view(position: Point3<f32>, yaw: Rad<f32>, pitch: Rad<f32>) -> Matrix4<f32> {
    // See: https://sotrh.github.io/learn-wgpu/intermediate/tutorial12-camera/#the-camera
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    let (sin_yaw, cos_yaw) = yaw.sin_cos();

    let cam_direction =
        Vector3::new(cos_pitch * sin_yaw, -sin_pitch, cos_pitch * cos_yaw).normalize();

    let target = position + cam_direction;

    let inv_unit_y = Vector3::new(0.0, -1.0, 0.0);

    Matrix::look_at_rh(&position, &target, &inv_unit_y)
}
