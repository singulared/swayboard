use clap::Parser;
use swayboard::{
    cli::{Commands, LayoutSubcommands},
    Cli, LayoutManager,
};
use tracing::Level;
use anyhow::{Result, Context};

// On start app should fill layout_map with current layout for app applications.

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");
    let cli = Cli::parse();

    let mut manager = LayoutManager::new("1:1:AT_Translated_Set_2_keyboard".to_owned()).await?;
    match &cli.command {
        Commands::Layout(layout) => match &layout.command {
            LayoutSubcommands::List => {
                manager
                    .layouts()
                    .await?
                    .iter()
                    .for_each(|layout| println!("{}", layout.name));
            }
            LayoutSubcommands::Get => println!("{}", manager.get_layout().await?.name),
            LayoutSubcommands::Set(args) => {
                let layouts = manager.layouts().await?;
                let layout = layouts
                    .iter()
                    .find(|layout| layout.name == args.layout)
                    .ok_or_else(|| anyhow::anyhow!("Wrong layout name")).context("Layout setting failed")?;
                manager.set_layout(layout).await?;
            },
        },
        Commands::Devices => manager
            .keyboards()
            .await?
            .iter()
            .for_each(|device| println!("{}", device)),
        Commands::Run => {
            manager.run().await?;
        }
    }
    Ok(())
}
