#[cfg(feature = "tracy")]
pub fn init_profiling() {
    use tracing_subscriber::layer::SubscriberExt;
    let subscriber = tracing_subscriber::registry()
        .with(tracing_tracy::TracyLayer::default());
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
}

#[cfg(not(feature = "tracy"))]
pub fn init_profiling() {
    // No-op: tracing subscriber set up by the application
}

pub fn profile_span(name: &'static str) -> tracing::Span {
    tracing::span!(tracing::Level::TRACE, "profile", name = name)
}
