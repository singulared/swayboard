use std::collections::HashMap;

use futures_util::stream::StreamExt;
use swayipc_async::{Connection, EventType, Fallible};
use swayipc_types::{Event, WindowChange};
use tracing::{debug, info, info_span, trace, Instrument};

use crate::Layout;

pub struct LayoutManager {
    last_container_id: Option<i64>,
    layout_map: HashMap<i64, Layout>,
    device: String,
}

impl LayoutManager {
    pub async fn new(device: String) -> Fallible<Self> {
        Ok(LayoutManager {
            device,
            last_container_id: None,
            layout_map: HashMap::new(),
        })
    }

    pub async fn keyboards(ipc: &mut Connection) -> Fallible<Vec<String>> {
        debug!("Get keyboards list");
        let devices = ipc
            .get_inputs()
            .await?
            .into_iter()
            .filter(|input| input.input_type == "keyboard")
            .map(|input| input.identifier)
            .collect::<Vec<_>>();
        Ok(devices)
    }

    pub async fn layouts(&self, ipc: &mut Connection) -> Fallible<Vec<Layout>> {
        debug!("Get layouts list");
        let layouts = ipc
            .get_inputs()
            .await?
            .iter()
            .find(|input| input.input_type == "keyboard" && input.identifier == self.device)
            .map(|input| {
                input
                    .xkb_layout_names
                    .iter()
                    .enumerate()
                    .map(Layout::from)
                    .collect::<Vec<_>>()
            });
        Ok(layouts.unwrap_or_default())
    }

    pub async fn set_layout(&self, ipc: &mut Connection, layout: &Layout) -> Fallible<()> {
        info!("Change layout to {}", layout.name);
        let res = ipc
            .run_command(format!(
                "input \"{}\" xkb_switch_layout {}",
                self.device,
                layout.id
            ))
            .await?;
        // dbg!(res);
        Ok(())
    }

    pub async fn run() -> Fallible<()> {
        let subs = [EventType::Input, EventType::Window];
        let events = Connection::new().await?;
        let mut ipc = Connection::new().await?;
        let mut manager = LayoutManager::new("1:1:AT_Translated_Set_2_keyboard".to_owned()).await?;

        let mut events = events.subscribe(&subs).await?;
        while let Some(event) = events.next().await {
            // println!("{:#?}\n", &event);
            match event? {
                Event::Window(node) => {
                    match node.change {
                        WindowChange::Focus => {
                            let span = info_span!("Focus changed event");
                            span.in_scope(|| trace!(node.container.id));
                            manager.last_container_id = Some(node.container.id);
                            // Change layout by container id.
                            // dbg!(&manager.last_container_id);
                            // let res = self.ipc2.run_command("input \"1:1:AT_Translated_Set_2_keyboard\" xkb_switch_layout 1").await?;
                            let layout = {
                                manager.layout_map.get(&node.container.id)
                            };
                            match layout {
                                Some(layout) => manager.set_layout(&mut ipc, layout)
                                    .instrument(span)
                                    .await?,
                                None => (),
                            }
                            // Self::set_layout(&mut self.ipc2, self.layout_map.get(&node.container.id).unwrap()).await;
                        }
                        _ => (),
                    }
                }
                Event::Input(input) => {
                    // dbg!(&input);
                    let name = &input.input.xkb_active_layout_name.as_ref().unwrap();
                    let layout_id = input
                        .input
                        .xkb_layout_names
                        .iter()
                        .position(|layout| &layout == name)
                        .unwrap();
                    // dbg!(layout_id);
                    manager.layout_map.insert(
                        manager.last_container_id.unwrap_or_default(),
                        Layout::from((layout_id, *name)),
                    );
                    // Add match by change => XkbLayout
                    // Update layout for last_container_id
                }
                _ => (),
            };
        }
        Ok(())
    }
}
