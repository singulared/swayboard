use std::collections::HashMap;

use futures_util::stream::StreamExt;
use swayipc_async::{Connection, EventType, Fallible, InputEvent, Node};
use swayipc_types::{Event, WindowChange};
use tracing::{debug, info, info_span, trace, Instrument};

use crate::{Error, Layout};

pub struct LayoutManager {
    last_container_id: Option<i64>,
    containers_layout: HashMap<i64, Layout>,
    device: String,
    ipc: Connection,
}

impl LayoutManager {
    pub async fn new(device: String) -> Result<Self, Error> {
        Ok(LayoutManager {
            device,
            last_container_id: None,
            containers_layout: HashMap::new(),
            ipc: Connection::new().await?,
        })
    }

    pub async fn keyboards(&mut self) -> Result<Vec<String>, Error> {
        debug!("Retrieve keyboards list");
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

    pub async fn layouts(&mut self) -> Result<Vec<Layout>, Error> {
        debug!("Get layouts list");
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

    pub async fn set_layout(&mut self, layout: &Layout) -> Result<(), Error> {
        debug!("Change layout to {}", layout.name);
        self.ipc
            .run_command(format!(
                "input \"{}\" xkb_switch_layout {}",
                self.device, layout.id
            ))
            .await?;
        Ok(())
    }

    pub async fn get_layout(&mut self) -> Result<Layout, Error> {
        debug!("Get current layout");
        self.ipc
            .get_inputs()
            .await?
            .into_iter()
            .find(|input| input.input_type == "keyboard" && input.identifier == self.device)
            .map(|input| Layout::try_from(input))
            .ok_or_else(|| Error::DeviceNotFound)?
            .map_err(Error::from)
    }

    pub async fn run(mut self) -> Result<(), Error> {
        let subscribe = [EventType::Input, EventType::Window];
        let connection = Connection::new().await?;
        let mut events = connection.subscribe(subscribe).await?;
        while let Some(event) = events.next().await {
            self.handle_event(&event?).await?;
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: &Event) -> Result<(), Error> {
        Ok(match event {
            Event::Window(window_event) => match window_event.change {
                WindowChange::Focus => self.handle_focus_change(&window_event.container).await?,
                _ => (),
            },
            Event::Input(input) => self.handle_input_event(&input).await?,
            _ => (),
        })
    }

    async fn handle_input_event(&mut self, event: &InputEvent) -> Result<(), Error> {
        let name = event.input.xkb_active_layout_name.as_ref().unwrap();
        let layout_id = event
            .input
            .xkb_layout_names
            .iter()
            .position(|layout| layout == name)
            .unwrap();
        self.containers_layout.insert(
            self.last_container_id.unwrap_or_default(),
            Layout::from((layout_id, name)),
        );
        Ok(())
    }

    async fn handle_focus_change(&mut self, node: &Node) -> Result<(), Error> {
        self.last_container_id = Some(node.id);
        let preferred_layout = self.containers_layout.get(&node.id);
        match preferred_layout {
            Some(layout) => self.set_layout(&layout.clone()).await?,
            None => (),
        };
        Ok(())
    }
}

#[derive(Default)]
pub enum NewWindowLayoutPolicy {
    DefaultLayout,
    #[default]
    PreviousLayout,
}

pub struct LayoutManagerBuilder {
    devices: Option<Vec<String>>,
    new_window_layout_policy: NewWindowLayoutPolicy,
}

impl LayoutManagerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn devices(self, devices: &[String]) -> Self {
        Self {
            devices: Some(devices.iter().cloned().collect::<Vec<_>>()),
            ..self
        }
    }

    pub fn device(self, device: String) -> Self {
        let mut devices = self.devices.clone();
        devices.as_mut().map(|devices| {
            devices.push(device);
            devices
        });
        Self { devices, ..self }
    }

    pub fn new_window_layout_policy(self, policy: NewWindowLayoutPolicy) -> Self {
        Self {
            new_window_layout_policy: policy,
            ..self
        }
    }

    pub async fn build(self) -> Result<LayoutManager, Error> {
        LayoutManager::new(self.devices.unwrap().first().unwrap().to_owned()).await
    }
}

impl Default for LayoutManagerBuilder {
    fn default() -> Self {
        LayoutManagerBuilder {
            devices: None,
            new_window_layout_policy: NewWindowLayoutPolicy::default(),
        }
    }
}
