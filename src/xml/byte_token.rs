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

#[derive(Debug)]
pub enum XmlToken<'a> {
    Declaration(&'a [u8]),

    DeclarationTagEnd,

    ProcessingInstruction { target: &'a [u8], data: &'a [u8] },

    DocumentType(&'a [u8]),

    //EntityDeclaration,

    //DocumentTypeTagEnd,
    StartTag(&'a [u8]),

    Attribute { name: &'a [u8], value: &'a [u8] },

    TagEnd { self_closing: bool },

    EndTag(&'a [u8]),

    Text(&'a [u8]),

    Comment(&'a [u8]),

    CharacterData(&'a [u8]),
}

impl<'a> std::fmt::Display for XmlToken<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declaration(b) => write!(f, "Declaration({})", String::from_utf8_lossy(b)),
            Self::DeclarationTagEnd => write!(f, "DeclarationTagEnd"),
            Self::ProcessingInstruction { target, data } => write!(
                f,
                "ProcessingInstruction(target={}, data={})",
                String::from_utf8_lossy(target),
                String::from_utf8_lossy(data)
            ),
            Self::DocumentType(b) => write!(f, "DocumentType({})", String::from_utf8_lossy(b)),
            Self::StartTag(b) => write!(f, "StartTag({})", String::from_utf8_lossy(b)),
            Self::Attribute { name, value } => write!(
                f,
                "Attribute (name={}, value={})",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(value)
            ),
            Self::TagEnd { self_closing } => write!(f, "TagEnd({})", self_closing),
            Self::EndTag(b) => write!(f, "EndTag({})", String::from_utf8_lossy(b)),
            Self::Text(b) => write!(f, "Text({})", String::from_utf8_lossy(b)),
            Self::Comment(b) => write!(f, "Comment({})", String::from_utf8_lossy(b)),
            Self::CharacterData(b) => write!(f, "CData({})", String::from_utf8_lossy(b)),
        }
    }
}

pub struct XmlTokenizer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    state: XmlState,
    pos: usize,
    //dt_depth: usize,
}

enum XmlState {
    Normal,
    InsideXmlDeclaration,
    AfterStartTagName,
    AfterDoctypeName,
    InternalSubset,
}

impl<'a> From<&'a str> for XmlTokenizer<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            input: value,
            bytes: value.as_bytes(),
            state: XmlState::Normal,
            pos: 0,
            //dt_depth: 0,
        }
    }
}

// FIXME: I need to drastically improve the errors.

// All of the consume errors will assume that whatever they
// need to consume start at the position of self.pos.
impl<'a> XmlTokenizer<'a> {
    #[inline]
    fn advance_past_whitespaces(&mut self) {
        if let Some(non_ws) = self.bytes[self.pos..]
            .iter()
            .position(|b| !b.is_ascii_whitespace())
        {
            self.pos += non_ws;
        } else {
            self.pos = self.bytes.len();
        }
    }

    #[inline]
    fn consume_tag_name(&mut self) -> Result<&'a [u8], Error> {
        let mut len = self.pos;

        while len < self.bytes.len() {
            match self.bytes[len] {
                b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/' | b'?' => break,
                _ => len += 1,
            }
        }

        if len == self.pos {
            return Err(Error::EmptyTagName);
        }

        let tag_name = &self.bytes[self.pos..len];
        self.pos = len;

