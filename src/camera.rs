use crate::input::WindowResize;
use angle::{Angle, Deg, Rad};
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{ResMut, Resource};
use nalgebra::{Matrix, Matrix4, Perspective3, Point3, UnitQuaternion, Vector, Vector3};

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

    pub fn proj(&self) -> &Matrix4<f32> {
        &self.proj.as_matrix()
    }

    pub fn update(&mut self) {
        self.view = calculate_view(self.position, self.orientation);
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
    let cam_direction = orientation * Vector::z_axis();
    let target = position + cam_direction.into_inner();

    let inv_unit_y = Vector3::new(0.0, -1.0, 0.0);

    Matrix::look_at_rh(&position, &target, &inv_unit_y)
}
