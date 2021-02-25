use swayipc_async::{Fallible, Connection};
use swayboard::LayoutManager;
use tracing::{Level, info};

// On start app should fill layout_map with current layout for app applications.

#[tokio::main]
async fn main() -> Fallible<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting tracing default failed");
    info!("test");
    let manager = LayoutManager::new("1:1:AT_Translated_Set_2_keyboard".to_owned()).await?;
    let mut ipc = Connection::new().await?;
    dbg!(LayoutManager::keyboards(&mut ipc).await?);
    dbg!(
        manager
            .layouts(&mut ipc)
            .await?
    );

    LayoutManager::run().await?;
    Ok(())
}
