
#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedEndOfFile,
    UnterminatedComment,
    UnterminatedAttributeName,
    UnterminatedAttributeValue,
    UnterminatedCharacterData,
    UnterminatedDocumentType,
    UnterminatedDeclaration,
    UnterminatedTagName,
    UnterminatedTag,
    UnterminatedExternalIdentifier,
    UnterminatedEntityDeclaration,

    UnexpectedAttributeFormat,

    UnknownExternalIdentifier,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub pos : usize,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfFile=> f.write_str("Unexpected end of file"),
            Self::UnterminatedComment => f.write_str("Unterminated comment"),
            Self::UnterminatedAttributeName => f.write_str("Unterminated attribute name"),
            Self::UnterminatedAttributeValue => f.write_str("Unterminated attribute value"),
            Self::UnterminatedCharacterData => f.write_str("Unterminated character data"),
            Self::UnterminatedDocumentType => f.write_str("Unterminated document type"),
            Self::UnterminatedDeclaration => f.write_str("Unterminated declaration"),
            Self::UnterminatedTagName => f.write_str("Unterminated tag name"),
            Self::UnterminatedTag => f.write_str("Unterminated tag"),
            Self::UnterminatedExternalIdentifier => f.write_str("Unterminated external identifier"),
            Self::UnterminatedEntityDeclaration => f.write_str("Unterminated entity declaration"),
            Self::UnexpectedAttributeFormat => f.write_str("Unexpected attribute format"),
            Self::UnknownExternalIdentifier => f.write_str("Unknown external identifier"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error ( kind: {} )", self.kind)
    }
}

impl std::error::Error for Error {}

/*
#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedEndOfFile,
    UnexpectedAttributeFormat,
    UnterminatedComment,
    UnterminatedCData,
    UnterminatedProcessingInstruction,
    UnterminatedDocumentType,
    EmptyTagName,
    EmptyAttributeName,
    EmptyAttributeValue,
    EmptyProcessingInstruction,
    UnknownExternalIdentifier,
    UnknownXmlDeclaration,
    MalformedProcessingInstruction,
}

impl From<crate::bs::Error> for crate::xml::errors::Error {
    fn from(value: crate::bs::Error) -> Self {
        match value {
            crate::bs::Error::UnexpectedEndOfFile => Self::UnexpectedEndOfFile
        }
    }
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
            Self::UnterminatedDocumentType => {
                write!(f, "Unterminated document type")
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
            Self::UnknownExternalIdentifier => {
                write!(f, "Unknown external identifier")
            }
            Self::UnknownXmlDeclaration => {
                write!(f, "Unknown xml declaration")
            }
            Self::MalformedProcessingInstruction => {
                write!(f, "Malformed processing instruction")
            }
        }
    }
}

impl std::error::Error for Error {}
*/
