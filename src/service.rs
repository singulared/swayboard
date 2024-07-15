use anyhow::{Context, Result};
use tracing::{info, instrument};

use crate::{
    cli::{Commands, LayoutSubcommands},
    config::Config,
    logging::init_logging,
    manager::LayoutManager,
};

pub struct Service {
    _config: Config,
    manager: LayoutManager,
}

impl Service {
    pub async fn init() -> Result<Self> {
        let config = Config::new()?;
        init_logging(&config.logging).await;
        let manager = LayoutManager::new("1:1:AT_Translated_Set_2_keyboard".to_owned()).await?;
        let service = Service {
            _config: config,
            manager,
        };
        info!("logging initialized");
        info!("service initialized");
        Ok(service)
    }

    #[instrument(name = "cli", skip(self))]
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
