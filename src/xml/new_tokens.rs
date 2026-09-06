#[derive(Debug)]
pub enum Error {
    UnexpectedNullCharacter,
    UnexpectedAttributeFormat,
    UnquotedAttributeValue,
    UnterminatedAttributeValueQuote,
    UnexpectedXmlDeclarationFormat,
    UnexpectedXmlDeclarationAttribute,
    UnterminatedComment,
    UnterminatedCharacterData,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedNullCharacter => {
                write!(f, "Unexpected null character encountered")
            }

            Self::UnexpectedAttributeFormat => {
                write!(f, "Unexpected attribute format encountered")
            }

            Self::UnquotedAttributeValue => {
                write!(f, "Unquoted attribute value encountered")
            }

            Self::UnterminatedAttributeValueQuote => {
                write!(f, "Unterminated attribute value quote")
            }

            Self::UnexpectedXmlDeclarationFormat => {
                write!(f, "Unexpected XML declaration format encountered")
            }

            Self::UnexpectedXmlDeclarationAttribute => {
                write!(f, "Unexpected attribute found in xml declaration")
            }

            Self::UnterminatedComment => {
                write!(f, "Unterminated comment")
            }

            Self::UnterminatedCharacterData => {
                write!(f, "Unterminated CData")
            }
        }
    }
}

impl std::error::Error for Error {}

// FIXME: Incorporate namespaces.

#[derive(Debug)]
pub enum XmlToken<'a> {
    // <?xml ... ?>
    Declaration {
        name: &'a str,
        version: &'a str,
        encoding: Option<&'a str>,
        standalone: Option<&'a str>,
    },

    // <?* .. ?>
    ProcessingInstruction {
        target: &'a str,
        data: &'a str,
    },

    // https://www.liquid-technologies.com/Reference/Glossary/XML_DocType.html
    DocumentType {
        name: &'a str,
        external_id: &'a str,
    },

    EntityDeclaration {
        name: &'a str,
        definition: &'a str,
    },

    // FIXME: Incorporate later.
    //AttrbuteListDeclaration,
    DocumentTypeEnd,

    CharacterData(&'a str),

    StartTag {
        name: &'a str,
    },

    // https://www.liquid-technologies.com/Reference/Glossary/XML_Attribute.html
    Attribute {
        name: &'a str,
        value: &'a str,
    },

    TagEnd {
        self_closing: bool,
    },

    EndTag {
        name: &'a str,
    },

    Text(&'a str),

    Comment(&'a str),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum XmlState {
    Normal,

    TagOpen,

    TagOpenBang,

    InsideComment,

    InsideCharacterData,

    DeclarationStart,

    InsideXmlDeclaration,

    ProcessingInstructionStart,

    StartTagName,

    EndTagName,

    SelfClosingStartTag,

    AfterStartTagName,

    InsideAttribute,

    DocumentTypeStart,
}

pub struct XmlTokenizer<'a> {
    input: &'a str,

    chars: std::str::CharIndices<'a>,

    current: Option<(usize, char)>,

    state: XmlState,

    mark: usize,

    depth: usize,
}

impl<'a> From<&'a str> for XmlTokenizer<'a> {
    fn from(value: &'a str) -> Self {
        let mut chars = value.char_indices();
        let current = chars.next();

        Self {
            input: value,
            chars,
            current,
            state: XmlState::Normal,
            mark: 0,
            depth: 0,
        }
    }
}

impl<'a> XmlTokenizer<'a> {
    #[inline]
    fn pos(&self) -> usize {
        self.current.map_or(self.input.len(), |(offset, _)| offset)
    }

    #[inline]
    fn mark(&mut self) {
        self.mark = self.pos();
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        self.current.map(|(_, c)| c)
    }

    #[inline]
    fn consume(&mut self) -> Option<char> {
        let (_, c) = self.current?;
        self.current = self.chars.next();
        Some(c)
    }

    #[inline]
    fn slice_from_mark(&self) -> &'a str {
        let end = self.pos();
        &self.input[self.mark..end]
    }

    fn consume_char_run(&mut self, stop: impl Fn(char) -> bool) -> Option<&'a str> {
        self.mark();

        while let Some(c) = self.peek() {
            if stop(c) {
                break;
            }
            self.consume();
        }

        if self.pos() > self.mark {
            Some(self.slice_from_mark())
        } else {
            None
        }
    }

    fn consume_whitespaces(&mut self) {
        while let Some(c) = self.peek() {
            if c != ' ' {
                break;
            }
            self.consume();
        }
    }
}

impl<'a> Iterator for XmlTokenizer<'a> {
    type Item = Result<XmlToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos() >= self.input.len() {
            return None;
        }

