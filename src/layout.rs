#[derive(Debug)]
pub struct Layout {
    pub id: u32,
    pub name: String,
}

impl From<(usize, &String)> for Layout {
    fn from(layout: (usize, &String)) -> Self {
        Self {
            name: layout.1.clone(),
            id: layout.0 as u32,
        }
    }
}
