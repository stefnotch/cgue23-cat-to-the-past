use bevy_ecs::query::Without;
use scene::loader::AssetServer;
use std::sync::Arc;
use std::time::Instant;

use bevy_ecs::system::{Commands, Query, Res};
use context::Context;
use nalgebra::{Translation3, UnitQuaternion};
use rapier3d::na::Vector3;

use crate::application::{AppConfig, ApplicationBuilder};
use crate::physics_context::BoxCollider;
use crate::player::PlayerSettings;
use crate::scene::light::{Light, PointLight};
use crate::scene::transform::{Transform, TransformBuilder};
use crate::time::Time;

mod application;
mod camera;
mod context;
mod input;
mod physics_context;
mod player;
mod render;
mod scene;
mod time;
mod time_manager;

fn spawn_world(mut commands: Commands, context: Res<Context>, asset_server: Res<AssetServer>) {
    let memory_allocator = Arc::new(
        vulkano::memory::allocator::StandardMemoryAllocator::new_default(context.device()),
    );

    commands.spawn((
        Light::Point(PointLight {
            color: Vector3::new(1.0, 1.0, 1.0),
            range: 0.0,
            intensity: 2.0,
        }),
        TransformBuilder::new()
            .translation(Translation3::new(0.0, 1.0, 0.0))
            .build(),
    ));

    let before = Instant::now();
    asset_server
        .load_default_scene(
            "./assets/scene/testing/only_floor_v3/untitled.gltf",
            &mut commands,
            &memory_allocator,
            &context,
        )
        .unwrap();
    println!(
        "Loading the scene took {}sec",
        before.elapsed().as_secs_f64()
    );

    // let cube = Mesh::cube(1.0, 1.0, 1.0, &memory_allocator);
    //
    // // let center = Vector3::new(4.07, 5.90, -1.01);
    // let center: Vector3<f32> = Vector3::zeros();
    //
    // for i in 0..32 {
    //     let angle = i as f32 * (2.0 * PI) / 32.0;
    //     let (sin, cos) = angle.sin_cos();
    //     commands.spawn((
    //         Model {
    //             primitives: vec![Primitive {
    //                 mesh: cube.clone(),
    //                 material: Arc::new(Material {
    //                     color: Vector3::new(1.0, 1.0, 1.0),
    //                     ka: 0.0,
    //                     kd: 1.0,
    //                     ks: 0.0,
    //                     alpha: 1.0,
    //                 }),
    //             }],
    //         },
    //         TransformBuilder::new()
    //             .translation(Translation3::new(
    //                 center.x + cos * 5.0,
    //                 center.y + 1.0,
    //                 center.z + sin * 5.0,
    //             ))
    //             .scale(Vector3::new(0.5, 0.5, 0.5))
    //             .build(),
    //     ));
    // }
    //
    // commands.spawn((
    //     Model {
    //         primitives: vec![Primitive {
    //             mesh: cube.clone(),
    //             material: Arc::new(Material {
    //                 color: Vector3::new(0.0, 1.0, 0.0),
    //                 ka: 0.0,
    //                 kd: 0.9,
    //                 ks: 0.3,
    //                 alpha: 10.0,
    //             }),
    //         }],
    //     },
    //     TransformBuilder::new()
    //         .translation(Translation3::new(0.0, 0.25, 0.0))
    //         .rotation(UnitQuaternion::from_axis_angle(
    //             &Vector3::y_axis(),
    //             PI / 2.0,
    //         ))
    //         .scale(Vector3::new(0.5, 0.5, 0.5))
    //         .build(),
    //     BoxCollider {
    //         size: Vector3::new(0.5, 0.5, 0.5),
    //     },
    //     RapierRigidBody { handle: None },
    // ));
    //
    // commands.spawn((
    //     Model {
    //         primitives: vec![Primitive {
    //             mesh: cube.clone(),
    //             material: Arc::new(Material {
    //                 color: Vector3::new(1.0, 0.0, 0.0),
    //                 ka: 0.0,
    //                 kd: 0.9,
    //                 ks: 0.0,
    //                 alpha: 10.0,
    //             }),
    //         }],
    //     },
    //     TransformBuilder::new()
    //         .translation(Translation3::new(0.0, -0.5, 0.0))
    //         .scale(Vector3::new(20.0, 0.1, 20.0))
    //         .build(),
    //     BoxCollider {
    //         size: Vector3::new(20.0, 0.1, 20.0),
    //     },
    // ));
}

