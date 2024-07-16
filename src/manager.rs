use std::collections::HashMap;

use futures_util::stream::StreamExt;
use swayipc_async::{Connection, EventType, InputEvent, Node};
use swayipc_types::{Event, WindowChange};
use tracing::{debug, trace, warn};

use crate::{layout::Layout, Error, LayoutError};

pub struct LayoutManager {
    current_container_id: Option<i64>,
    current_layout: Option<Layout>,
    containers_layout: HashMap<i64, Layout>,
    device: String,
    ipc: Connection,
    _policy: LayoutPolicy,
}

impl LayoutManager {
    pub(crate) async fn new(device: String, policy: Option<LayoutPolicy>) -> Result<Self, Error> {
        Ok(LayoutManager {
            device,
            current_container_id: None,
            current_layout: None,
            containers_layout: HashMap::new(),
            ipc: Connection::new().await?,
            _policy: policy.unwrap_or_default(),
        })
    }

    pub(crate) async fn keyboards(&mut self) -> Result<Vec<String>, Error> {
        debug!("retrieve keyboards list");
        let devices = self
            .ipc
            .get_inputs()
            .await?
            .into_iter()
            .filter(|input| input.input_type == "keyboard")
            .map(|input| input.identifier)
            .collect();
        Ok(devices)
    }

    pub(crate) async fn layouts(&mut self) -> Result<Vec<Layout>, Error> {
        debug!("get layouts list");
        let layouts = self
            .ipc
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
                    .collect()
            });
        Ok(layouts.unwrap_or_default())
    }

    pub(crate) async fn set_layout(&mut self, layout: &Layout) -> Result<(), Error> {
        debug!("change layout to {}", layout.name);
        if self.current_layout.as_ref() != Some(layout) {
            self.ipc
                .run_command(format!(
                    "input \"{}\" xkb_switch_layout {}",
                    self.device, layout.id
                ))
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn get_layout(&mut self) -> Result<Layout, Error> {
        debug!("get current layout");
        match &self.current_layout {
            None => {
                debug!("get current layout by IPC");
                self.ipc
                    .get_inputs()
                    .await?
                    .into_iter()
                    .find(|input| input.input_type == "keyboard" && input.identifier == self.device)
                    .map(Layout::try_from)
                    .ok_or(Error::DeviceNotFound)?
                    .map_err(Error::from)
            }
            Some(layout) => Ok(layout.clone()),
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), Error> {
        let subscribe = [EventType::Input, EventType::Window];
        self.current_layout = self.get_layout().await.ok();
        let connection = Connection::new().await?;
        let mut events = connection.subscribe(subscribe).await?;
        while let Some(event) = events.next().await {
            match self.handle_event(&event?).await {
                Ok(_) => (),
                Err(error) => warn!("handle event error: {error}"),
            };
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: &Event) -> Result<(), Error> {
        match event {
            Event::Window(window_event) => {
                if window_event.change == WindowChange::Focus {
                    self.handle_focus_change(&window_event.container).await?
                }
                if window_event.change == WindowChange::Close {
                    self.handle_window_close(&window_event.container).await?
                }
            }
            Event::Input(input) => self.handle_input_event(input).await?,
            _ => (),
        };
        Ok(())
    }

    async fn handle_input_event(&mut self, event: &InputEvent) -> Result<(), Error> {
        debug!("handle input event");
        trace!("{event:#?}");
        let name = event.input.xkb_active_layout_name.as_ref().ok_or_else(|| {
            LayoutError::LayoutDetection("Unable to detect current active layout".to_owned())
        })?;
        let layout_id = event
            .input
            .xkb_layout_names
            .iter()
            .position(|layout| layout == name)
            .ok_or_else(|| {
                LayoutError::LayoutDetection("unable to find layout {name}".to_owned())
            })?;
        let layout = Layout::from((layout_id, name));
        if self.current_layout.as_ref() != Some(&layout) {
            self.current_layout = Some(layout.clone())
        }
        if let Some(current_container) = self.current_container_id {
            debug!(
                "update current container layout {} {}",
                current_container, layout.name
            );
            self.containers_layout.insert(current_container, layout);
        }
        Ok(())
    }

    async fn handle_window_close(&mut self, node: &Node) -> Result<(), Error> {
        debug!("handle window close event");
        self.containers_layout.remove(&node.id);
        Ok(())
    }

    async fn handle_focus_change(&mut self, node: &Node) -> Result<(), Error> {
        debug!("handle focus change");
        self.current_container_id = Some(node.id);
        let preferred_layout = self.containers_layout.get(&node.id);
        match preferred_layout {
            Some(layout) => {
                if Some(layout) != self.current_layout.as_ref() {
                    self.set_layout(&layout.clone()).await?
                }
            }
            None => {
                let layout = self.get_layout().await?;
                debug!("save container layout {} {}", node.id, layout.name);
                self.containers_layout.insert(node.id, layout);
            }
        };
        Ok(())
    }
}

#[derive(Default)]
pub(crate) enum LayoutPolicy {
    #[allow(dead_code)]
    Default(String),
    #[default]
    Previous,
}

// #[derive(Default)]
// pub(crate) struct LayoutManagerBuilder {
//     devices: Option<Vec<String>>,
//     _window_layout_policy: LayoutPolicy,
// }
//
// impl LayoutManagerBuilder {
//     pub(crate) fn new() -> Self {
//         Self::default()
//     }
//
//     pub(crate) fn devices(self, devices: &[String]) -> Self {
//         Self {
//             devices: Some(devices.to_vec()),
//             ..self
//         }
//     }
//
//     pub(crate) fn device(self, device: String) -> Self {
//         let mut devices = self.devices.clone();
//         devices.as_mut().map(|devices| {
//             devices.push(device);
//             devices
//         });
//         Self { devices, ..self }
//     }
//
//     pub(crate) fn layout_policy(self, policy: LayoutPolicy) -> Self {
//         Self {
//             _window_layout_policy: policy,
//             ..self
//         }
//     }
//
//     pub(crate) async fn build(self) -> Result<LayoutManager, Error> {
//         LayoutManager::new(self.devices.unwrap().first().unwrap().to_owned()).await
//     }
// }
