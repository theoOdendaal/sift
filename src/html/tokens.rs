use std::borrow::Cow;

use crate::html::errors::Error;

#[derive(Debug, PartialEq)]
pub struct Doctype<'a> {
    name: Option<Cow<'a, str>>,
    public_identifier: Option<Cow<'a, str>>,
    system_identifier: Option<Cow<'a, str>>,
    force_quirks_flag: bool,
}

#[derive(Debug, PartialEq)]
pub struct Attribute<'a> {
    name: Option<Cow<'a, str>>,
    value: Option<Cow<'a, str>>,
}


#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    name: Option<Cow<'a, str>>,
    self_closing_tag: Option<bool>,
    attributes: Vec<Attribute<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum HtmlToken<'a> {
    Doctype(Doctype<'a>),

    StartTag(Tag<'a>),

    EndTag(Tag<'a>),

    Comment(Cow<'a, str>),

    Character(Cow<'a, str>),

    EndOfFile,

}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenizationState {
    Data,

    RcData,

    RawText,

    ScriptData,

    PlainText,

    TagOpen,

    EndTagOpen,

    TagName,

    RcDataLessThanSign,

    RcDataEndTagOpen,

    RcDataEndTagName,

    RawTextLessThanSign,

    RawTextEndTagOpen,

    RawTextEndTagName,

    ScriptDataLessThanSign,

    ScriptDataEndTagOpen,

    ScriptDataEndTagName,

    ScriptDataEscapeStart,

    ScriptDataEscapeStartDash,

    ScriptDataEscaped,

    ScriptDataEscapedDash,

    ScriptDataEscapedDashDash,

    ScriptDataEscapedLessThanSign,

    ScriptDataEscapedEndTagOpen,

    ScriptDataEscapedEndTagName,

    ScriptDataDoubleEscapeStart,

    ScriptDataDoubleEscaped,

    ScriptDataDoubleEscapedDash,

    ScriptDataDoubleEscapedDashDash,

    ScriptDataDoubleEscapedLessThanSign,

    ScriptDataDoubleEscapeEnd,

    BeforeAttributeName,

    AttributeName,

    AfterAttributeName,

    BeforeAttributeValue,

    AttributeValueDoubleQuoted,

    AttributeValueSingleQuoted,

    AttributeValueUnquoted,

    AfterAttributeValueQuoted,

    SelfClosingStartTag,

    BogusComment,

    MarkupDeclarationOpen,

    CommentStart,

    CommentStartDash,

    Comment,

    CommentLessThanSign,

    CommentLessThanSignBang,

    CommentLessThanSignBangDash,

    CommentLessThanSignBangDashDash,

    CommentEndDash,

    CommentEnd,

    CommentEndBang,

    Doctype,

    BeforeDoctypeName,

    DoctypeName,

    AfterDoctypeName,

    AfterDoctypePublicKeyword,

    BeforeDoctypePublicIdentifier,

    DoctypePublicIdentifierDoubleQuoted,

    DoctypePublicIdentifierSingleQuoted,

    AfterDoctypePublicIdentifier,

    BetweenDoctypePublicAndSystemIdentifiers,

    AfterDoctypeSystemKeyword,

    BeforeDoctypeSystemIdentifier,

    DoctypeSystemIdentifierDoubleQuoted,

    DoctypeSystemIdentifierSingleQuoted,

    AfterDoctypeSystemIdentifier,

    BogusDoctype,

    CdataSection,

    CdataSectionBracket,

    CdataSectionEnd,

    ProcessingInstructionOpen,

    ProcessingInstructionTarget,

    AfterProcessingInstructionTarget,

    ProcessingInstructionData,

    ProcessingInstructionQuestionable,

    CharacterReference,

    NamedCharacterReference,

    AmbiguousAmpersand,

    NumericCharacterReference,

    HexadecimalCharacterReferenceStart,

    HexadecimalCharacterReference,

    DecimalCharacterReference,

    NumericCharacterReferenceEnd,

}


pub struct HtmlTokenizer<'a> {
    input: &'a str,

    pos: usize,

    state: TokenizationState,

    return_state: TokenizationState,

    errors: Vec<Error>,

    mark: usize,

    current_tag_buffer: Option<Tag<'a>>,

    is_current_tag_end: bool,

    last_start_tag_name: Option<Cow<'a, str>>,

    //current_doctype: Option<Doctype<'a>>,

    //character_reference_code: u32,
}

impl<'a> HtmlTokenizer<'a> {

    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            state: TokenizationState::Data,
            return_state: TokenizationState::Data,
            errors: Vec::new(),
            mark: 0,
            current_tag_buffer: None,
            is_current_tag_end: false,
            last_start_tag_name: None,
            //current_doctype: None,
            //character_reference_code: 0,
        }
    }

    fn set_state(&mut self, state: TokenizationState) {
        self.state = state;
    }

    fn set_last_start_tag_name(&mut self, last_start_tag_name: &'a str) {
        self.last_start_tag_name = Some(Cow::Borrowed(last_start_tag_name));
    }
    
    #[inline]
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    #[inline]
    fn consume(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    #[inline]
    fn is_ascii_alpha(c: char) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase()
    }

    #[inline]
    fn is_appropriate_end_tag(&self) -> bool {
        if let Some(last_start) = &self.last_start_tag_name {
            let current_name = &self.input[self.mark..self.pos];
            return current_name.eq_ignore_ascii_case(last_start);
        }
        false
    }

    #[inline]
    fn to_lower_cow(slice: &'a str) -> Cow<'a, str> {
        if slice.bytes().any(|b| b.is_ascii_uppercase()) {
            Cow::Owned(slice.to_ascii_lowercase())
        } else {
            Cow::Borrowed(slice)
        }
    }

    pub fn next_token(&mut self) -> Option<HtmlToken<'a>> {
        loop {

            if self.pos >= self.input.len() && self.state == TokenizationState::Data {
                return Some(HtmlToken::EndOfFile);
            }


            match self.state {
                
                // https://html.spec.whatwg.org/#data-state
                TokenizationState::Data => {
                    self.mark = self.pos;
                    
                    // 'Anything else' logic. Will consume until characters
                    // until one is encountered that change the state.
                    while let Some(c) = self.peek() {
                        if c == '&' || c == '<' || c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                    }

                    match self.consume() {
                        Some('&') => {
                            self.return_state = TokenizationState::Data;
                            self.state = TokenizationState::CharacterReference;
                        },

                        Some('<') => {
                            self.state = TokenizationState::TagOpen;
                        },
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.pos - 1..self.pos])));
                        },
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => unreachable!()
                    }
                },

                // https://html.spec.whatwg.org/#rcdata-state
                TokenizationState::RcData => {
                    self.mark = self.pos;
                    
                    // 'Anything else' logic. Will consume until characters
                    // until one is encountered that change the state.
                    while let Some(c) = self.peek() {
                        if c == '&' || c == '<' || c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                    }

                    match self.consume() {
                        Some('&') => {
                            self.return_state = TokenizationState::RcData;
                            self.state = TokenizationState::CharacterReference;
                        },
                        Some('<') => {
                            self.state = TokenizationState::RcDataLessThanSign;
                        },
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                        },
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        },
                        _ => unreachable!() 

                    }
                },

                // https://html.spec.whatwg.org/#rawtext-state
                TokenizationState::RawText => {
                    self.mark = self.pos;
                    
                    // 'Anything else' logic. Will consume until characters
                    // until one is encountered that change the state.
                    while let Some(c) = self.peek() {
                        if c == '&' || c == '<' || c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                    }

                    match self.consume() {
                        Some('<') => {
                            self.state = TokenizationState::RawTextLessThanSign;
                        },
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                        },
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        },
                        _ => unreachable!() 

                    }
                },

                // https://html.spec.whatwg.org/#script-data-state
                TokenizationState::ScriptData => {
                    self.mark = self.pos;
                    
                    // 'Anything else' logic. Will consume until characters
                    // until one is encountered that change the state.
                    while let Some(c) = self.peek() {
                        if c == '&' || c == '<' || c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                    }

                    match self.consume() {
                        Some('<') => {
                            self.state = TokenizationState::ScriptDataLessThanSign;
                        },
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                        },
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        },
                        _ => unreachable!() 

                    }
                },
                
                // https://html.spec.whatwg.org/#plaintext-state
                TokenizationState::PlainText => {
                    self.mark = self.pos;

                    while let Some(c) = self.peek() {
                        if c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                    }

                    match self.consume() {
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                        },
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        },
                        _ => unreachable!()
                    }

                },
                
                //TokenizationState::TagOpen,
                //TokenizationState::EndTagOpen,
                //TokenizationState::TagName,

                // https://html.spec.whatwg.org/#rcdata-less-than-sign-state
                TokenizationState::RcDataLessThanSign => {
                    match self.peek() {
                        Some('/') => {
                            self.consume();
                            self.state = TokenizationState::RcDataEndTagOpen;
                        },
                        _ => {
                            self.state = TokenizationState::RcData;
                            return Some(HtmlToken::Character(Cow::Borrowed("<")));
                        }

                    }

                },

                // https://html.spec.whatwg.org/#rcdata-end-tag-open-state
                TokenizationState::RcDataEndTagOpen => {
                    match self.peek() {
                        Some(c)  if Self::is_ascii_alpha(c) => {
                            self.current_tag_buffer = Some(Tag { name: None, self_closing_tag: None, attributes: Vec::new() });
                            self.is_current_tag_end = true;
                            self.mark = self.pos;
                            self.state = TokenizationState::RcDataEndTagName;
                        },
                        _ => {
                            self.state = TokenizationState::RcData;
                            return Some(HtmlToken::Character(Cow::Borrowed("</")));
                        }
                    }

                    
                },

                // https://html.spec.whatwg.org/#rcdata-end-tag-name-state
                TokenizationState::RcDataEndTagName => {
                    
                    loop {
                   
                        match self.peek() {
                            Some(c) if Self::is_ascii_alpha(c) => {
                                // We'll distinguish between upper and lower when emitting.
                                self.consume();
                            },

                            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') if self.is_appropriate_end_tag() => {
                                self.consume(); 
                                self.state = TokenizationState::BeforeAttributeName;
                                break;
                            },
                            Some('/') if self.is_appropriate_end_tag() => {
                                self.consume();
                                self.state = TokenizationState::SelfClosingStartTag;
                                break;
                            },
                            Some('>') if self.is_appropriate_end_tag() => {
                                self.state = TokenizationState::Data;

                                if let Some(tag) = self.current_tag_buffer.as_mut() {
                                    if tag.name.is_none() {
                                        let name_slice = &self.input[self.mark..self.pos];
                                        tag.name = Some(Self::to_lower_cow(name_slice));
                                    }
                                }
                                // Onlu consime after extracting name to ensure '>'
                                // is not included as part of the name.
                                self.consume();
                                return Some(HtmlToken::EndTag(self.current_tag_buffer.take().unwrap()));
                            },
                            _ => {
                                self.current_tag_buffer = None;
                                self.state = TokenizationState::RcData;
                                // The only way to get to this state is by the preceding 2 chars
                                // being '</'.
                                let char_slice = &self.input[self.mark-2..self.pos];
                                return Some(HtmlToken::Character(Cow::Borrowed(char_slice)));
                                 
                            },

                        }
                    }

                    

                },

                // https://html.spec.whatwg.org/#rawtext-less-than-sign-state
                TokenizationState::RawTextLessThanSign => {
                    match self.peek() {
                        Some('/') => {
                            self.consume();
                            self.state = TokenizationState::RawTextEndTagOpen;
                        },
                        _ => {
                            self.state = TokenizationState::RawText;
                            return Some(HtmlToken::Character(Cow::Borrowed("<")));
                        }

                    }

                },

                TokenizationState::RawTextEndTagOpen => {
                    match self.peek() {
                        Some(c)  if Self::is_ascii_alpha(c) => {
                            self.current_tag_buffer = Some(Tag { name: None, self_closing_tag: None, attributes: Vec::new() });
                            self.is_current_tag_end = true;
                            self.mark = self.pos;
                            self.state = TokenizationState::RawTextEndTagName;
                        },
                        _ => {
                            self.state = TokenizationState::RawText;
                            return Some(HtmlToken::Character(Cow::Borrowed("</")));
                        }
                    }
                },

                // https://html.spec.whatwg.org/#rawtext-end-tag-name-state
                TokenizationState::RawTextEndTagName => {
                    loop {
                   
                        match self.peek() {
                            Some(c) if Self::is_ascii_alpha(c) => {
                                // We'll distinguish between upper and lower when emitting.
                                self.consume();
                            },

                            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') if self.is_appropriate_end_tag() => {
                                self.consume(); 
                                self.state = TokenizationState::BeforeAttributeName;
                                break;
                            },
                            Some('/') if self.is_appropriate_end_tag() => {
                                self.consume();
                                self.state = TokenizationState::SelfClosingStartTag;
                                break;
                            },
                            Some('>') if self.is_appropriate_end_tag() => {
                                self.state = TokenizationState::Data;

                                if let Some(tag) = self.current_tag_buffer.as_mut() {
                                    if tag.name.is_none() {
                                        let name_slice = &self.input[self.mark..self.pos];
                                        tag.name = Some(Self::to_lower_cow(name_slice));
                                    }
                                }
                                // Onlu consime after extracting name to ensure '>'
                                // is not included as part of the name.
                                self.consume();
                                return Some(HtmlToken::EndTag(self.current_tag_buffer.take().unwrap()));
                            },
                            _ => {
                                self.current_tag_buffer = None;
                                self.state = TokenizationState::RawText;
                                // The only way to get to this state is by the preceding 2 chars
                                // being '</'.
                                let char_slice = &self.input[self.mark-2..self.pos];
                                return Some(HtmlToken::Character(Cow::Borrowed(char_slice)));
                                 
                            },

                        }
                    }

                },

                //TokenizationState::ScriptDataLessThanSign,
                //TokenizationState::ScriptDataEndTagOpen,
                //TokenizationState::ScriptDataEndTagName,
                //TokenizationState::ScriptDataEscapeStart,
                //TokenizationState::ScriptDataEscapeStartDash,
                //TokenizationState::ScriptDataEscaped,
                //TokenizationState::ScriptDataEscapedDash,
                //TokenizationState::ScriptDataEscapedDashDash,
                //TokenizationState::ScriptDataEscapedLessThanSign,
                //TokenizationState::ScriptDataEscapedEndTagOpen,
                //TokenizationState::ScriptDataEscapedEndTagName,
                //TokenizationState::ScriptDataDoubleEscapeStart,
                //TokenizationState::ScriptDataDoubleEscaped,
                //TokenizationState::ScriptDataDoubleEscapedDash,
                //TokenizationState::ScriptDataDoubleEscapedDashDash,
                //TokenizationState::ScriptDataDoubleEscapedLessThanSign,
                //TokenizationState::ScriptDataDoubleEscapeEnd,
                //TokenizationState::BeforeAttributeName,
                //TokenizationState::AttributeName,
                //TokenizationState::AfterAttributeName,
                //TokenizationState::BeforeAttributeValue,
                //TokenizationState::AttributeValueDoubleQuoted,
                //TokenizationState::AttributeValueSingleQuoted,
                //TokenizationState::AttributeValueUnquoted,
                //TokenizationState::AfterAttributeValueQuoted,
                //TokenizationState::SelfClosingStartTag,
                //TokenizationState::BogusComment,
                //TokenizationState::MarkupDeclarationOpen,
                //TokenizationState::CommentStart,
                //TokenizationState::CommentStartDash,
                //TokenizationState::Comment,
                //TokenizationState::CommentLessThanSign,
                //TokenizationState::CommentLessThanSignBang,
                //TokenizationState::CommentLessThanSignBangDash,
                //TokenizationState::CommentLessThanSignBangDashDash,
                //TokenizationState::CommentEndDash,
                //TokenizationState::CommentEnd,
                //TokenizationState::CommentEndBang,
                //TokenizationState::Doctype,
                //TokenizationState::BeforeDoctypeName,
                //TokenizationState::DoctypeName,
                //TokenizationState::AfterDoctypeName,
                //TokenizationState::AfterDoctypePublicKeyword,
                //TokenizationState::BeforeDoctypePublicIdentifier,
                //TokenizationState::DoctypePublicIdentifierDoubleQuoted,
                //TokenizationState::DoctypePublicIdentifierSingleQuoted,
                //TokenizationState::AfterDoctypePublicIdentifier,
                //TokenizationState::BetweenDoctypePublicAndSystemIdentifiers,
                //TokenizationState::AfterDoctypeSystemKeyword,
                //TokenizationState::BeforeDoctypeSystemIdentifier,
                //TokenizationState::DoctypeSystemIdentifierDoubleQuoted,
                //TokenizationState::DoctypeSystemIdentifierSingleQuoted,
                //TokenizationState::AfterDoctypeSystemIdentifier,
                //TokenizationState::BogusDoctype,
                //TokenizationState::CdataSection,
                //TokenizationState::CdataSectionBracket,
                //TokenizationState::CdataSectionEnd,
                //TokenizationState::ProcessingInstructionOpen,
                //TokenizationState::ProcessingInstructionTarget,
                //TokenizationState::AfterProcessingInstructionTarget,
                //TokenizationState::ProcessingInstructionData,
                //TokenizationState::ProcessingInstructionQuestionable,
                //TokenizationState::CharacterReference,
                //TokenizationState::NamedCharacterReference,
                //TokenizationState::AmbiguousAmpersand,
                //TokenizationState::NumericCharacterReference,
                //TokenizationState::HexadecimalCharacterReferenceStart,
                //TokenizationState::HexadecimalCharacterReference,
                //TokenizationState::DecimalCharacterReference,
                //TokenizationState::NumericCharacterReferenceEnd,
                _ => unimplemented!()

            }
        }

    }
}




