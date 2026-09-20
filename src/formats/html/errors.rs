#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    UnexpectedEndOfFile,
}

#[derive(Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub pos : usize,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfFile=> f.write_str("Unexpected end of file"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error ( kind: {} )", self.kind)
    }
}

impl std::error::Error for Error {}
