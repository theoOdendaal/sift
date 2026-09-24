
#[repr(u8)]
#[derive(Debug)]
pub enum Error {
    Utf8Parse(std::str::Utf8Error),
    Other(String),

    ElementMismatch { expected: String, found: String },
    UnexpectedRssChannelElement(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8Parse(e) => write!(f, "{e}"),
            Self::Other(e) => write!(f, "{e}"),

            Self::ElementMismatch { expected, found }=> write!(f, "Element mismatch. Expected: {}, found: {}", expected, found),
            Self::UnexpectedRssChannelElement(element) => write!(f, "Unexpected rss channel element: {element}"),

        }
    }
}

impl std::error::Error for Error {}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Parse(value)
    }
}
