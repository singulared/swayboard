use tracing_appender::non_blocking::{NonBlockingBuilder, WorkerGuard};
use tracing_subscriber::{fmt::layer, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::Logging;

pub(crate) async fn init_logging(config: &Logging) -> WorkerGuard {
    let (non_blocking, guard) = NonBlockingBuilder::default().finish(std::io::stdout());
    let layer = layer();
    let subscriber_layer = layer.compact().with_writer(non_blocking);
    tracing_subscriber::registry()
        .with(EnvFilter::new(format!(
            "warn,swayboard={}",
            config.level.as_str()
        )))
        .with(subscriber_layer)
        .init();
    guard
}