#[cfg(test)]
mod tests {
    use super::*;
    
    /// https://github.com/html5lib/html5lib-tests/blob/master/tokenizer/contentModelFlags.test
    #[test]
    fn test_plaintext_content_model_flag() {
        let initial_state = TokenizationState::PlainText;
        let last_start_tag = "plaintext";
        let input = "<head>&body;";
        let output = &[HtmlToken::Character(Cow::Borrowed("<head>&body;"))];
    
        let mut tokenizer = HtmlTokenizer::new(input);
        tokenizer.set_state(initial_state);
        tokenizer.set_last_start_tag_name(last_start_tag);

        let mut tokens = Vec::new();
        while let Some(token) = tokenizer.next_token() {
            if token == HtmlToken::EndOfFile {
                break
            } 
            tokens.push(token);
        }

        assert_eq!(output, tokens.as_slice());
    }

    /// https://github.com/html5lib/html5lib-tests/blob/master/tokenizer/contentModelFlags.test
    #[test]
    fn test_plaintext_with_seeming_close_tag() {
        let initial_state = TokenizationState::PlainText;
        let last_start_tag = "plaintext";
        let input = "</plaintext>&body";
        let output = &[HtmlToken::Character(Cow::Borrowed("</plaintext>&body"))];
    
        let mut tokenizer = HtmlTokenizer::new(input);
        tokenizer.set_state(initial_state);
        tokenizer.set_last_start_tag_name(last_start_tag);

        let mut tokens = Vec::new();
        while let Some(token) = tokenizer.next_token() {
            if token == HtmlToken::EndOfFile {
                break
            } 
            tokens.push(token);
        }

        assert_eq!(output, tokens.as_slice());
    }

