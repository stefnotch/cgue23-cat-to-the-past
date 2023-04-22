#[cfg(feature = "tracing-chrome")]
pub struct FlushGuard {
    _guard: tracing_chrome::FlushGuard,
}

#[cfg(not(feature = "tracing-chrome"))]
pub struct FlushGuard {}

#[cfg(feature = "trace")]
pub fn start_tracing() -> FlushGuard {
    // source: https://github.com/bevyengine/bevy/blob/main/crates/bevy_log/src/lib.rs (LICENSE MIT)
    // https://github.com/bevyengine/bevy/issues/8123

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
        #[cfg(feature = "tracing-chrome")]
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

        #[cfg(feature = "tracing-chrome")]
        let subscriber = subscriber.with(chrome_layer);
        #[cfg(feature = "tracing-tracy")]
        let subscriber = subscriber.with(tracy_layer);

        finished_subscriber = subscriber;

        #[cfg(feature = "tracing-chrome")]
        let flush_guard = FlushGuard { _guard: guard };

        #[cfg(all(not(feature = "tracing-chrome"), feature = "tracing-tracy"))]
        let flush_guard = FlushGuard {};
        flush_guard
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

#[cfg(not(feature = "trace"))]
pub fn start_tracing() -> FlushGuard {
    // Dummy
    FlushGuard {}
}
