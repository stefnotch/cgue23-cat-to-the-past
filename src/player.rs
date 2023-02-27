use crate::camera::Camera;
use crate::input::InputMap;
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{Res, ResMut};
use cgmath::{Deg, Rad};
use rapier3d::na::Vector3;
use winit::event::VirtualKeyCode;

pub struct PlayerController {
    speed: f32,
    sensitivity: f32,
}

impl PlayerController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        PlayerController { speed, sensitivity }
    }

    pub fn handle_mouse_movement(
        &self,
        mut reader: EventReader<MouseMovement>,
        camera: ResMut<Camera>,
    ) {
        for (event) in reader.iter() {}
        camera.yaw += Deg(dx as f32 * self.sensitivity).into();
        camera.pitch += Deg(dy as f32 * self.sensitivity).into();

        const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }

    pub fn update_camera(&mut self, camera: ResMut<Camera>, input: Res<InputMap>, delta_time: f64) {
        let mut direction: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        if input.is_pressed(VirtualKeyCode::W) {
            direction.z += 1.0;
        }
        if input.is_pressed(VirtualKeyCode::S) {
            direction.z += -1.0;
        }

        if input.is_pressed(VirtualKeyCode::A) {
            direction.x += -1.0;
        }
        if input.is_pressed(VirtualKeyCode::D) {
            direction.x += 1.0;
        }

        if input.is_pressed(VirtualKeyCode::Space) {
            direction.y += 1.0;
        }
        if input.is_pressed(VirtualKeyCode::LShift) {
            direction.y += -1.0;
        }

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_sin, 0.0, yaw_cos).normalize();
        let right = Vector3::new(yaw_cos, 0.0, -yaw_sin).normalize();
        let up = Vector3::unit_y();

        camera.position += forward * direction.z * self.speed * delta_time as f32;
        camera.position += right * direction.x * self.speed * delta_time as f32;
        camera.position += up * direction.y * self.speed * delta_time as f32;
    }
}
