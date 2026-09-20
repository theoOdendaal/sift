// https://www.w3.org/TR/xml/

// FIXME: How strict should I make my xml parser?
// Ex. for STag, should I allow '<' S Name, even thought
// its actually not allowed?


use super::errors::{Error, ErrorKind};
use crate::core::scanner::Scanner;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum ExternalIdentifier {
    System,
    Public,
}

impl std::fmt::Display for ExternalIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "SYSTEM"),
            Self::Public => write!(f, "PUBLIC"),
        }
    }
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
    
    EntityDeclaration {
        name: &'a [u8],
        identifier: ExternalIdentifier,
        literal: &'a [u8]
    },

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



impl<'a> std::fmt::Display for XmlToken<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declaration(b) => write!(
                f,
                "Declaration({})",
                String::from_utf8_lossy(b)
            ),

            Self::DeclarationTagEnd => f.write_str("DeclarationTagEnd"),

            Self::ProcessingInstruction { target, data } => write!(
                f,
                "ProcessingInstruction(target={}, data={})",
                String::from_utf8_lossy(target),
                String::from_utf8_lossy(data)
            ),

            Self::DocumentType(b) => write!(
                f,
                "DocumentType({})",
                String::from_utf8_lossy(b)
            ),

            Self::ExternalIdentifier {
                identifier_type,
                literal,
            } => write!(
                f,
                "ExternalIdentifier(type={}, literal={})",
                identifier_type,
                String::from_utf8_lossy(literal)
            ),

            Self::InternalSubsetTagStart => f.write_str("InternalSubsetTagStart"),

            Self::EntityDeclaration { name, identifier, literal } => write!(
                f,
                "EntityDeclaration(name={}, identifier={}, literal={})",
                String::from_utf8_lossy(name),
                identifier,
                String::from_utf8_lossy(literal)
            ),

            Self::InternalSubsetTagEnd => f.write_str("InternalSubsetTagEnd"),

            Self::DocumentTypeTagEnd => f.write_str("DocumentTypeTagEnd"),

            Self::StartTag(b) => write!(
                f,
                "StartTag({})",
                String::from_utf8_lossy(b)
            ),

            Self::Attribute { name, value } => write!(
                f,
                "Attribute(name={}, value={})",
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
    scanner: Scanner<'a>,
    state: XmlState,
}

enum XmlState {
    Normal,
    InsideXmlDeclaration,
    AfterStartTagName,
    AfterDoctypeName,
    InsideInternalSubset,
}

impl<'a> From<&'a [u8]> for XmlTokenizer<'a> {
    fn from(value: &'a [u8]) -> Self {
        let scanner = Scanner::from(value);
        Self {
            scanner,
            state: XmlState::Normal,
        }
    }
}

impl<'a> XmlTokenizer<'a> {

    #[inline]
    fn error(&self, kind: ErrorKind) -> Error {
        Error { kind, pos: self.scanner.pos }
    }

