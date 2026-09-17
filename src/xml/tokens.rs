use crate::xml::errors::Error;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum ExternalIdentifier {
    System,
    Public,
}

struct MarkupDeclaration<'a> {
    name: &'a str,
    external_identifier: ExternalIdentifier,
    literal: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum XmlToken<'a> {
    Declaration(&'a [u8]),

    DeclarationTagEnd,

    ProcessingInstruction {
        target: &'a [u8],
        data: &'a [u8],
    },

    // FIXME: Combine external identifier and DocumentType tag
    //DocumentType(MarkupDeclaration),
    DocumentType(&'a [u8]),
    
    // FIXME: The literal is always quoted
    // for SYSTEM. For PUBLIC there can
    // be multiple literals, but each of these
    // are also quoted.
    ExternalIdentifier {
        identifier_type: ExternalIdentifier,
        literal: &'a [u8],
    },

    InternalSubsetTagStart,

    //EntityDeclaration(MarkupDeclaration)
    EntityDeclaration(&'a [u8]),

    InternalSubsetTagEnd,

    DocumentTypeTagEnd,

    StartTag(&'a [u8]),

    Attribute {
        name: &'a [u8],
        value: &'a [u8],
    },

    TagEnd {
        self_closing: bool,
    },

    EndTag(&'a [u8]),

    Text(&'a [u8]),

    Comment(&'a [u8]),

    CharacterData(&'a [u8]),
}

impl std::fmt::Display for ExternalIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "SYSTEM"),
            Self::Public => write!(f, "PUBLIC"),
        }
    }
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

            Self::ExternalIdentifier {
                identifier_type,
                literal,
            } => write!(
                f,
                "ExternalIdentifier(type={}, literal={})",
                identifier_type,
                String::from_utf8_lossy(literal)
            ),

            Self::InternalSubsetTagStart => write!(f, "InternalSubsetTagStart"),

            Self::EntityDeclaration(b) => {
                write!(f, "EntityDeclaration({})", String::from_utf8_lossy(b))
            }

            Self::InternalSubsetTagEnd => write!(f, "InternalSubsetTagEnd"),

            Self::DocumentTypeTagEnd => write!(f, "DocumentTypeTagEnd"),

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
    bytes: &'a [u8],
    state: XmlState,
    pos: usize,
}

enum XmlState {
    Normal,
    InsideXmlDeclaration,
    AfterStartTagName,

    AfterDoctypeName,
    InsideInternalSubset,
}

impl<'a> From<&'a str> for XmlTokenizer<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            bytes: value.as_bytes(),
            state: XmlState::Normal,
            pos: 0,
        }
    }
}

impl<'a> From<&'a [u8]> for XmlTokenizer<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self {
            bytes: value,
            state: XmlState::Normal,
            pos: 0,
        }
    }
}

// FIXME: I need to drastically improve the errors.

// All of the consume fn's will assume that whatever they
// need to consume start at the position of self.pos.
impl<'a> XmlTokenizer<'a> {
    // Continue until a non-whitespaces
    // have been consumed. If the byte
    // at self.pos is not a whitespace,
    // then self.pos is not advanved.
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
    fn consume_quoted_attribute_value(&mut self, quote_char: u8) -> Result<&'a [u8], Error> {
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

        let attribute_value = &self.bytes[self.pos..len];
        // Consume end quote. Even though this quote
        // does break the loop, it isn't used to
        // make a decision on the succeeding state.
        self.pos = len + 1;

        Ok(attribute_value)
    }

    // Assumes the first byte is the start of the attribute name.
    #[inline]
    fn consume_quoted_attribute_pair(&mut self) -> Result<(&'a [u8], &'a [u8]), Error> {
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
        // Consume quote byte.
        self.pos += 1;

        let attribute_value = self.consume_quoted_attribute_value(quote_char)?;

        Ok((attribute_name, attribute_value))
    }

    fn consume_text(&mut self) -> Result<&'a [u8], Error> {
        let idx = self.bytes[self.pos..].iter().position(|&w| w == b'<');

        // Text cannot be unterminated.
        // Malformed XML will be
        // emitted as Text.
        let idx = match idx {
            Some(idx) => idx,
            None => self.bytes.len() - self.pos,
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

    // FIXME: This actually need to be properly broken down
    // into the various components of an entity declaration.
    fn consume_entity_declaration(&mut self) -> Result<&'a [u8], Error> {
        let mut len = self.pos;

        while len < self.bytes.len() {
            match self.bytes[len] {
                b'>' => break,
                _ => len += 1,
            }
        }

        if len >= self.bytes.len() {
            return Err(Error::UnexpectedEndOfFile);
        }

        let entity_declation = &self.bytes[self.pos..len];
        self.pos = len;

        Ok(entity_declation)
    }
}

impl<'a> Iterator for XmlTokenizer<'a> {
    type Item = Result<XmlToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_past_whitespaces();
        if self.pos >= self.bytes.len() {
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

                            // Consume all whitespaces after
                            // end tag name.
                            self.advance_past_whitespaces();
                            if self.pos >= self.bytes.len() {
                                return None;
                            }

                            // FIXME: For now I'm just going
                            // to lazily skip all bytes
                            // after an end tag name. But I do need
                            // to add better validation to ensure
                            // that no attributes are included in
                            // end tags.
                            while self.pos < self.bytes.len() {
                                match self.bytes[self.pos] {
                                    b'>' => break,
                                    _ => self.pos += 1,
                                }
                            }

                            // > not found before EOF.
                            if self.pos == self.bytes.len() {
                                return Some(Err(Error::UnexpectedEndOfFile));
                            }

                            self.pos += 1;
                            self.state = XmlState::Normal;

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
                    match self.consume_quoted_attribute_pair() {
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
                    match self.consume_quoted_attribute_pair() {
                        Ok((name, value)) => Some(Ok(XmlToken::Attribute { name, value })),
                        Err(e) => Some(Err(e)),
                    }
                }
            }

