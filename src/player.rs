use crate::application::{AppStage, ApplicationBuilder};
use crate::camera::Camera;
use crate::input::{handle_keyboard_input, InputMap, MouseMovement};
use crate::time::Time;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::Resource;
use bevy_ecs::system::{Res, ResMut};
use cgmath::{Deg, InnerSpace, Rad, Vector3};
use std::f32::consts::FRAC_PI_2;
use winit::event::VirtualKeyCode;

#[derive(Resource)]
pub struct PlayerSettings {
    speed: f32,
    sensitivity: f32,
}

impl PlayerSettings {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        PlayerSettings { speed, sensitivity }
    }
}

pub fn handle_mouse_movement(
    mut reader: EventReader<MouseMovement>,
    camera: ResMut<Camera>,
    settings: Res<PlayerSettings>,
) {
    for event in reader.iter() {
        let MouseMovement(dx, dy) = *event;

        camera.yaw += Deg(dx as f32 * settings.sensitivity).into();
        camera.pitch += Deg(dy as f32 * settings.sensitivity).into();

        const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}

pub fn update_camera(
    camera: ResMut<Camera>,
    input: Res<InputMap>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
) {
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

    let delta_time = time.delta_seconds as f32;

    camera.position += forward * direction.z * settings.speed * delta_time;
    camera.position += right * direction.x * settings.speed * delta_time;
    camera.position += up * direction.y * settings.speed * delta_time;
}

impl ApplicationBuilder {
    pub fn with_player_controller(mut self, settings: PlayerSettings) -> Self {
        self.with_resource(settings)
            .with_system(AppStage::Update, handle_keyboard_input)
            .with_system(AppStage::Update, update_camera)
    }
}
