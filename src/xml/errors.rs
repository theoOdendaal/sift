#[repr(u8)]
#[derive(Debug)]
pub enum Error {
    UnexpectedEndOfFile,
    UnexpectedAttributeFormat,

    UnterminatedComment,
    UnterminatedCData,
    UnterminatedProcessingInstruction,

    EmptyTagName,
    EmptyAttributeName,
    EmptyAttributeValue,
    EmptyProcessingInstruction,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfFile => {
                write!(f, "Unexpected EOF")
            }
            Self::UnterminatedComment => {
                write!(f, "Unterminated comment")
            }
            Self::UnterminatedCData => {
                write!(f, "Unterminated CData")
            }
            Self::UnexpectedAttributeFormat => {
                write!(f, "Unexpected attribute format")
            }
            Self::EmptyTagName => {
                write!(f, "Encountered empty tag name")
            }
            Self::EmptyAttributeName => {
                write!(f, "Encountered empty attribute name")
            }
            Self::EmptyAttributeValue => {
                write!(f, "Encountered empty attribute value")
            }
            Self::UnterminatedProcessingInstruction => {
                write!(f, "Unterminated processing instruction")
            }
            Self::EmptyProcessingInstruction => {
                write!(f, "Encountered empty processing instruction")
            }
        }
    }
}

impl std::error::Error for Error {}
