use angle::Rad;
use app::plugin::Plugin;
use bevy_ecs::prelude::*;
use bevy_ecs::system::Res;
use image::{DynamicImage, GenericImageView};
use nalgebra::{Point3, Vector2};
use scene::asset::AssetId;
use scene::texture::{
    AddressMode, BytesTextureData, CpuTexture, Filter, SamplerInfo, TextureFormat,
};
use scene::ui_component::{UIComponent, UITexturePosition};
use std::sync::Arc;
use time::time_manager::TimeManager;

use crate::rewind_power::RewindPower;

#[derive(Component)]
struct UICrosshair;

#[derive(Component)]
struct UIRewind;

fn spawn_ui_components(mut commands: Commands) {
    let sampler_info = SamplerInfo {
        min_filter: Filter::Nearest,
        mag_filter: Filter::Nearest,
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
    commands.spawn(UIComponent {
        texture: create_cpu_texture(game_over_texture),
        position: Point3::new(0.5, 0.5, 0.0),
        texture_position: UITexturePosition {
            scale: Vector2::new(10.0, 10.0),
            ..UITexturePosition::centered()
        },
        visible: false,
    });

    let rewind_texture = image::open("assets/textures/rewind_arrow.png").unwrap();
    commands.spawn((
        UIComponent {
            texture: create_cpu_texture(rewind_texture),
            position: Point3::new(0.5, 0.5, 0.0),
            texture_position: UITexturePosition {
                scale: Vector2::new(1.0, 1.0),
                ..UITexturePosition::centered()
            },
            visible: false,
        },
        UIRewind,
    ));

    let progress_fill = image::open("assets/textures/progress_fill.png").unwrap();
    commands.spawn(UIComponent {
        texture: create_cpu_texture(progress_fill),
        position: Point3::new(0.95, 0.15, -0.1),
        texture_position: UITexturePosition {
            scale: Vector2::new(2.0, 2.0),
            ..UITexturePosition::default()
        },
        visible: true,
    });

    let progress = image::open("assets/textures/progress_outline_stepped.png").unwrap();
    commands.spawn(UIComponent {
        texture: create_cpu_texture(progress),
        position: Point3::new(0.95, 0.15, 0.0),
        texture_position: UITexturePosition {
            scale: Vector2::new(2.0, 2.0),
            ..UITexturePosition::default()
        },
        visible: true,
    });
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

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app.with_startup_system(spawn_ui_components)
            .with_system(update_rewind);
    }
}
