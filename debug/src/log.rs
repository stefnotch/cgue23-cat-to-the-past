#[cfg(feature = "trace")]
pub fn enable_logging() {}

#[cfg(not(feature = "trace"))]
pub fn enable_logging() {
    tracing_subscriber::fmt().init();
}
