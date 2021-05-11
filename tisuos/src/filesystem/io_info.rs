
#[derive(Debug, Clone, Copy)]
pub enum IoError {
    FileIdError(usize),
    ReadError,
    NotOpen,
}