fn _rotate_entites(mut query: Query<&mut Transform, Without<BoxCollider>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.05);
    }
}

fn _print_fps(time: Res<Time>) {
    println!("{}", 1.0 / time.delta_seconds())
}

#[cfg(feature = "trace")]
fn start_tracing() -> tracing_chrome::FlushGuard {
    pub use bevy_utils::tracing::{
        debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
        Level,
    };
    use std::panic;
    use tracing_log::LogTracer;
    use tracing_subscriber::fmt::{format::DefaultFields, FormattedFields};
    use tracing_subscriber::{prelude::*, registry::Registry, EnvFilter};

    let level = Level::INFO;
    let filter = "";

    let old_handler = panic::take_hook();
    panic::set_hook(Box::new(move |infos| {
        println!("{}", tracing_error::SpanTrace::capture());
        old_handler(infos);
    }));

    let finished_subscriber;
    let default_filter = { format!("{},{}", level, filter) };
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&default_filter))
        .unwrap();
    let subscriber = Registry::default().with(filter_layer);

    let subscriber = subscriber.with(tracing_error::ErrorLayer::default());

    let guard = {
        let (chrome_layer, guard) = {
            let mut layer = tracing_chrome::ChromeLayerBuilder::new();
            if let Ok(path) = std::env::var("TRACE_CHROME") {
                layer = layer.file(path);
            }
            let (chrome_layer, guard) = layer
                .name_fn(Box::new(|event_or_span| match event_or_span {
                    tracing_chrome::EventOrSpan::Event(event) => event.metadata().name().into(),
                    tracing_chrome::EventOrSpan::Span(span) => {
                        if let Some(fields) =
                            span.extensions().get::<FormattedFields<DefaultFields>>()
                        {
                            format!("{}: {}", span.metadata().name(), fields.fields.as_str())
                        } else {
                            span.metadata().name().into()
                        }
                    }
                }))
                .build();
            //app.world.insert_non_send_resource(guard);
            (chrome_layer, guard)
        };

        #[cfg(feature = "tracing-tracy")]
        let tracy_layer = tracing_tracy::TracyLayer::new();

        let fmt_layer = tracing_subscriber::fmt::Layer::default();

        // bevy_render::renderer logs a `tracy.frame_mark` event every frame
        // at Level::INFO. Formatted logs should omit it.
        #[cfg(feature = "tracing-tracy")]
        let fmt_layer = fmt_layer.with_filter(tracing_subscriber::filter::FilterFn::new(|meta| {
            meta.fields().field("tracy.frame_mark").is_none()
        }));

        let subscriber = subscriber.with(fmt_layer);

        let subscriber = subscriber.with(chrome_layer);
        #[cfg(feature = "tracing-tracy")]
        let subscriber = subscriber.with(tracy_layer);

        finished_subscriber = subscriber;
        guard
    };

    let logger_already_set = LogTracer::init().is_err();
    let subscriber_already_set =
        bevy_utils::tracing::subscriber::set_global_default(finished_subscriber).is_err();

    match (logger_already_set, subscriber_already_set) {
        (true, true) => warn!(
            "Could not set global logger and tracing subscriber as they are already set. Consider disabling LogPlugin."
        ),
        (true, _) => warn!("Could not set global logger as it is already set. Consider disabling LogPlugin."),
        (_, true) => warn!("Could not set global tracing subscriber as it is already set. Consider disabling LogPlugin."),
        _ => (),
    }
    guard
}

fn main() {
    #[cfg(feature = "trace")]
    let _guard = start_tracing();

    // TODO: remove this
    std::env::set_var("RUST_BACKTRACE", "1");

    // TODO: read from file
    let config = AppConfig {
        resolution: (1920, 1080),
        fullscreen: true,
        brightness: 1.0,
        refresh_rate: 144,
    };

    let player_settings = PlayerSettings::new(5.0, 1.0, 9.81);

    let application = ApplicationBuilder::new(config)
        .with_startup_system(spawn_world)
        // .with_system(print_fps)
        // .with_system(rotate_entites)
        .with_player_controller(player_settings)
        .build();

    application.run();
}
