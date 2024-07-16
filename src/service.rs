use anyhow::{Context, Result};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;

use crate::{
    cli::{Commands, LayoutSubcommands},
    config::Config,
    logging::init_logging,
    manager::LayoutManager,
};

pub struct Service {
    _config: Config,
    manager: LayoutManager,
    _logging_guard: WorkerGuard,
}

impl Service {
    pub async fn init() -> Result<Self> {
        let config = Config::new()?;
        let logging_guard = init_logging(&config.logging).await;
        let dev = config
            .device
            .as_ref()
            .map(|dev| dev.identifier.clone())
            .unwrap_or_else(|| "*".to_owned());
        info!("configured device: {dev}");
        let manager = LayoutManager::new(dev, None).await?;
        let service = Service {
            _config: config,
            manager,
            _logging_guard: logging_guard,
        };
        info!("service initialized");
        Ok(service)
    }

    pub async fn execute(mut self, command: &Commands) -> Result<()> {
        info!("execute CMD {command:?}");

        match command {
            Commands::Layout(layout) => match &layout.command {
                LayoutSubcommands::List => {
                    self.manager
                        .layouts()
                        .await?
                        .iter()
                        .for_each(|layout| println!("{}", layout.name));
                }
                LayoutSubcommands::Get => println!("{}", self.manager.get_layout().await?.name),
                LayoutSubcommands::Set(args) => {
                    let layouts = self.manager.layouts().await?;
                    let layout = layouts
                        .iter()
                        .find(|layout| layout.name == args.layout)
                        .ok_or_else(|| anyhow::anyhow!("Wrong layout name"))
                        .context("Layout setting failed")?;
                    self.manager.set_layout(layout).await?;
                }
            },
            Commands::Devices => self
                .manager
                .keyboards()
                .await?
                .iter()
                .for_each(|device| println!("{}", device)),
            Commands::Run => {
                self.manager.run().await?;
            }
        };
        Ok(())
    }
}