        loop {
            match self.state {
                XmlState::Normal => {
                    if let Some(tok) = self.consume_char_run(|c| matches!(c, '<' | '\0')) {
                        return Some(Ok(XmlToken::Text(tok)));
                    }

                    match self.consume() {
                        Some('<') => {
                            self.state = XmlState::TagOpen;
                        }
                        Some('\0') => return Some(Err(Error::UnexpectedNullCharacter)),

                        None => return None,

                        _ => unimplemented!("1"),
                    }
                }

                XmlState::TagOpen => match self.peek() {
                    Some('!') => {
                        self.consume();
                        self.state = XmlState::TagOpenBang
                    }

                    Some('?') => {
                        self.state = XmlState::DeclarationStart;
                        self.consume();
                        self.mark();
                    }

                    Some('/') => {
                        self.state = XmlState::EndTagName;
                        self.consume();
                    }

                    Some(c) if c.is_ascii_alphabetic() => {
                        self.state = XmlState::StartTagName;
                    }

                    _ => unimplemented!("2"),
                },

                XmlState::DeclarationStart => {
                    let remaining = &self.input[self.pos()..];

                    if remaining.starts_with("xml ") {
                        self.state = XmlState::InsideXmlDeclaration;
                    } else {
                        self.state = XmlState::ProcessingInstructionStart;
                    }
                }

                XmlState::StartTagName => {
                    if let Some(tok) = self.consume_char_run(|c| matches!(c, '>' | '/' | ' ')) {
                        return Some(Ok(XmlToken::StartTag { name: tok }));
                    }

                    match self.peek() {
                        Some('>') => {
                            self.consume();
                            self.state = XmlState::Normal;
                            return Some(Ok(XmlToken::TagEnd {
                                self_closing: false,
                            }));
                        }
                        Some('/') => {
                            self.consume();
                            self.state = XmlState::SelfClosingStartTag;
                        }
                        Some(' ') => {
                            self.consume();
                            self.state = XmlState::AfterStartTagName;
                        }

                        _ => unimplemented!("3"),
                    }
                }

                XmlState::EndTagName => {
                    if let Some(tok) = self.consume_char_run(|c| matches!(c, '>' | ' ')) {
                        return Some(Ok(XmlToken::EndTag { name: tok }));
                    }
                    self.consume_whitespaces();

                    match self.peek() {
                        Some('>') => {
                            self.consume();
                            self.state = XmlState::Normal;
                            return Some(Ok(XmlToken::TagEnd {
                                self_closing: false,
                            }));
                        }

                        _ => unimplemented!("5"),
                    }
                }

                XmlState::SelfClosingStartTag => match self.peek() {
                    Some('>') => {
                        self.consume();
                        self.state = XmlState::Normal;
                        return Some(Ok(XmlToken::TagEnd { self_closing: true }));
                    }

                    _ => unimplemented!("4"),
                },

                XmlState::AfterStartTagName => {
                    self.consume_whitespaces();

                    match self.peek() {
                        // Reconsume in StartTagName state, as no attributes
                        // are found.
                        Some('>') | Some('/') => {
                            self.state = XmlState::StartTagName;
                        }
                        Some(c) if c.is_ascii_alphabetic() => {
                            self.state = XmlState::InsideAttribute;
                        }

                        _ => unimplemented!("6"),
                    }
                }

                XmlState::InsideAttribute => {
                    let attribute_name = match self.consume_char_run(|c| matches!(c, '=')) {
                        Some(tok) => tok,
                        None => return Some(Err(Error::UnexpectedAttributeFormat)),
                    };
                    self.consume();

                    self.consume_whitespaces();

                    let quote_char = match self.consume() {
                        Some('\'') => '\'',
                        Some('"') => '"',
                        _ => return Some(Err(Error::UnquotedAttributeValue)),
                    };

                    let attribute_value = match self.consume_char_run(|c| c == quote_char) {
                        Some(tok) => tok,
                        None => return Some(Err(Error::UnterminatedAttributeValueQuote)),
                    };
                    // Consume closing char.
                    self.consume();

                    self.state = XmlState::AfterStartTagName;
                    return Some(Ok(XmlToken::Attribute {
                        name: attribute_name,
                        value: attribute_value,
                    }));
                }

                XmlState::InsideXmlDeclaration => {
                    // The XmlDeclaration format should be strictly:
                    // version -> encoding -> standalone

                    let declaration_name = match self.consume_char_run(|c| matches!(c, ' ' | '?')) {
                        Some(tok) => tok,
                        None => return Some(Err(Error::UnexpectedXmlDeclarationFormat)),
                    };

                    //  -- version
                    self.consume_whitespaces();

                    // Ensure first attribute name is version.
                    match self.consume_char_run(|c| matches!(c, '=')) {
                        Some(tok) if tok == "version" => tok,
                        _ => return Some(Err(Error::UnexpectedXmlDeclarationFormat)),
                    };
                    self.consume();

                    let quote_char = match self.consume() {
                        Some('\'') => '\'',
                        Some('"') => '"',
                        _ => return Some(Err(Error::UnquotedAttributeValue)),
                    };

                    let version_value = match self.consume_char_run(|c| c == quote_char) {
                        Some(tok) => tok,
                        None => return Some(Err(Error::UnterminatedAttributeValueQuote)),
                    };
                    self.consume();
                    self.consume_whitespaces();

                    // Parse remaining optional attributes.
                    let mut encoding_value = None;
                    let mut standalone_value = None;

                    loop {
                        if let Some(c) = self.peek()
                            && c.is_ascii_alphabetic()
                        {
                            let attribute_name = match self
                                .consume_char_run(|c| matches!(c, '=' | '?'))
                            {
                                Some(tok) => tok,
                                None => return Some(Err(Error::UnexpectedXmlDeclarationFormat)),
                            };

                            if self.consume() == Some('?') {
                                return Some(Err(Error::UnexpectedXmlDeclarationFormat));
                            }

                            let quote_char = match self.consume() {
                                Some('\'') => '\'',
                                Some('"') => '"',
                                _ => return Some(Err(Error::UnquotedAttributeValue)),
                            };

                            let attribute_value = match self.consume_char_run(|c| c == quote_char) {
                                Some(tok) => tok,
                                None => return Some(Err(Error::UnterminatedAttributeValueQuote)),
                            };
                            self.consume();
                            self.consume_whitespaces();

                            match attribute_name {
                                "encoding" => encoding_value = Some(attribute_value),
                                "standalone" => standalone_value = Some(attribute_value),
                                _ => return Some(Err(Error::UnexpectedXmlDeclarationAttribute)),
                            }
                        } else {
                            break;
                        }
                    }

                    if self.input[self.pos()..].starts_with("?>") {
                        self.consume();
                        self.consume();
                    }

                    self.state = XmlState::Normal;
                    return Some(Ok(XmlToken::Declaration {
                        name: declaration_name,
                        version: version_value,
                        encoding: encoding_value,
                        standalone: standalone_value,
                    }));
                }

                XmlState::TagOpenBang => {
                    let remaining = &self.input[self.pos()..];
                    if remaining.starts_with("--") {
                        self.consume();
                        self.consume();
                        self.state = XmlState::InsideComment;
                    } else if remaining.starts_with("[CDATA[") {
                        for _ in 0..7 {
                            self.consume();
                        }
                        self.state = XmlState::InsideCharacterData;
                    } else if remaining.starts_with("DOCTYPE") {
                        for _ in 0..7 {
                            self.consume();
                        }
                        self.state = XmlState::DocumentTypeStart;
                    } else {
                        unimplemented!("TagOpenBang")
                    }
                }

                XmlState::InsideComment => {
                    self.mark();

                    let mut hyphen_count: usize = 0;
                    while let Some(c) = self.peek() {
                        match c {
                            '-' => {
                                // TODO: XML strictly disallowed the occurrence of --, except for
                                // termination of comments.
                                hyphen_count += 1;
                                self.consume();
                            }

                            '>' if hyphen_count == 2 => {
                                let comment_value = &self.input[self.mark..self.pos() - 2];
                                self.consume();
                                self.state = XmlState::Normal;
                                return Some(Ok(XmlToken::Comment(comment_value)));
                            }
                            _ => {
                                hyphen_count = 0;
                                self.consume();
                            }
                        }
                    }
                    return Some(Err(Error::UnterminatedComment));
                }

                XmlState::InsideCharacterData => {
                    self.mark();

                    let mut bracket_count: usize = 0;
                    while let Some(c) = self.peek() {
                        match c {
                            ']' => {
                                bracket_count += 1;
                                self.consume();
                            }

                            '>' if bracket_count == 2 => {
                                let comment_value = &self.input[self.mark..self.pos() - 2];
                                self.consume();
                                self.state = XmlState::Normal;
                                return Some(Ok(XmlToken::CharacterData(comment_value)));
                            }
                            _ => {
                                bracket_count = 0;
                                self.consume();
                            }
                        }
                    }
                    return Some(Err(Error::UnterminatedCharacterData));
                }

                XmlState::ProcessingInstructionStart => {
                    unimplemented!("ProcessingInstructionStart")
                }

                XmlState::DocumentTypeStart => {
                    unimplemented!("DocumentTypeStart")
                }
            }
        }
    }
}
