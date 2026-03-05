use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init(level: &str, disable_timestamp: bool) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let subscriber = tracing_subscriber::registry().with(filter);

    if disable_timestamp {
        let fmt_layer = fmt::layer()
            .without_time()
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false);
        subscriber.with(fmt_layer).init();
    } else {
        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false);
        subscriber.with(fmt_layer).init();
    }
}
