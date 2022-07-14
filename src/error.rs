use thiserror::Error;


#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Ipc(#[from] swayipc_types::Error),
    #[error(transparent)]
    Layout(#[from] crate::layout::LayoutError),
    #[error("Device not found")]
    DeviceNotFound,
}
