use std::collections::HashMap;
use swayipc::async_std;
use swayipc::{Connection, EventType, Fallible};
use async_std::stream::StreamExt;

#[derive(Debug)]
pub struct Layout {
    id: u32,
    name: String,
}

impl From<(usize, &String)> for Layout {
    fn from(layout: (usize, &String)) -> Self {
        Self { name: layout.1.clone(), id: layout.0 as u32 }
    }
}

pub struct LayoutManager {
    ipc: Connection,
    last_container_id: Option<i64>,
    layout_map: HashMap<i64, Layout>,
}

impl LayoutManager {
    pub async fn layouts(&mut self, device: String) -> Fallible<Vec<Layout>> {
        let layouts = self.ipc
            .get_inputs()
            .await?
            .iter()
            .find(|input| input.input_type == "keyboard" && input.identifier == device)
            .map(|input| input
                .xkb_layout_names
                .iter()
                .enumerate()
                .map(Layout::from)
                .collect::<Vec<_>>()
            );
        Ok(layouts.unwrap())
    }

    pub async fn run(mut self) -> Fallible<()> {
        let subs = [
            EventType::Input,
            EventType::Window,
        ];
        use swayipc::reply::{Event, WindowChange};

        let mut events = self.ipc.subscribe(&subs).await?;
        while let Some(event) = events.next().await {
            // println!("{:#?}\n", &event);
            match event? {
                Event::Window(node) => {
                    match node.change {
                        WindowChange::Focus => {
                            self.last_container_id = Some(node.container.id);
                            // Change layout by container id.
                            dbg!(&self.last_container_id);
                        },
                        _ => (),
                    }
                },
                Event::Input(input) => {
                    dbg!(input);
                    // Add match by change => XkbLayout
                    // Update layout for last_container_id
                }
                _ => ()
            };
        }
        Ok(())
    }
}

#[async_std::main]
async fn main() -> Fallible<()> {
    let connection = Connection::new().await?;

    let mut manager = LayoutManager { ipc: connection, last_container_id: None, layout_map: HashMap::new() };
    dbg!(manager.layouts("1:1:AT_Translated_Set_2_keyboard".to_owned()).await?);

    manager.run().await?;

    // let inputs = manager.ipc.get_inputs().await?;
    // let layouts = inputs
        // .iter()
        // .filter(|input| input.input_type == "keyboard" && input.vendor == 1)
        // .map(|input| {
            // dbg!(&input);
            // input
                // .xkb_layout_names
                // .iter()
                // .map(move |layout_name| {
                    // (
                        // Layout::from((42, layout_name)),
                        // input
                            // .xkb_active_layout_name
                            // .as_ref()
                            // .map(|active_layout| active_layout == layout_name)
                            // .unwrap_or_default(),
                    // )
                // })
                // .collect::<Vec<_>>()
        // })
        // .flatten()
        // .collect::<Vec<_>>();
    // // dbg!(&layouts);
    // let active_layout = layouts
        // .iter()
        // .enumerate()
        // .find(|(_, layout)| layout.1)
        // .map(|(id, _)| id)
        // .unwrap_or_default();
    // let layouts = layouts
        // .into_iter()
        // .map(|(layout, _)| layout)
        // .collect::<Vec<_>>();
    // println!("Layout: {:?}", &layouts[active_layout]);

    // let subs = [
        // EventType::Input,
        // EventType::Window,
    // ];

    // let mut events = manager.ipc.subscribe(&subs).await?;
    // while let event = events.next().await {
        // println!("{:#?}\n", event?)
    // }

    Ok(())
}
