use swayipc::async_std;
use swayipc::{Connection, EventType, Fallible};

#[derive(Debug)]
struct Layout {
    name: String,
}

impl From<&String> for Layout {
    fn from(name: &String) -> Self {
        Self { name: name.clone() }
    }
}

#[async_std::main]
async fn main() -> Fallible<()> {
    let mut connection = Connection::new().await?;
    let inputs = connection.get_inputs().await?;
    let layouts = inputs
        .iter()
        .filter(|input| input.input_type == "keyboard" && input.vendor == 1)
        .map(|input| {
            input
                .xkb_layout_names
                .iter()
                .map(move |layout_name| {
                    (
                        Layout::from(layout_name),
                        input
                            .xkb_active_layout_name
                            .as_ref()
                            .map(|active_layout| active_layout == layout_name)
                            .unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();
    // dbg!(&layouts);
    let active_layout = layouts
        .iter()
        .enumerate()
        .find(|(_, layout)| layout.1)
        .map(|(id, _)| id)
        .unwrap_or_default();
    let layouts = layouts
        .into_iter()
        .map(|(layout, _)| layout)
        .collect::<Vec<_>>();
    println!("Layout: {:?}", &layouts[active_layout]);

    let subs = [
        EventType::Input,
        EventType::Window,
    ];

    let mut events = connection.subscribe(&subs).await?;
    while let event = events.next().await {
        println!("{:#?}\n", event?)
    }

    Ok(())
}