    #[inline]
    fn consume_attribute(&mut self) -> Result<XmlToken<'a>, Error> {
        let attribute_name = self.scanner.consume_attribute_name()
            .ok_or_else(|| self.error(ErrorKind::UnterminatedAttributeName))?;

        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Err(self.error(ErrorKind::UnexpectedEndOfFile));
        }
        
        if !self.scanner.consume_byte_if(b'=') {
            return Err(self.error(ErrorKind::UnexpectedAttributeFormat));
        }

        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Err(self.error(ErrorKind::UnexpectedEndOfFile));
        }
        
        let quote_char = self.scanner.get_byte();
        if quote_char != b'\'' && quote_char != b'"' {
            return Err(self.error(ErrorKind::UnexpectedAttributeFormat))
        }
        // Consume start quote byte.
        self.scanner.consume_byte();

        let attribute_value = self.scanner.consume_quoted_attribute_value(quote_char)
            .ok_or_else(|| self.error(ErrorKind::UnterminatedAttributeValue))?;

        // Consume end quote byte.
        self.scanner.consume_byte();

        Ok(XmlToken::Attribute { name: attribute_name, value: attribute_value })
    }

    fn next_normal(&mut self) -> Option<Result<XmlToken<'a>, Error>> {
        if !self.scanner.is_byte(b'<') {
            return Some(Ok(XmlToken::Text(self.scanner.consume_text())));
        }
        
        if self.scanner.get_checked_nth_byte(1) == Some(&b'!') {
            
            if self.scanner.starts_with(b"<!--") {
                self.scanner.consume_n_bytes(4);

                match self.scanner.consume_comment() {
                    Some(comment) => return Some(Ok(XmlToken::Comment(comment))),
                    None => return Some(Err(self.error(ErrorKind::UnterminatedComment))),
                }
            }

            if self.scanner.starts_with(b"<![CDATA[") {
                self.scanner.consume_n_bytes(9);

                match self.scanner.consume_cdata() {
                    Some(comment) => return Some(Ok(XmlToken::CharacterData(comment))),
                    None => return Some(Err(self.error(ErrorKind::UnterminatedCharacterData))),
                }
            }

            if self.scanner.starts_with(b"<!DOCTYPE") {
                self.scanner.consume_n_bytes(9);

                self.scanner.advance_past_whitespaces();
                if self.scanner.is_eof() {
                    return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
                }

                match self.scanner.consume_tag_name() {
                    Some(name) => {
                        self.state = XmlState::AfterDoctypeName;
                        return Some(Ok(XmlToken::DocumentType(name)))
                    },
                    None => return  Some(Err(self.error(ErrorKind::UnterminatedDocumentType)))
                }
            }
        }


        if self.scanner.starts_with(b"<?xml ") {
            self.scanner.consume_n_bytes(5);
            self.state = XmlState::InsideXmlDeclaration;
            return Some(Ok(XmlToken::Declaration(b"xml")));
        }

        if self.scanner.starts_with(b"<?") {
            self.scanner.consume_n_bytes(2);
                
            self.scanner.advance_past_whitespaces();
            if self.scanner.is_eof() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }

            let target = match self.scanner.consume_tag_name() {
                Some(value) => value,
                None => return Some(Err(self.error(ErrorKind::UnterminatedDeclaration))),
            };

            self.scanner.advance_past_whitespaces();
            if self.scanner.is_eof() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }

            let data = match self.scanner.consume_until(|b| b == b'?') {
                Some(d) => d,
                None => return Some(Err(self.error(ErrorKind::UnterminatedDeclaration))),
            };

            if !self.scanner.starts_with(b"?>") {
                return Some(Err(self.error(ErrorKind::UnterminatedDeclaration)));
            }
            self.scanner.consume_n_bytes(2);

            return Some(Ok(XmlToken::ProcessingInstruction { target, data }));
        }

        if self.scanner.starts_with(b"</") {
            self.scanner.consume_n_bytes(2);

            let tag_name = match self.scanner.consume_tag_name() {
                Some(value) => value,
                None => return Some(Err(self.error(ErrorKind::UnterminatedTagName))),
            };

            // FIXME: For now I'm just going to lazily
            // skip all bytes until tag end is found.
            // I need to add some validation here, as attributes
            // are not allowed in a ETag.
            if self.scanner.consume_until(|b| b == b'>').is_none() {
                return Some(Err(self.error(ErrorKind::UnterminatedTag)));
            }

            self.scanner.consume_byte();
            self.state = XmlState::Normal;

            return Some(Ok(XmlToken::EndTag(tag_name)));
        }
        
        self.scanner.consume_byte();
        let tag_name = match self.scanner.consume_tag_name() {
            Some(name) => name,
            None => return Some(Err(self.error(ErrorKind::UnterminatedTagName))),
        };
        self.state = XmlState::AfterStartTagName;
        Some(Ok(XmlToken::StartTag(tag_name)))

    }

    fn next_inside_xml_declaration(&mut self) -> Option<Result<XmlToken<'a>, Error>> {
        if self.scanner.starts_with(b"?>") {
            self.scanner.consume_n_bytes(2);
            self.state = XmlState::Normal;
            return Some(Ok(XmlToken::DeclarationTagEnd));
        }
        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
        }
        Some(self.consume_attribute())
    }

    fn next_after_start_tag_name(&mut self) -> Option<Result<XmlToken<'a>, Error>> {
        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
        }

        if self.scanner.starts_with(b"/>") {
            self.scanner.consume_n_bytes(2);
            self.state = XmlState::Normal;

            Some(Ok(XmlToken::TagEnd { self_closing: true }))
        } else if self.scanner.is_byte(b'>') {
            self.scanner.consume_byte();
            self.state = XmlState::Normal;
            Some(Ok(XmlToken::TagEnd { self_closing: false }))

        } else {
            Some(self.consume_attribute())
        }
    }

    fn next_after_doctype_name(&mut self) -> Option<Result<XmlToken<'a>, Error>> {
        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
        }
        
        if self.scanner.starts_with(b">") {
            self.scanner.consume_byte();
            self.state = XmlState::Normal;
            Some(Ok(XmlToken::DocumentTypeTagEnd))

        } else if self.scanner.starts_with(b"SYSTEM") {
            self.scanner.consume_n_bytes(6);
            let identifier_type = ExternalIdentifier::System;

            self.scanner.advance_past_whitespaces();
            if self.scanner.is_eof() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }

            if let Some(value) = self.scanner.consume_until(|b| matches!(b, b'>' | b'[')) {
                return Some(Ok(XmlToken::ExternalIdentifier { identifier_type, literal: value }));
            } 
            Some(Err(self.error(ErrorKind::UnterminatedExternalIdentifier)))

        } else if self.scanner.starts_with(b"PUBLIC") {
            self.scanner.consume_n_bytes(6);
            let identifier_type = ExternalIdentifier::Public;

            self.scanner.advance_past_whitespaces();
            if self.scanner.is_eof() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }

            if let Some(value) = self.scanner.consume_until(|b| matches!(b, b'>' | b'[')) {
                return Some(Ok(XmlToken::ExternalIdentifier { identifier_type, literal: value }));
            } 
            Some(Err(self.error(ErrorKind::UnterminatedExternalIdentifier)))

        } else if self.scanner.is_byte(b'[') {
            self.scanner.consume_byte();
            self.state = XmlState::InsideInternalSubset;
            Some(Ok(XmlToken::InternalSubsetTagStart))

        } else {
            unimplemented!("AfterDocTypeName")
        }
    }

    fn next_inside_internal_subset(&mut self) -> Option<Result<XmlToken<'a>, Error>> {
        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
        }

        if self.scanner.starts_with(b"<!--") {
            self.scanner.consume_n_bytes(4);
            match self.scanner.consume_comment() {
                Some(value) => Some(Ok(XmlToken::Comment(value))),
                None => Some(Err(self.error(ErrorKind::UnterminatedComment)))
            }

        } else if self.scanner.starts_with(b"<!ENTITY") {
            self.scanner.consume_n_bytes(8);

            self.scanner.advance_past_whitespaces();
            if self.scanner.is_eof() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }

            let name = match self.scanner.consume_tag_name() {
                Some(name) => name,
                None => return Some(Err(self.error(ErrorKind::UnterminatedEntityDeclaration))),
            };

            self.scanner.advance_past_whitespaces();
            if self.scanner.is_eof() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }

            let identifier = if self.scanner.starts_with(b"SYSTEM") {
                self.scanner.consume_n_bytes(6);
                ExternalIdentifier::System
            } else if self.scanner.starts_with(b"PUBLIC") {
                self.scanner.consume_n_bytes(6);
                ExternalIdentifier::Public
            } else {
                return Some(Err(self.error(ErrorKind::UnknownExternalIdentifier)));
            };

            self.scanner.advance_past_whitespaces();
            if self.scanner.is_eof() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }

            if let Some(literal) = self.scanner.consume_until(|b| b == b'>') {
                self.scanner.consume_byte();
                return Some(Ok(XmlToken::EntityDeclaration { name, identifier, literal })) 

            }
            Some(Err(self.error(ErrorKind::UnknownExternalIdentifier)))

        } else if self.scanner.is_byte(b']') {
            self.state = XmlState::AfterDoctypeName;
            self.scanner.consume_byte();
            Some(Ok(XmlToken::InternalSubsetTagEnd))

        } else {
            unimplemented!("InsideInternalSubset")
        }
    }

}