        Ok(tag_name)
    }

    #[inline]
    fn consume_processing_instruction_data(&mut self) -> Result<&'a [u8], Error> {
        let mut len = self.pos;

        while len < self.bytes.len() {
            if self.bytes[len] == b'?' {
                break;
            }
            len += 1;
        }

        if len == self.pos {
            return Err(Error::EmptyProcessingInstruction);
        }

        let data_value = &self.bytes[self.pos..len];
        self.pos = len;

        Ok(data_value)
    }

    #[inline]
    fn consume_attribute_name(&mut self) -> Result<&'a [u8], Error> {
        let mut len = self.pos;

        while len < self.bytes.len() {
            match self.bytes[len] {
                b'=' | b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/' | b'?' => break,
                _ => len += 1,
            }
        }

        if len >= self.bytes.len() {
            return Err(Error::UnexpectedEndOfFile);
        }

        if len == self.pos {
            return Err(Error::EmptyAttributeName);
        }

        let attribute_name = &self.bytes[self.pos..len];
        self.pos = len;

        Ok(attribute_name)
    }

    #[inline]
    fn consume_attribute_value(&mut self, quote_char: u8) -> Result<&'a [u8], Error> {
        let mut len = self.pos;

        while len < self.bytes.len() {
            if self.bytes[len] == quote_char {
                break;
            }
            len += 1;
        }

        if len >= self.bytes.len() {
            return Err(Error::UnexpectedEndOfFile);
        }

        if len == self.pos {
            return Err(Error::EmptyAttributeValue);
        }

        let tag_name = &self.bytes[self.pos..len];
        // Consume end quote.
        self.pos = len + 1;

        Ok(tag_name)
    }

    // Assumes the first byte is the start of the attribute name.
    #[inline]
    fn consume_attribute_pair(&mut self) -> Result<(&'a [u8], &'a [u8]), Error> {
        let attribute_name = self.consume_attribute_name()?; 

        self.advance_past_whitespaces();
        if self.pos >= self.bytes.len() {
            return Err(Error::UnexpectedEndOfFile);
        }

        if self.bytes[self.pos] == b'=' {
            self.pos += 1;
        } else {
            return Err(Error::UnexpectedAttributeFormat);
        }
        self.advance_past_whitespaces();
        if self.pos >= self.bytes.len() {
            return Err(Error::UnexpectedEndOfFile);
        }

        let quote_char = match self.bytes[self.pos] {
            b'\'' | b'"' => self.bytes[self.pos],
            _ => return Err(Error::UnexpectedAttributeFormat),
        };
        // Consume quote char.
        self.pos += 1;
    
        let attribute_value = self.consume_attribute_value(quote_char)?;

        Ok((attribute_name, attribute_value))
    }

    fn consume_text(&mut self) -> Result<&'a [u8], Error> {
        let idx = self.bytes[self.pos..].iter().position(|&w| w == b'<');

        // Text cannot be unterminated.
        // Malformed XML will be
        // emitted as Text.
        let idx = match idx {
            Some(idx) => idx,
            None => self.bytes.len(),
        };

        let content = &self.bytes[self.pos..self.pos + idx];
        self.pos += idx;
        Ok(content)
    }

    fn consume_comment(&mut self) -> Result<&'a [u8], Error> {
        let idx = self.bytes[self.pos..].windows(3).position(|w| w == b"-->");

        if let Some(idx) = idx {
            let content = &self.bytes[self.pos..self.pos + idx];
            self.pos += idx + 3;
            Ok(content)
        } else {
            Err(Error::UnterminatedComment)
        }
    }

    fn consume_cdata(&mut self) -> Result<&'a [u8], Error> {
        let idx = self.bytes[self.pos..].windows(3).position(|w| w == b"]]>");

        if let Some(idx) = idx {
            let content = &self.bytes[self.pos..self.pos + idx];
            self.pos += idx + 3;
            Ok(content)
        } else {
            Err(Error::UnterminatedCData)
        }
    }
}

