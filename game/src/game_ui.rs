use bevy_ecs::prelude::Commands;
use image::{DynamicImage, GenericImageView};
use nalgebra::{Point3, Vector2};
use scene::asset::AssetId;
use scene::texture::{
    AddressMode, BytesTextureData, CpuTexture, Filter, SamplerInfo, TextureFormat,
};
use scene::ui_component::{UIComponent, UITexturePosition};
use std::sync::Arc;

pub fn spawn_ui_component(mut commands: Commands) {
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

    commands.spawn(UIComponent {
        texture: create_cpu_texture(crosshair_texture),
        position: Point3::new(0.5, 0.5, -0.5),
        texture_position: UITexturePosition {
            scale: Vector2::new(1.0, 1.0),
            ..UITexturePosition::centered()
        },
        visible: true,
    });

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
    commands.spawn(UIComponent {
        texture: create_cpu_texture(rewind_texture),
        position: Point3::new(0.05, 0.05, 0.0),
        texture_position: UITexturePosition {
            scale: Vector2::new(5.0, 5.0),
            ..UITexturePosition::default()
        },
        visible: true,
    });

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