    /// https://github.com/html5lib/html5lib-tests/blob/master/tokenizer/contentModelFlags.test
    #[test]
    fn test_end_tag_closing_rcdata() {
        let initial_state = TokenizationState::RcData;
        let last_start_tag = "xmp";
        let input = "foo</xmp>";
        let output = &[HtmlToken::Character(Cow::Borrowed("foo")), HtmlToken::EndTag(Tag { name: Some(Cow::Borrowed("xmp")), self_closing_tag: None, attributes: vec![] })];
    
        let mut tokenizer = HtmlTokenizer::new(input);
        tokenizer.set_state(initial_state);
        tokenizer.set_last_start_tag_name(last_start_tag);

        let mut tokens = Vec::new();
        while let Some(token) = tokenizer.next_token() {
            if token == HtmlToken::EndOfFile {
                break
            } 
            tokens.push(token);
        }

        assert_eq!(output, tokens.as_slice());
    }
    
    
    /// https://github.com/html5lib/html5lib-tests/blob/master/tokenizer/contentModelFlags.test
    #[test]
    fn test_end_tag_closing_rawtext() {
        let initial_state = TokenizationState::RawText;
        let last_start_tag = "xmp";
        let input = "foo</xmp>";
        let output = &[HtmlToken::Character(Cow::Borrowed("foo")), HtmlToken::EndTag(Tag { name: Some(Cow::Borrowed("xmp")), self_closing_tag: None, attributes: vec![] })];
    
        let mut tokenizer = HtmlTokenizer::new(input);
        tokenizer.set_state(initial_state);
        tokenizer.set_last_start_tag_name(last_start_tag);

        let mut tokens = Vec::new();
        while let Some(token) = tokenizer.next_token() {
            if token == HtmlToken::EndOfFile {
                break
            } 
            tokens.push(token);
        }

        assert_eq!(output, tokens.as_slice());
    }

}

