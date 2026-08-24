#[derive(Debug)]
pub enum TokenErrorKind {
    Declaration,
    Start,
    End,
    Text,
    Comment,
    Attribute,
    CharacterData,
    Tag, // FIXME: Tag is too generic.
}

#[derive(Debug)]
pub enum Error {
    UnterminatedToken {
        pos: usize,
        kind: TokenErrorKind,
    },
    UnquotedToken {
        pos: usize,
        kind: TokenErrorKind,
    },
    MissingExpectedChar {
        pos: usize,
        expected_char: char,
        kind: TokenErrorKind,
    },
}

impl std::fmt::Display for TokenErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declaration => write!(f, "Declaration"),
            Self::Start => write!(f, "Start"),
            Self::End => write!(f, "End"),
            Self::Text => write!(f, "Text"),
            Self::Comment => write!(f, "Comment"),
            Self::Attribute => write!(f, "Attribute"),
            Self::CharacterData => write!(f, "CData"),
            Self::Tag => write!(f, "Tag"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnterminatedToken { pos, kind } => {
                write!(f, "Unterminated {} tag at pos: {}", pos, kind)
            }
            Self::UnquotedToken { pos, kind } => {
                write!(f, "Unquoted {} value at pos: {}", pos, kind)
            }
            Self::MissingExpectedChar {
                pos,
                expected_char,
                kind,
            } => write!(
                f,
                "Missing char: {} for {} tag, around pos: {}",
                expected_char, kind, pos
            ),
        }
    }
}

impl std::error::Error for Error {}
