
#[repr(u8)]
#[derive(Debug)]
pub enum Error {
    UnexpectedEndOfFile,
    UnterminatedComment,
    UnterminatedCData,
    UnexpectedAttributeFormat,

}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfFile => {
                write!(f, "Unexpected EOF")
            },
            Self::UnterminatedComment => {
                write!(f, "Unterminated comment")
            },
            Self::UnterminatedCData => {
                write!(f, "Unterminated CData")
            },
            Self::UnexpectedAttributeFormat => {
                write!(f, "Unexpected attribute format")
            },
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum XmlToken<'a> {
    Declaration(&'a [u8]),
    
    DeclarationTagEnd,

    //DocumentType,

    //EntityDeclaration,

    //DocumentTypeTagEnd,

    StartTag(&'a [u8]),

    Attribute { name: &'a [u8], value: &'a [u8]},

    TagEnd {self_closing: bool },

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
            Self::StartTag(b) => write!(f, "StartTag({})", String::from_utf8_lossy(b)),
            Self::Attribute { name, value } => write!(f, "Attribute ( name: {} value: {} )", String::from_utf8_lossy(name), String::from_utf8_lossy(value)),
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
    dt_depth: usize,
}

pub enum XmlState {
    Normal,
    InsideXmlDeclaration,
    AfterStartTagName,
}

impl<'a> From<&'a str> for XmlTokenizer<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            input: value,
            bytes: value.as_bytes(),
            state: XmlState::Normal,
            pos: 0,
            dt_depth: 0,
        }
    }
}

// FIXME: I need to drastically improve the error handling.

// All of the consume errors will assume that whatever they
// need to consume start at the position of self.pos.
impl<'a> XmlTokenizer<'a> {

    fn advance_past_whitespaces(&mut self) {
        if let Some(non_ws) = self.bytes[self.pos..].iter().position(|b| !b.is_ascii_whitespace()) {
            self.pos += non_ws;
        } else {
            self.pos = self.bytes.len();
        }
    }

    // FIXME: Add errors.
    fn consume_tag_name(&mut self) -> Option<Result<&'a [u8], Error>> {
        let mut len = self.pos;

        while len < self.bytes.len() {
            match self.bytes[len] {
                b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/' | b'?' => break,
                _ => len += 1,
            }
        }

        if len == self.pos {
            return None;
        }
        
        let tag_name = &self.bytes[self.pos..len];
        self.pos = len;

