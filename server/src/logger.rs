use tracing::{level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init(level: &LevelFilter) {
    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(level.clone())
        .init();
}
