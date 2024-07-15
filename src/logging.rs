use crate::config::Logging;

pub(crate) async fn init_logging(config: &Logging) {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(config.level.to_tracing_level())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");
}