impl<'a> Iterator for XmlTokenizer<'a> {
    type Item = Result<XmlToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        match self.state {
            XmlState::Normal => {
                let remaining = &self.bytes[self.pos..];

                if remaining[0] == b'<' {
                    match remaining.get(1) {
                        Some(b'!') => {
                            if remaining.starts_with(b"<!--") {
                                self.pos += 4;
                                match self.consume_comment() {
                                    Ok(comment) => Some(Ok(XmlToken::Comment(comment))),
                                    Err(e) => Some(Err(e)),
                                }
                            } else if remaining.starts_with(b"<![CDATA[") {
                                self.pos += 9;
                                match self.consume_cdata() {
                                    Ok(comment) => Some(Ok(XmlToken::CharacterData(comment))),
                                    Err(e) => Some(Err(e)),
                                }
                            } else if remaining.starts_with(b"<!DOCTYPE") {
                                self.pos += 9;

                                self.advance_past_whitespaces();
                                if self.pos >= self.bytes.len() {
                                    return Some(Err(Error::UnexpectedEndOfFile));
                                }

                                let name = match self.consume_tag_name() {
                                    Ok(name) => name,
                                    Err(e) => return Some(Err(e)),
                                };
                                self.state = XmlState::AfterDoctypeName;
                                Some(Ok(XmlToken::DocumentType(name)))
                            } else {
                                unimplemented!("!")
                            }
                        }

                        Some(b'?') => {
                            if remaining.starts_with(b"<?xml ") {
                                self.pos += 5;
                                self.state = XmlState::InsideXmlDeclaration;
                                Some(Ok(XmlToken::Declaration(
                                    &self.bytes[self.pos - 3..self.pos],
                                )))
                            } else if remaining.starts_with(b"<?") {
                                self.pos += 2;
                                let target = match self.consume_tag_name() {
                                    Ok(name) => name,
                                    Err(e) => return Some(Err(e)),
                                };

                                self.advance_past_whitespaces();
                                if self.pos >= self.bytes.len() {
                                    return Some(Err(Error::UnexpectedEndOfFile));
                                }

                                let data = match self.consume_processing_instruction_data() {
                                    Ok(data) => data,
                                    Err(e) => return Some(Err(e)),
                                };

                                if !self.bytes[self.pos..].starts_with(b"?>") {
                                    return Some(Err(Error::UnterminatedProcessingInstruction));
                                }

                                self.pos += 2;
                                self.state = XmlState::Normal;
                                Some(Ok(XmlToken::ProcessingInstruction { target, data }))
                            } else {
                                unimplemented!("?")
                            }
                        }

                        // End tag
                        Some(b'/') => {
                            self.pos += 2;
                            let tag_name = match self.consume_tag_name() {
                                Ok(name) => name,
                                Err(e) => return Some(Err(e)),
                            };
                            self.advance_past_whitespaces();
                            if self.pos >= self.bytes.len() {
                                return None;
                            }

                            self.pos += 1;
                            self.state = XmlState::Normal;

                            // No attributes are allowed in end tag.
                            self.advance_past_whitespaces();
                            if self.pos >= self.bytes.len() {
                                return None;
                            }
                            // FIXME: Add logic if > not found.
                            Some(Ok(XmlToken::EndTag(tag_name)))
                        }

                        Some(_) => {
                            self.pos += 1;
                            let tag_name = match self.consume_tag_name() {
                                Ok(name) => name,
                                Err(e) => return Some(Err(e)),
                            };
                            self.state = XmlState::AfterStartTagName;
                            Some(Ok(XmlToken::StartTag(tag_name)))
                        }

                        None => unimplemented!(),
                    }
                } else {
                    let text = match self.consume_text() {
                        Ok(text) => text,
                        Err(e) => return Some(Err(e)),
                    };
                    Some(Ok(XmlToken::Text(text)))
                }
            }

            XmlState::InsideXmlDeclaration => {
                self.advance_past_whitespaces();

                if self.pos >= self.bytes.len() {
                    return None;
                }

                if self.bytes[self.pos..].starts_with(b"?>") {
                    self.pos += 2;
                    self.state = XmlState::Normal;
                    Some(Ok(XmlToken::DeclarationTagEnd))
                } else {
                    match self.consume_attribute_pair() {
                        Ok((name, value)) => Some(Ok(XmlToken::Attribute { name, value })),
                        Err(e) => Some(Err(e)),
                    }
                }
            }

            XmlState::AfterStartTagName => {
                self.advance_past_whitespaces();
                if self.pos >= self.bytes.len() {
                    return None;
                }

                let remaining = &self.bytes[self.pos..];

                if remaining.starts_with(b"/>") {
                    self.pos += 2;
                    self.state = XmlState::Normal;
                    Some(Ok(XmlToken::TagEnd { self_closing: true }))
                } else if remaining.starts_with(b">") {
                    self.pos += 1;
                    self.state = XmlState::Normal;
                    Some(Ok(XmlToken::TagEnd {
                        self_closing: false,
                    }))
                } else {
                    match self.consume_attribute_pair() {
                        Ok((name, value)) => Some(Ok(XmlToken::Attribute { name, value })),
                        Err(e) => Some(Err(e)),
                    }
                }
            }

            XmlState::AfterDoctypeName => {
                unimplemented!("AfterDoctypeName")
            }

            XmlState::InternalSubset => {
                unimplemented!("InternalSubset")
            }
        }
    }
}
