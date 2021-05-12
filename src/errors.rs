use std::io::Error as Io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to do something useful")]
    Io(#[from] Io)
}
