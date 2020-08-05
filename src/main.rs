use std::collections::HashMap;
use swayipc::async_std;
use swayipc::{Connection, EventType, Fallible};
use async_std::stream::StreamExt;

#[derive(Debug)]
pub struct Layout {
    pub id: u32,
    pub name: String,
}

impl From<(usize, &String)> for Layout {
    fn from(layout: (usize, &String)) -> Self {
        Self { name: layout.1.clone(), id: layout.0 as u32 }
    }
}

pub struct LayoutManager {
    ipc: Connection,
    ipc2: Connection,
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

    pub async fn set_layout(ipc: &mut Connection, layout: &Layout) -> Fallible<()> {
        let res = ipc.run_command(format!("input \"1:1:AT_Translated_Set_2_keyboard\" xkb_switch_layout {}", layout.id)).await?;
        dbg!(res);
        Ok(())
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
                            // let res = self.ipc2.run_command("input \"1:1:AT_Translated_Set_2_keyboard\" xkb_switch_layout 1").await?;
                            let layout = self.layout_map.get(&node.container.id);
                            match layout {
                                Some(layout) => Self::set_layout(&mut self.ipc2, layout).await?,
                                None => ()
                            }
                            // Self::set_layout(&mut self.ipc2, self.layout_map.get(&node.container.id).unwrap()).await;
                        },
                        _ => (),
                    }
                },
                Event::Input(input) => {
                    dbg!(&input);
                    let name = &input.input.xkb_active_layout_name.as_ref().unwrap();
                    let layout_id = input.input.xkb_layout_names.iter().position(|layout| &layout == name).unwrap();
                    dbg!(layout_id);
                    self.layout_map.insert(self.last_container_id.unwrap_or_default(), Layout::from((layout_id, *name)));
                    // Add match by change => XkbLayout
                    // Update layout for last_container_id
                }
                _ => ()
            };
        }
        Ok(())
    }
}

// On start app should fill layout_map with current layout for app applications.

#[async_std::main]
async fn main() -> Fallible<()> {
    let connection = Connection::new().await?;
    let connection2 = Connection::new().await?;

    let mut manager = LayoutManager { ipc: connection, ipc2: connection2, last_container_id: None, layout_map: HashMap::new() };
    dbg!(manager.layouts("1:1:AT_Translated_Set_2_keyboard".to_owned()).await?);

    manager.run().await?;
    Ok(())
}