impl<'a> Iterator for XmlTokenizer<'a> {
    type Item = Result<XmlToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.scanner.is_eof() {
            return None;
        }

        match self.state {
            XmlState::Normal => self.next_normal(),
            XmlState::InsideXmlDeclaration => self.next_inside_xml_declaration(),
            XmlState::AfterStartTagName => self.next_after_start_tag_name(),
            XmlState::AfterDoctypeName => self.next_after_doctype_name(),
            XmlState::InsideInternalSubset => self.next_inside_internal_subset(),
        }
    }
}

#[cfg(test)]
mod tests {

use super::*;

    #[cfg(test)]
    impl<'a> XmlTokenizer<'a> {
        pub fn collect_all(mut self) -> Vec<Result<XmlToken<'a>, Error>> {
            self.by_ref().collect()
        }
    }
    
    #[test]
    fn test_xml_declaration1() {
        let input = br#"<?xml version="1.0" encoding="UTF-8"?>"#;
        let tokenizer = XmlTokenizer::from(input.as_slice());
        let tokens = tokenizer.collect_all();
        assert_eq!(
            tokens,
            vec![
                Ok(XmlToken::Declaration(b"xml")),
                Ok(XmlToken::Attribute { name: b"version", value: b"1.0" }),
                Ok(XmlToken::Attribute { name: b"encoding", value: b"UTF-8" }),
                Ok(XmlToken::DeclarationTagEnd),
            ]
        )
    }

    #[test]
    fn test_tag1() {
        let input = br#"<TESTSUITE PROFILE="XML 1.0 (2nd edition) W3C Conformance Test Suite, 6 October 2000">"#;
        let tokenizer = XmlTokenizer::from(input.as_slice());
        let tokens = tokenizer.collect_all();
        assert_eq!(
            tokens,
            vec![
                Ok(XmlToken::StartTag(b"TESTSUITE")),
                Ok(XmlToken::Attribute { name: b"PROFILE", value: b"XML 1.0 (2nd edition) W3C Conformance Test Suite, 6 October 2000" }),
                Ok(XmlToken::TagEnd { self_closing: false }),
            ]
        )
    }

} 

