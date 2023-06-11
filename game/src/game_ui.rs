use angle::Rad;
use app::plugin::Plugin;
use bevy_ecs::prelude::*;
use bevy_ecs::system::Res;
use image::{DynamicImage, GenericImageView};
use nalgebra::{Point2, Point3, Vector2};
use scene::asset::AssetId;
use scene::texture::{
    AddressMode, BytesTextureData, CpuTexture, Filter, MipmapMode, SamplerInfo, TextureFormat,
};
use scene::ui_component::{UIComponent, UITexturePosition};
use std::sync::Arc;
use time::time::Time;
use time::time_manager::TimeManager;

use crate::game_over::GameOver;
use crate::pickup_system::PickupInfo;
use crate::rewind_power::RewindPower;

#[derive(Component)]
struct UICrosshair;

#[derive(Component)]
struct UIRewind;

#[derive(Component)]
struct UIProgressFill;
#[derive(Component)]
struct UIProgressBar;
#[derive(Component)]
struct UIGameOver;

fn spawn_ui_components(mut commands: Commands) {
    let sampler_info = SamplerInfo {
        min_filter: Filter::Nearest,
        mag_filter: Filter::Nearest,
        mipmap_mode: MipmapMode::Nearest,
        address_mode: [AddressMode::ClampToBorder; 3],
    };

    let crosshair_texture = image::open("assets/textures/crosshair.png").unwrap();

    let create_cpu_texture = |texture: DynamicImage| {
        Arc::new(CpuTexture {
            id: AssetId::new_v4(),
            data: Box::new(BytesTextureData {
                dimensions: texture.dimensions(),
                format: TextureFormat::R8G8B8A8_UNORM,
                bytes: texture.as_bytes().to_vec(),
            }),
            sampler_info,
        })
    };

    commands.spawn((
        UIComponent {
            texture: create_cpu_texture(crosshair_texture),
            position: Point3::new(0.5, 0.5, -0.5),
            texture_position: UITexturePosition {
                scale: Vector2::new(1.0, 1.0),
                ..UITexturePosition::centered()
            },
            visible: true,
        },
        UICrosshair,
    ));

    let game_over_texture = image::open("assets/textures/game_over.png").unwrap();
    commands.spawn((
        UIComponent {
            texture: create_cpu_texture(game_over_texture),
            position: Point3::new(0.5, 0.5, 0.0),
            texture_position: UITexturePosition {
                scale: Vector2::new(10.0, 10.0),
                ..UITexturePosition::centered()
            },
            visible: false,
        },
        UIGameOver,
    ));

    let rewind_texture = image::open("assets/textures/rewind_arrow.png").unwrap();
    commands.spawn((
        UIComponent {
            texture: create_cpu_texture(rewind_texture),
            position: Point3::new(0.5, 0.5, -0.1),
            texture_position: UITexturePosition {
                scale: Vector2::new(2.0, 2.0),
                ..UITexturePosition::centered()
            },
            visible: false,
        },
        UIRewind,
    ));

    let progress_fill = image::open("assets/textures/progress_fill.png").unwrap();
    commands.spawn((
        UIComponent {
            texture: create_cpu_texture(progress_fill),
            position: Point3::new(0.95, 0.05, 0.0),
            texture_position: UITexturePosition {
                scale: Vector2::new(1.0, 1.0),
                texture_origin: Point2::new(0.5, 1.0),
                angle: Rad(std::f32::consts::FRAC_PI_2),
            },
            visible: true,
        },
        UIProgressFill,
    ));

    let progress = image::open("assets/textures/progress_outline_stepped.png").unwrap();
    commands.spawn((
        UIComponent {
            texture: create_cpu_texture(progress),
            position: Point3::new(0.95, 0.05, 0.0),
            texture_position: UITexturePosition {
                scale: Vector2::new(1.0, 1.0),
                texture_origin: Point2::new(0.5, 1.0),
                angle: Rad(std::f32::consts::FRAC_PI_2),
            },
            visible: true,
        },
        UIProgressBar,
    ));
}

fn update_rewind(
    time_manager: Res<TimeManager>,
    mut crosshair_query: Query<&mut UIComponent, With<UICrosshair>>,
    mut rewind_query: Query<&mut UIComponent, (With<UIRewind>, Without<UICrosshair>)>,
) {
    let mut rewind = rewind_query.single_mut();
    let mut crosshair = crosshair_query.single_mut();

    if time_manager.is_rewinding() {
        rewind.visible = true;
        crosshair.visible = false;

        rewind.texture_position.angle +=
            Rad(time_manager.level_delta_time().duration().as_secs_f32() * 4.0);
    } else {
        rewind.visible = false;
        crosshair.visible = true;
    }
}

fn update_rewind_power(
    time_manager: Res<TimeManager>,
    time: Res<Time>,
    rewind_power: Res<RewindPower>,
    mut progress_fill_query: Query<&mut UIComponent, With<UIProgressFill>>,
    mut progress_bar_query: Query<&mut UIComponent, (With<UIProgressBar>, Without<UIProgressFill>)>,
) {
    let mut progress_fill = progress_fill_query.single_mut();
    let mut progress_bar = progress_bar_query.single_mut();

    progress_fill.texture_position.scale.y = rewind_power.get_percent();
    let start_angle = std::f32::consts::FRAC_PI_2;
    progress_fill.texture_position.angle = Rad(start_angle);
    progress_bar.texture_position.angle = Rad(start_angle);

    if time_manager.is_rewinding() {
        if time_manager.level_delta_time().duration().is_zero() {
            // if we cannot rewind anymore
            let elapsed_time = time.time_since_startup().as_secs_f32();
            let angle = Rad(start_angle + (elapsed_time * 50.0).sin() * 0.02);
            progress_fill.texture_position.angle = angle;
            progress_bar.texture_position.angle = angle;
        }
    }
}

fn update_pickup_crosshair(
    pickup_info: Res<PickupInfo>,
    mut crosshair_query: Query<&mut UIComponent, With<UICrosshair>>,
) {
    let mut crosshair = crosshair_query.single_mut();

    if pickup_info.can_pickup {
        crosshair.texture_position.scale = Vector2::new(1.5, 1.5);
    } else {
        crosshair.texture_position.scale = Vector2::new(1.0, 1.0);
    }
}

fn update_game_over(
    game_over: Res<GameOver>,
    mut game_over_query: Query<&mut UIComponent, With<UIGameOver>>,
) {
    let mut game_over_component = game_over_query.single_mut();
    game_over_component.visible = game_over.is_game_over();
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app.with_startup_system(spawn_ui_components)
            .with_system(update_rewind)
            .with_system(update_rewind_power.after(update_rewind))
            .with_system(update_pickup_crosshair.after(update_rewind_power))
            .with_system(update_game_over.after(update_pickup_crosshair));
    }
}
