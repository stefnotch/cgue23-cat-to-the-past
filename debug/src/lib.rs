use log::enable_logging;
use tracing::start_tracing;

pub mod log;
pub mod tracing;

pub fn setup_debugging() -> tracing::FlushGuard {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_BACKTRACE", "1");

    let guard = start_tracing();

    enable_logging();
    guard
}