            // FIXME: Refine this state.
            XmlState::AfterDoctypeName => {
                self.advance_past_whitespaces();
                if self.pos >= self.bytes.len() {
                    return None;
                }

                let remaining = &self.bytes[self.pos..];

                if remaining.starts_with(b">") {
                    self.pos += 1;
                    self.state = XmlState::Normal;
                    Some(Ok(XmlToken::DocumentTypeTagEnd))
                } else if remaining.starts_with(b"SYSTEM") || remaining.starts_with(b"PUBLIC") {
                    let mut len = self.pos;

                    while len < self.bytes.len() {
                        if self.bytes[len..].starts_with(b">")
                            || self.bytes[len..].starts_with(b"[")
                        {
                            break;
                        }
                        len += 1;
                    }

                    if len >= self.bytes.len() {
                        return Some(Err(Error::UnexpectedEndOfFile));
                    }
                    // FIXME: Because we only yield on > or [, trailing whitespaces
                    // are included. Improve this when refactoring.

                    let external_identifier_slice = &self.bytes[self.pos..len];
                    let identifier_type = if external_identifier_slice.starts_with(b"SYSTEM") {
                        self.pos += 6;
                        ExternalIdentifier::System
                    } else if external_identifier_slice.starts_with(b"PUBLIC") {
                        self.pos += 6;
                        ExternalIdentifier::Public
                    } else {
                        return Some(Err(Error::UnknownExternalIdentifier));
                    };

                    self.advance_past_whitespaces();
                    if self.pos >= self.bytes.len() {
                        return None;
                    }

                    let identifier_literal = &self.bytes[self.pos..len];

                    // Do no consunme break character, as it
                    // need to be used during the next iteration
                    // to transition states.
                    self.pos = len;
                    Some(Ok(XmlToken::ExternalIdentifier {
                        identifier_type,
                        literal: identifier_literal,
                    }))
                } else if remaining.starts_with(b"[") {
                    self.pos += 1;
                    self.state = XmlState::InsideInternalSubset;
                    Some(Ok(XmlToken::InternalSubsetTagStart))
                } else {
                    unimplemented!("AfterDoctypeName")
                }
            }

            // FIXME: Refactor this state.
            // The entity declarations should be better
            // parsed so they can actually be used.
            XmlState::InsideInternalSubset => {
                self.advance_past_whitespaces();
                if self.pos >= self.bytes.len() {
                    return None;
                }

                let remaining = &self.bytes[self.pos..];

                if remaining.starts_with(b"<!--") {
                    self.pos += 4;
                    match self.consume_comment() {
                        Ok(comment) => Some(Ok(XmlToken::Comment(comment))),
                        Err(e) => Some(Err(e)),
                    }
                } else if remaining.starts_with(b"<!ENTITY") {
                    self.pos += 8;

                    self.advance_past_whitespaces();
                    if self.pos >= self.bytes.len() {
                        return None;
                    }

                    match self.consume_entity_declaration() {
                        Ok(ed) => {
                            self.pos += 1;
                            Some(Ok(XmlToken::EntityDeclaration(ed)))
                        }
                        Err(e) => Some(Err(e)),
                    }
                } else if remaining.starts_with(b"]") {
                    self.state = XmlState::AfterDoctypeName;
                    self.pos += 1;
                    Some(Ok(XmlToken::InternalSubsetTagEnd))
                } else {
                    unimplemented!("InsideInternalSubset")
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {

    use super::*;

    #[cfg(test)]
    impl<'a> XmlTokenizer<'a> {
        fn collect_all(mut self) -> Vec<Result<XmlToken<'a>, Error>> {
            self.by_ref().collect()
        }
    }

    #[test]
    fn test_html_declaration1() {
        let input = br#"<?xml version="1.0" encoding="UTF-8"?>"#;
        let tokens = XmlTokenizer::from(input.as_slice()).collect_all();
        assert_eq!(
            tokens,
            vec![
                Ok(XmlToken::Declaration(b"xml")),
                Ok(XmlToken::Attribute { name: b"version", value: b"1.0"}),
                Ok(XmlToken::Attribute { name: b"encoding", value: b"UTF-8"}),
                Ok(XmlToken::DeclarationTagEnd),

            ]
        );
    }
    
    #[test]
    fn test_doctype1() {
        let input = br#"<!DOCTYPE TESTSUITE SYSTEM "testcases.dtd">"#;
        let tokens = XmlTokenizer::from(input.as_slice()).collect_all();
        assert_eq!(
            tokens,
            vec![
                Ok(XmlToken::DocumentType(b"TESTSUITE")),
                Ok(XmlToken::ExternalIdentifier { identifier_type: ExternalIdentifier::System, literal: br#""testcases.dtd""# }),
                Ok(XmlToken::DocumentTypeTagEnd),
            ]
        );
    }

}

