use crate::application::{AppStage, ApplicationBuilder};
use crate::camera::Camera;
use crate::input::{InputMap, MouseMovement};
use crate::physics_context::CharacterController;
use crate::scene::transform::{Transform, TransformBuilder};
use crate::time::Time;
use angle::{Deg, Rad};
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use bevy_ecs::system::{Res, ResMut};
use nalgebra::{Point3, UnitQuaternion, Vector3};
use std::f32::consts::FRAC_PI_2;
use winit::event::VirtualKeyCode;

#[derive(Resource)]
pub struct PlayerSettings {
    speed: f32,
    /// players use a different gravity
    gravity: f32,
    sensitivity: f32,
}

#[derive(Resource)]
pub struct Player {
    pub desired_movement: Vector3<f32>,
}

impl PlayerSettings {
    pub fn new(speed: f32, sensitivity: f32, gravity: f32) -> Self {
        PlayerSettings {
            speed,
            sensitivity,
            gravity,
        }
    }
}

pub fn handle_mouse_movement(
    mut reader: EventReader<MouseMovement>,
    mut camera: ResMut<Camera>,
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

pub fn update_camera_position(
    mut camera: ResMut<Camera>,
    input: Res<InputMap>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
) {
    let direction = input_to_direction(&input);

    let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
    let forward = Vector3::new(yaw_sin, 0.0, yaw_cos).normalize();
    let right = Vector3::new(yaw_cos, 0.0, -yaw_sin).normalize();
    let up = Vector3::new(0.0, 1.0, 0.0);

    let delta_time = time.delta_seconds as f32;

    camera.position += forward * direction.z * settings.speed * delta_time;
    camera.position += right * direction.x * settings.speed * delta_time;
    camera.position += up * direction.y * settings.speed * delta_time;
}

fn input_to_direction(input: &InputMap) -> Vector3<f32> {
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

    // TODO: Only jump on floors
    if input.is_pressed(VirtualKeyCode::Space) {
        direction.y += 1.0;
    }
    if input.is_pressed(VirtualKeyCode::LShift) {
        direction.y += -1.0;
    }
    direction
}

pub fn update_player(
    camera: Res<Camera>,
    input: Res<InputMap>,
    settings: Res<PlayerSettings>,
    mut player: ResMut<Player>,
) {
    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), camera.yaw.0);

    let mut desired_movement = rot * input_to_direction(&input);

    desired_movement *= settings.speed;

    desired_movement += Vector3::new(0.0, -1.0, 0.0) * settings.gravity;

    player.desired_movement = desired_movement;
}

impl ApplicationBuilder {
    pub fn with_player_controller(self, settings: PlayerSettings) -> Self {
        self.with_resource(settings)
            .with_resource(Player {
                desired_movement: Vector3::new(0.0, 0.0, 0.0),
            })
            .with_startup_system(setup_player)
            .with_system(handle_mouse_movement.in_set(AppStage::Update))
            .with_system(update_player.in_set(AppStage::Update))
        // Freecam mode, make sure to disable the character controller
        //.with_system(update_camera_position.in_set(AppStage::Update))
    }
}

fn setup_player(mut commands: Commands) {
    commands.spawn(CharacterController { handle: None });
}
