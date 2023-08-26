use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing() {
    #[cfg(debug_assertions)]
    let fmt_layer = fmt::layer().with_timer(fmt::time::uptime()).pretty();
    #[cfg(not(debug_assertions))]
    let fmt_layer = fmt::layer();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("info"))
                .unwrap(),
        )
        .with(fmt_layer)
        .init();
}
