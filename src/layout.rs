use swayipc_async::Input;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Layout {
    pub id: usize,
    pub name: String,
}

impl From<(usize, &String)> for Layout {
    fn from(layout: (usize, &String)) -> Self {
        Self {
            name: layout.1.clone(),
            id: layout.0,
        }
    }
}

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("layout detection error: {0}")]
    LayoutDetection(String),
}

impl TryFrom<Input> for Layout {
    type Error = LayoutError;
    fn try_from(input: Input) -> Result<Self, Self::Error> {
        Ok(Self {
            id: input
                .xkb_active_layout_index
                .ok_or_else(|| LayoutError::LayoutDetection("layout index not found".to_owned()))?
                as usize,
            name: input
                .xkb_active_layout_name
                .ok_or_else(|| LayoutError::LayoutDetection("layout name not found".to_owned()))?,
        })
    }
}
