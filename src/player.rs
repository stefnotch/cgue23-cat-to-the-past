use crate::application::{AppStage, ApplicationBuilder};
use crate::camera::Camera;
use crate::input::{InputMap, MouseMovement};
use crate::physics_context::CharacterController;
use crate::scene::transform::{Transform, TransformBuilder};
use crate::time::Time;
use angle::{Angle, Deg, Rad};
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use bevy_ecs::system::{Res, ResMut};
use nalgebra::{Point3, Quaternion, UnitQuaternion, Vector, Vector3};
use std::f32::consts::FRAC_PI_2;
use winit::dpi::Position;
use winit::event::VirtualKeyCode;

#[derive(Resource)]
pub struct PlayerSettings {
    freecam_speed: f32,
    /// players use a different gravity
    gravity: f32,
    sensitivity: f32,

    friction: f32,
    ground_accelerate: f32,
    air_accelerate: f32,
    max_velocity_ground: f32,
    max_velocity_air: f32,
    camera_smoothing: f32,
    jump_force: f32,
}

#[derive(Resource, Debug)]
pub struct Player {
    pub desired_movement: Vector3<f32>,

    pub velocity: Vector3<f32>,

    pub jump_available: bool,

    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl PlayerSettings {
    pub fn new(speed: f32, sensitivity: f32, gravity: f32) -> Self {
        PlayerSettings {
            freecam_speed: speed,
            sensitivity,
            gravity,

            friction: 8.0,
            ground_accelerate: 50.0,
            air_accelerate: 100.0,
            max_velocity_ground: 4.0,
            max_velocity_air: 2.0,
            jump_force: 8.0,
            camera_smoothing: 20.0,
        }
    }
}

pub fn handle_mouse_movement(
    mut reader: EventReader<MouseMovement>,
    mut camera: ResMut<Camera>,
    mut player: ResMut<Player>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
) {
    let mut pitch: Deg<f32> = player.pitch.into();
    let mut yaw: Deg<f32> = player.yaw.into();

    for event in reader.iter() {
        let MouseMovement(dx, dy) = *event;

        yaw += Deg(dx as f32 * settings.sensitivity).into();
        pitch += Deg(dy as f32 * settings.sensitivity).into();
    }

    let max_pitch: Deg<f32> = Deg(90.0 - 0.01);

    if pitch < -max_pitch {
        pitch = -max_pitch;
    } else if pitch > max_pitch {
        pitch = max_pitch;
    }
    let camera_factor = settings.camera_smoothing * time.delta_seconds as f32;

    let target_orientation = UnitQuaternion::from_axis_angle(&Vector::y_axis(), yaw.to_rad().0)
        * UnitQuaternion::from_axis_angle(&Vector::x_axis(), pitch.to_rad().0);

    camera.orientation = camera.orientation.slerp(&target_orientation, camera_factor);
    player.pitch = pitch.into();
    player.yaw = yaw.into();
}

pub fn update_camera_position(
    mut camera: ResMut<Camera>,
    input: Res<InputMap>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
) {
    let direction = input_to_direction(&input);

    let forward = camera.orientation * Vector::z_axis();
    let right = camera.orientation * Vector::x_axis();
    let up = Vector3::new(0.0, 1.0, 0.0);

    let delta_time = time.delta_seconds as f32;

    camera.position += forward.into_inner() * direction.z * settings.freecam_speed * delta_time;
    camera.position += right.into_inner() * direction.x * settings.freecam_speed * delta_time;
    camera.position += up * direction.y * settings.freecam_speed * delta_time;
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

    if input.is_pressed(VirtualKeyCode::Space) {
        direction.y += 1.0;
    }
    if input.is_pressed(VirtualKeyCode::LShift) {
        direction.y += -1.0;
    }
    direction
}

fn get_horizontal(input_direction: &Vector3<f32>) -> Vector3<f32> {
    Vector3::new(input_direction.x, 0.0, input_direction.z)
}

fn normalize_if_not_zero(vector: Vector3<f32>) -> Vector3<f32> {
    let length_squared = vector.norm_squared();
    if length_squared.abs() < 0.001 {
        Vector3::zeros()
    } else {
        vector.normalize()
    }
}

pub fn update_player(
    input: Res<InputMap>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
    mut player: ResMut<Player>,
) {
    let input_direction = input_to_direction(&input);
    let last_velocity = player.velocity;
    let horizontal_input: Vector3<f32> = normalize_if_not_zero(get_horizontal(&input_direction));
    let vertical_input = input_direction.y;
    //let (_, _, yaw) = camera.orientation.euler_angles();
    let yaw = player.yaw;

    let mut velocity = UnitQuaternion::from_axis_angle(&Vector::y_axis(), yaw.0) * horizontal_input;

    if player.jump_available {
        velocity = move_ground(&velocity, get_horizontal(&last_velocity), &settings, &time);
        velocity.y = -settings.gravity.abs() * 0.5;
    } else {
        velocity = move_air(&velocity, get_horizontal(&last_velocity), &settings, &time);
        velocity.y = last_velocity.y;
    }

    if velocity.norm() < 0.05 {
        velocity = Vector3::zeros()
    }

    if player.jump_available && vertical_input > 0.0 {
        velocity.y = settings.jump_force;
        player.jump_available = false;
    }

    velocity.y -= settings.gravity * time.delta_seconds as f32;

    // player hitting their head on the roof logic could go here

    player.velocity = velocity;
    player.desired_movement = velocity;
}

fn move_air(
    velocity: &Vector3<f32>,
    last_horizontal_velocity: Vector3<f32>,
    settings: &PlayerSettings,
    time: &Time,
) -> Vector3<f32> {
    accelerate(
        velocity,
        last_horizontal_velocity,
        settings.max_velocity_air,
        settings.air_accelerate,
        time,
    )
}

fn move_ground(
    velocity: &Vector3<f32>,
    mut last_horizontal_velocity: Vector3<f32>,
    settings: &PlayerSettings,
    time: &Time,
) -> Vector3<f32> {
    let speed = last_horizontal_velocity.norm();
    if speed.abs() > 0.01 {
        let drop = speed * settings.friction * time.delta_seconds as f32;
        last_horizontal_velocity *= (speed - drop).max(0.0) / speed;
    }

    accelerate(
        velocity,
        last_horizontal_velocity,
        settings.max_velocity_ground,
        settings.ground_accelerate,
        time,
    )
}

fn accelerate(
    acceleration_direction: &Vector3<f32>,
    last_velocity: Vector3<f32>,
    max_velocity: f32,
    acceleration: f32,
    time: &Time,
) -> Vector3<f32> {
    // see https://github.com/FlaxEngine/FlaxSamples/blob/efebd54fa3cf3171c90d43061b138f399407318d/FirstPersonShooterTemplate/Source/FirstPersonShooter/PlayerScript.cs#L164
    let projected_velocity = last_velocity.dot(acceleration_direction);
    let mut acceleration = acceleration * time.delta_seconds as f32;

    if projected_velocity + acceleration > max_velocity {
        acceleration = max_velocity - projected_velocity;
    }

    last_velocity + acceleration_direction * acceleration
}

impl ApplicationBuilder {
    pub fn with_player_controller(self, settings: PlayerSettings) -> Self {
        self.with_resource(settings)
            .with_resource(Player {
                desired_movement: Vector3::new(0.0, 0.0, 0.0),
                velocity: Vector3::new(0.0, 0.0, 0.0),
                jump_available: false,
                yaw: Default::default(),
                pitch: Default::default(),
            })
            .with_startup_system(setup_player)
            .with_system(handle_mouse_movement.in_set(AppStage::Update))
            .with_system(update_player.in_set(AppStage::Update))
        // Freecam mode, make sure to disable the character controller
        // .with_system(update_camera_position.in_set(AppStage::Update))
    }
}

fn setup_player(mut commands: Commands) {
    commands.spawn(CharacterController { handle: None });
}
