use app::plugin::{Plugin, PluginAppAccess};
use game_core::time::Time;

use angle::{Angle, Deg, Rad};
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use game_core::time_manager::is_rewinding;
use input::events::{KeyboardInput, MouseMovement};
use input::input_map::InputMap;
use nalgebra::{UnitQuaternion, Vector3};
use physics::player_physics::PlayerCharacterController;
use scene::camera::Camera;
use scene::transform::Transform;
use windowing::event::ElementState;
use windowing::event::VirtualKeyCode;

#[derive(Component)]
pub struct CameraMode {
    free_cam_activated: bool,
}

#[derive(Component, Clone)]
pub struct PlayerControllerSettings {
    eye_height: f32,
    free_cam_speed: f32,
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

#[derive(Component, Debug)]
pub struct Player {
    pub velocity: Vector3<f32>,

    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl PlayerControllerSettings {
    pub fn new(speed: f32, sensitivity: f32, gravity: f32) -> Self {
        PlayerControllerSettings {
            eye_height: 1.75,
            free_cam_speed: speed,
            sensitivity,
            gravity,

            friction: 8.0,
            ground_accelerate: 50.0,
            air_accelerate: 100.0,
            max_velocity_ground: 4.0,
            max_velocity_air: 2.0,
            jump_force: 6.0,
            camera_smoothing: 20.0,
        }
    }
}

impl Default for PlayerControllerSettings {
    fn default() -> Self {
        PlayerControllerSettings::new(5.0, 1.0, 9.81)
    }
}

pub fn handle_mouse_movement(
    mut reader: EventReader<MouseMovement>,
    mut camera: ResMut<Camera>,
    mut query: Query<(&mut Player, &PlayerControllerSettings)>,
    time: Res<Time>,
) {
    let (mut player, settings) = query.single_mut();

    let mut pitch: Deg<f32> = player.pitch.into();
    let mut yaw: Deg<f32> = player.yaw.into();

    for event in reader.iter() {
        let MouseMovement(dx, dy) = *event;

        // Note: positive rotations are counter-clockwise. Adding to yaw rotates the camera to the
        // left. Moving the mouse to the left gives us negative dx values, so we flipped those.
        // Same logic applies to the y coordinate
        yaw += Deg(-dx as f32 * settings.sensitivity).into();
        pitch += Deg(-dy as f32 * settings.sensitivity).into();
    }

    let max_pitch: Deg<f32> = Deg(88.0);

    if pitch < -max_pitch {
        pitch = -max_pitch;
    } else if pitch > max_pitch {
        pitch = max_pitch;
    }
    let camera_factor = settings.camera_smoothing * time.delta_seconds();

    let target_orientation = UnitQuaternion::from_axis_angle(&Camera::up(), yaw.to_rad().0)
        * UnitQuaternion::from_axis_angle(&Camera::right(), pitch.to_rad().0);

    camera.orientation = camera.orientation.slerp(&target_orientation, camera_factor);

    player.pitch = pitch.into();
    player.yaw = yaw.into();
}

pub fn update_camera_position(
    mut camera: ResMut<Camera>,
    query: Query<(&Player, &PlayerControllerSettings)>,
    input: Res<InputMap>,
    time: Res<Time>,
) {
    let (player, settings) = query.single();

    let direction = input_to_direction(&input);

    let horizontal_movement = normalize_if_not_zero(get_horizontal(&direction));
    let vertical_movement = Camera::up().into_inner() * direction.y;

    let camera_horizontal_orientation =
        UnitQuaternion::from_axis_angle(&Camera::up(), player.yaw.0);

    let horizontal_direction = camera_horizontal_orientation * horizontal_movement;

    let delta_time = time.delta_seconds();

    camera.position += horizontal_direction * settings.free_cam_speed * delta_time;
    camera.position += vertical_movement * settings.free_cam_speed * delta_time;
}

fn input_to_direction(input: &InputMap) -> Vector3<f32> {
    let mut direction: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
    if input.is_pressed(VirtualKeyCode::W) {
        direction += Camera::forward().into_inner();
    }
    if input.is_pressed(VirtualKeyCode::S) {
        direction -= Camera::forward().into_inner();
    }

    if input.is_pressed(VirtualKeyCode::D) {
        direction += Camera::right().into_inner();
    }
    if input.is_pressed(VirtualKeyCode::A) {
        direction -= Camera::right().into_inner();
    }

    if input.is_pressed(VirtualKeyCode::Space) {
        direction += Camera::up().into_inner();
    }
    if input.is_pressed(VirtualKeyCode::LShift) {
        direction -= Camera::up().into_inner();
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

fn update_player(
    mut query: Query<(
        &mut Player,
        &mut PlayerCharacterController,
        &PlayerControllerSettings,
    )>,
    input: Res<InputMap>,
    time: Res<Time>,
) {
    let (mut player, mut character_controller, settings) = query.single_mut();

    let input_direction = input_to_direction(&input);
    let last_velocity = player.velocity;
    let horizontal_input: Vector3<f32> = normalize_if_not_zero(get_horizontal(&input_direction));
    let vertical_input = input_direction.y;
    let camera_horizontal_orientation =
        UnitQuaternion::from_axis_angle(&Camera::up(), player.yaw.0 + 0.1);

    let mut velocity = camera_horizontal_orientation * horizontal_input;

    if character_controller.grounded {
        velocity = move_ground(&velocity, get_horizontal(&last_velocity), &settings, &time);
        velocity.y = 0.0;
    } else {
        velocity = move_air(&velocity, get_horizontal(&last_velocity), &settings, &time);
        velocity.y = last_velocity.y;
    }

    if get_horizontal(&velocity).norm() < 0.05 {
        velocity.x = 0.0;
        velocity.z = 0.0;
    }

    if character_controller.grounded && vertical_input > 0.0 {
        velocity.y = settings.jump_force;
    }

    velocity.y -= settings.gravity * time.delta_seconds();

    // player hitting their head on the roof logic could go here

    player.velocity = velocity;
    character_controller.desired_movement = velocity;
}

// Dirty workaround for https://github.com/dimforge/rapier/issues/485
fn update_player2(mut query: Query<&mut PlayerCharacterController>) {
    let mut character_controller = query.single_mut();
    character_controller.desired_movement = [0.0, -0.1, 0.0].into();
}

fn update_player_camera(
    query: Query<(&Transform, &PlayerControllerSettings), With<Player>>,
    mut camera: ResMut<Camera>,
) {
    let (player_transform, player_settings) = query.single();
    camera.position =
        player_transform.position + Camera::up().into_inner() * player_settings.eye_height;
}

fn move_air(
    velocity: &Vector3<f32>,
    last_horizontal_velocity: Vector3<f32>,
    settings: &PlayerControllerSettings,
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
    settings: &PlayerControllerSettings,
    time: &Time,
) -> Vector3<f32> {
    let speed = last_horizontal_velocity.norm();
    if speed.abs() > 0.01 {
        let drop = speed * settings.friction * time.delta_seconds();
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
    let mut acceleration = acceleration * time.delta_seconds();

    if projected_velocity + acceleration > max_velocity {
        acceleration = max_velocity - projected_velocity;
    }

    last_velocity + acceleration_direction * acceleration
}

pub fn has_free_camera_activated(query: Query<&CameraMode, With<Player>>) -> bool {
    let camera_mode = query.single();
    camera_mode.free_cam_activated
}

pub fn free_cam_toggle_system(
    mut query: Query<&mut CameraMode, With<Player>>,
    mut reader: EventReader<KeyboardInput>,
) {
    for event in reader.iter() {
        if event.key_code == VirtualKeyCode::T && event.state == ElementState::Released {
            let mut camera_mode = query.single_mut();
            camera_mode.free_cam_activated = !camera_mode.free_cam_activated;
        }
    }
}

pub struct PlayerPlugin {
    player_spawn_settings: Option<PlayerSpawnSettings>,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerPluginSets {
    /// This set is used to update the player position and velocity. Should run before physics.
    Update,
    /// This set is used to update the camera position after the player has moved. Should run after physics.
    UpdateCamera,
}

impl PlayerPlugin {
    pub fn new(player_spawn_settings: PlayerSpawnSettings) -> Self {
        Self {
            player_spawn_settings: Some(player_spawn_settings),
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_resource(self.player_spawn_settings.take().unwrap())
            .with_startup_system(setup_player)
            .with_system(handle_mouse_movement.in_set(PlayerPluginSets::Update))
            .with_system(free_cam_toggle_system.in_set(PlayerPluginSets::Update))
            .with_system(
                update_player
                    .in_set(PlayerPluginSets::Update)
                    .after(free_cam_toggle_system)
                    .run_if(not(has_free_camera_activated))
                    .run_if(not(is_rewinding)),
            )
            .with_system(
                update_player2
                    .in_set(PlayerPluginSets::Update)
                    .after(free_cam_toggle_system)
                    .run_if(not(has_free_camera_activated))
                    .run_if(is_rewinding),
            )
            .with_system(
                update_camera_position
                    .in_set(PlayerPluginSets::Update)
                    .after(free_cam_toggle_system)
                    .run_if(has_free_camera_activated),
            )
            .with_system(
                update_player_camera
                    .in_set(PlayerPluginSets::UpdateCamera)
                    .run_if(not(has_free_camera_activated)),
            )
            .with_set((PlayerPluginSets::Update).before(PlayerPluginSets::UpdateCamera));
    }
}

#[derive(Resource)]
pub struct PlayerSpawnSettings {
    pub initial_transform: Transform,
    pub controller_settings: PlayerControllerSettings,
    pub free_cam_activated: bool,
}

fn setup_player(mut commands: Commands, spawn_settings: Res<PlayerSpawnSettings>) {
    // spawn player, character-controller
    commands.spawn((
        spawn_settings.controller_settings.clone(),
        Player {
            velocity: Default::default(),
            yaw: Default::default(),
            pitch: Default::default(),
        },
        spawn_settings.initial_transform.clone(),
        PlayerCharacterController::default(),
        CameraMode {
            free_cam_activated: spawn_settings.free_cam_activated,
        },
    ));
}