        Some(Ok(tag_name))
    }

    fn consume_attribute_name(&mut self) -> Option<Result<&'a [u8], Error>> {
        let mut len = self.pos;

        while len < self.bytes.len() {
            match self.bytes[len] {
                b'=' | b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/' | b'?' => break,
                _ => len += 1,
            }
        }

        if len == self.pos {
            return None;
        }
        
        let attribute_name = &self.bytes[self.pos..len];
        self.pos = len;

        Some(Ok(attribute_name))
    }

    fn consume_attribute_value(&mut self, quote_char: u8) -> Option<Result<&'a [u8], Error>> {
        let mut len = self.pos;

        while len < self.bytes.len() {
            if self.bytes[len] == quote_char {
                break;
            }
            len += 1;
        }

        if len == self.pos {
            return None;
        }
        
        let tag_name = &self.bytes[self.pos..len];
        // Consume end quote.
        self.pos = len + 1;

        Some(Ok(tag_name))
    }

    // Assumes the first byte is the start of the attribute name.
    fn consume_attribute_pair(&mut self) -> Option<Result<(&'a [u8], &'a [u8]), Error>> {
        
        let attribute_name = match self.consume_attribute_name() {
            Some(Ok(name)) => name,
            Some(Err(e)) => return Some(Err(e)),
            None => return None,
        };

        self.advance_past_whitespaces();

        if self.bytes[self.pos] == b'=' {
            self.pos += 1;
        } else {
            return Some(Err(Error::UnexpectedAttributeFormat));
        }
        self.advance_past_whitespaces();

        let quote_char = match self.bytes[self.pos] {
            b'\'' | b'"' => self.bytes[self.pos],
            _ => return Some(Err(Error::UnexpectedAttributeFormat)),
        };
        // Consume quote char.
        self.pos += 1;

        let attribute_value = match self.consume_attribute_value(quote_char) {
            Some(Ok(value)) => value,
            Some(Err(e)) => return Some(Err(e)),
            None => return None,
        };

        Some(Ok((attribute_name, attribute_value)))

    }

    fn consume_text(&mut self) -> Option<Result<&'a [u8], Error>> {
        let idx = self.bytes[self.pos..]
            .iter()
            .position(|&w| w == b'<');

        // Text cannot be unterminated.
        // Malformed XML will be
        // emitted as Text.
        let idx = match idx {
            Some(idx) => idx,
            None => self.bytes.len(),
        };

        let content = &self.bytes[self.pos..self.pos+idx];
        self.pos += idx;
        Some(Ok(content))
    }

    fn consume_comment(&mut self) -> Option<Result<&'a [u8], Error>> {
        let idx = self.bytes[self.pos..]
            .windows(3)
            .position(|w| w == b"-->");

        if let Some(idx) = idx {
            let content = &self.bytes[self.pos..self.pos+idx];
            self.pos += idx + 3;
            Some(Ok(content))
        } else {
            Some(Err(Error::UnterminatedComment))
        }
    }

    fn consume_cdata(&mut self) -> Option<Result<&'a [u8], Error>> {
        let idx = self.bytes[self.pos..]
            .windows(3)
            .position(|w| w == b"]]>");

        if let Some(idx) = idx {
            let content = &self.bytes[self.pos..self.pos+idx];
            self.pos += idx + 3;
            Some(Ok(content))
        } else {
            Some(Err(Error::UnterminatedCData))
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

                if remaining.starts_with(b"<!--") {
                    self.pos += 4;
                    match self.consume_comment() {
                        Some(Ok(comment)) => Some(Ok(XmlToken::Comment(comment))),
                        Some(Err(e)) => Some(Err(e)),
                        None => None,
                    }

                } else if remaining.starts_with(b"<![CDATA[") {
                    self.pos += 9;
                    match self.consume_cdata() {
                        Some(Ok(comment)) => Some(Ok(XmlToken::CharacterData(comment))),
                        Some(Err(e)) => Some(Err(e)),
                        None => None,
                    }

                } else if remaining.starts_with(b"<?xml") {
                    self.pos += 5;
                    self.state = XmlState::InsideXmlDeclaration;
                    return Some(Ok(XmlToken::Declaration(&self.bytes[self.pos-3..self.pos])));

                } else if remaining.starts_with(b"<?") {
                    unimplemented!("<?")


                } else if remaining.starts_with(b"</") {
                    self.pos += 2;
                    let tag_name = match self.consume_tag_name() {
                        Some(Ok(name)) => name,
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    };
                    self.pos += 1;
                    self.state = XmlState::Normal;

                    // No attributes are allowed in end tag.
                    self.advance_past_whitespaces();
                    // FIXME: Add logic if > not found.
                    return Some(Ok(XmlToken::EndTag(tag_name)));

                // Start tag. 
                } else if remaining.starts_with(b"<") {
                    self.pos += 1;
                    let tag_name = match self.consume_tag_name() {
                        Some(Ok(name)) => name,
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    };
                    self.state = XmlState::AfterStartTagName;
                    return Some(Ok(XmlToken::StartTag(tag_name)));

                } else { 
                    let text = match self.consume_text() {
                        Some(Ok(text)) => text,
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    };
                    return Some(Ok(XmlToken::Text(text)));

                }
            },

            XmlState::InsideXmlDeclaration => {
                self.advance_past_whitespaces();

                if self.pos >= self.bytes.len() {
                    return None;
                }
                
                if self.bytes[self.pos..].starts_with(b"?>") {
                    self.pos += 2;
                    self.state = XmlState::Normal;
                    return Some(Ok(XmlToken::DeclarationTagEnd));

                } else {
                    match self.consume_attribute_pair() {
                        Some(Ok((name, value))) => {
                            return Some(Ok(XmlToken::Attribute { name, value }))
                        },
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    }
                }
            },

            XmlState::AfterStartTagName => {
                self.advance_past_whitespaces();
                
                if self.pos >= self.bytes.len() {
                    return None;
                }

                self.advance_past_whitespaces();

                let remaining = &self.bytes[self.pos..];

                if remaining.starts_with(b"/>") {
                    self.pos += 2;
                    self.state = XmlState::Normal;
                    return Some(Ok(XmlToken::TagEnd { self_closing: true }));
                } else if remaining.starts_with(b">") {
                    self.pos += 1;
                    self.state = XmlState::Normal;
                    return Some(Ok(XmlToken::TagEnd { self_closing: false }));
                } else {
                    match self.consume_attribute_pair() {
                        Some(Ok((name, value))) => {
                            return Some(Ok(XmlToken::Attribute { name, value }))
                        },
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None,
                    }
                }

            }




        }
    }
}
