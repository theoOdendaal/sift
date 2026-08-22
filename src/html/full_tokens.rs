// https://html.spec.whatwg.org/#tokenization

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

    current_tag: Option<Tag<'a>>,

    is_current_tag_end: bool,

    last_start_tag_name: Option<Cow<'a, str>>,

    current_doctype: Option<Doctype<'a>>,

    character_reference_code: u32,
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
            current_tag: None,
            is_current_tag_end: false,
            last_start_tag_name: None,
            current_doctype: None,
            character_reference_code: 0,
        }
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
    fn to_lower_cow(slice: &'a str) -> Cow<'a, str> {
        if slice.bytes().any(|b| b.is_ascii_uppercase()) {
            Cow::Owned(slice.to_ascii_lowercase())
        } else {
            Cow::Borrowed(slice)
        }
    }

    #[inline]
    fn is_appropriate_end_tag(&self) -> bool {
        if let Some(last_start) = &self.last_start_tag_name {
            let current_name = &self.input[self.mark..self.pos];
            return current_name.eq_ignore_ascii_case(last_start);
        }
        false
    }

    pub fn next_token(&mut self) -> Option<HtmlToken<'a>> {
        loop {

            if self.pos >= self.input.len() && self.state == TokenizationState::Data {
                return Some(HtmlToken::EndOfFile);
            }


            match self.state {
    
                TokenizationState::Data => {
                    self.mark = self.pos;

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

                TokenizationState::RcData => {
                    self.mark = self.pos;

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

                TokenizationState::RawText => {
                    self.mark = self.pos;

                    while let Some(c) = self.peek() {
                        if c == '<' || c == '\0' {
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

                TokenizationState::ScriptData => {
                    self.mark = self.pos;

                    while let Some(c) = self.peek() {
                        if c == '<' || c == '\0' {
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


                TokenizationState::TagOpen => {
                    match self.peek() {
                        Some('!') => {
                            self.consume();
                            self.state = TokenizationState::MarkupDeclarationOpen;
                        },
                        Some('/') => {
                            self.consume();
                            self.state = TokenizationState::EndTagOpen;
                        },
                        Some(c) if Self::is_ascii_alpha(c) => {
                            self.current_tag = Some(Tag { name: None, self_closing_tag: None, attributes: Vec::new() });
                            self.state = TokenizationState::TagName;
                        },
                        Some('?') => {
                            self.consume();
                            self.state = TokenizationState::ProcessingInstructionOpen;
                        },
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        },
                        Some(_) => {
                            self.errors.push(Error::InvalidFirstCharacterOfTagName);
                            self.state = TokenizationState::Data;
                            return Some(HtmlToken::Character(Cow::Borrowed("<")));

                        }
                        
                    }

                },

                TokenizationState::EndTagOpen => {
                    match self.peek() {
                        Some(c) if Self::is_ascii_alpha(c) => {
                            self.current_tag = Some(Tag { name: None, self_closing_tag: None, attributes: Vec::new() });
                            self.state = TokenizationState::TagName;
                        },
                        Some('>') => {
                            self.consume();
                            self.errors.push(Error::MissingEndTagName);
                            self.state = TokenizationState::Data;
                        },
                        None => {
                            self.errors.push(Error::EofBeforeTagName);
                            return Some(HtmlToken::Character(Cow::Borrowed("</")));
                        },
                        Some(_) => {
                            self.errors.push(Error::InvalidFirstCharacterOfTagName);
                            self.state = TokenizationState::BogusComment;
                        }
                    }

                },

                TokenizationState::TagName => {

                    let mut tag_complete = false;

                    while let Some(c) = self.peek() {
                        match c {
                            '\t' | '\n' | '\x0C' | ' ' => {
                                self.consume();
                                self.state = TokenizationState::BeforeAttributeName;
                                break;
                            },
                            '/' => {
                                self.consume();
                                self.state = TokenizationState::SelfClosingStartTag;
                                break;
                            },
                            '>' => {
                                self.consume();
                                self.state = TokenizationState::Data;
                                tag_complete = true;
                                break;
                            },
                            '\0' => {
                                self.errors.push(Error::UnexpectedNullCharacter);
                                self.consume();
                            },
                            _ => {
                                self.consume();
                            }
                        }
                    }

                    if self.pos >= self.input.len() && !tag_complete && self.state == TokenizationState::TagName {
                            self.errors.push(Error::EofInTag);
                            return Some(HtmlToken::EndOfFile);
                    }

                    if let Some(tag) = self.current_tag.as_mut() {
                        if tag.name.is_none() {
                            let mut name_slice = &self.input[self.mark..self.pos];
                            name_slice = name_slice.trim_end_matches(&['\t', '\n', '\x0C', ' ', '/', '>'][..]);

                            tag.name = Some(Self::to_lower_cow(name_slice));
                        }
                    }

                    if tag_complete && let Some(tag) = self.current_tag.take() {
                        if !self.is_current_tag_end {
                            self.last_start_tag_name = tag.name.clone();
                            return Some(HtmlToken::StartTag(tag));
                        } else {
                            return Some(HtmlToken::EndTag(tag));
                        }

                    }

                },

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

                TokenizationState::RcDataEndTagOpen => {
                    match self.peek() {
                        Some(c) if Self::is_ascii_alpha(c) => {
                            self.current_tag = Some(Tag { name: None, self_closing_tag: None, attributes: Vec::new() });
                            self.is_current_tag_end = true;
                            
                            // Mark acts as start of the temporary buffer.
                            self.mark = self.pos;

                            self.state = TokenizationState::RcDataEndTagName;
                        },
                        _ => {
                            self.state = TokenizationState::RcData;
                            return Some(HtmlToken::Character(Cow::Borrowed("</")));
                        }
                    }
                },


                TokenizationState::RcDataEndTagName => {
                    let mut is_appropriate = false;
                    let mut tag_complete = false;
                    let mut switch_state = None;

                    while let Some(c) = self.peek() {
                        match c {
                            '\t' | '\n' | '\x0C' | ' ' => {
                                is_appropriate = self.is_appropriate_end_tag();
                                if is_appropriate {
                                    self.consume();
                                    switch_state = Some(TokenizationState::BeforeAttributeName);
                                }
                                break;
                            },
                            '/' => {
                                is_appropriate = self.is_appropriate_end_tag();
                                if is_appropriate {
                                    self.consume();
                                    switch_state = Some(TokenizationState::SelfClosingStartTag);
                                }
                                break;
                            },
                            '>' => {
                                is_appropriate = self.is_appropriate_end_tag();
                                if is_appropriate {
                                    self.consume();
                                    switch_state = Some(TokenizationState::Data);
                                    tag_complete = true;
                                }
                                break;
                            },
                            c if Self::is_ascii_alpha(c) => {
                                self.consume();
                            }
                            _ => {
                                break;
                            }



                        }

                    }
                    if is_appropriate {
                        self.state = switch_state.unwrap();

                        if let Some(tag) = self.current_tag.as_mut() {
                            if tag.name.is_none() {
                                let mut name_slice = &self.input[self.mark..self.pos];
                                name_slice = name_slice.trim_end_matches(&['\t', '\n', '\x0C', ' ', '/', '>'][..]);
                                tag.name = Some(Self::to_lower_cow(name_slice));
                            }
                        }

                        if tag_complete {
                            return Some(HtmlToken::EndTag(self.current_tag.take().unwrap()));
                        }
                    } else {
                        // "Anything else" failback.
                        let emit_slice_start = self.mark-2;
                        let buffered_slice = &self.input[emit_slice_start..self.pos];

                        self.state = TokenizationState::RcData;
                        self.current_tag = None;

                        return Some(HtmlToken::Character(Cow::Borrowed(buffered_slice)));
                    }
                },

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
                    Some(c) if Self::is_ascii_alpha(c) => {
                        self.current_tag = Some(Tag {
                            name: None,
                            self_closing_tag: Some(false),
                            attributes: Vec::new(),
                        });
                        self.is_current_tag_end = true;
                        self.mark = self.pos;
                        self.state = TokenizationState::RawTextEndTagName;
                    }
                    _ => {
                        self.state = TokenizationState::RawText;
                        return Some(HtmlToken::Character(Cow::Borrowed("</")));
                    }
                }
            },

            TokenizationState::RawTextEndTagName => {
                let mut is_appropriate = false;
                let mut tag_completed = false;
                let mut switch_state = None;

                while let Some(c) = self.peek() {
                    match c {
                        '\t' | '\n' | '\x0C' | ' ' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::BeforeAttributeName);
                            }
                            break;
                        }
                        '/' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::SelfClosingStartTag);
                            }
                            break;
                        }
                        '>' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::Data);
                                tag_completed = true;
                            }
                            break;
                        }
                        c if Self::is_ascii_alpha(c) => {
                            self.consume();
                        }
                        _ => break,
                    }
                }

                if is_appropriate {
                    self.state = switch_state.unwrap();
                    
                    if let Some(tag) = self.current_tag.as_mut() {
                        if tag.name.is_none() {
                            let mut name_slice = &self.input[self.mark..self.pos];
                            name_slice = name_slice.trim_end_matches(&['\t', '\n', '\x0C', ' ', '/', '>'][..]);
                            tag.name = Some(Self::to_lower_cow(name_slice));
                        }
                    }

                    if tag_completed {
                        return Some(HtmlToken::EndTag(self.current_tag.take().unwrap()));
                    }
                } else {
                    let emit_slice_start = self.mark - 2; 
                    let buffered_slice = &self.input[emit_slice_start..self.pos];
                    
                    self.state = TokenizationState::RawText;
                    self.current_tag = None;
                    return Some(HtmlToken::Character(Cow::Borrowed(buffered_slice)));
                }
            },

            TokenizationState::ScriptDataLessThanSign => {
                match self.peek() {
                    Some('/') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEndTagOpen;
                    }
                    Some('!') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapeStart;
                        return Some(HtmlToken::Character(Cow::Borrowed("<!")));
                    }
                    _ => {
                        self.state = TokenizationState::ScriptData;
                        return Some(HtmlToken::Character(Cow::Borrowed("<")));
                    }
                }
            },

            TokenizationState::ScriptDataEndTagOpen => {
                match self.peek() {
                    Some(c) if Self::is_ascii_alpha(c) => {
                        self.current_tag = Some(Tag {
                            name: None,
                            self_closing_tag: Some(false),
                            attributes: Vec::new(),
                        });
                        self.is_current_tag_end = true;
                        self.mark = self.pos;
                        self.state = TokenizationState::ScriptDataEndTagName;
                    }
                    _ => {
                        self.state = TokenizationState::ScriptData;
                        return Some(HtmlToken::Character(Cow::Borrowed("</")));
                    }
                }
            },

            TokenizationState::ScriptDataEndTagName => {
                let mut is_appropriate = false;
                let mut tag_completed = false;
                let mut switch_state = None;

                while let Some(c) = self.peek() {
                    match c {
                        '\t' | '\n' | '\x0C' | ' ' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::BeforeAttributeName);
                            }
                            break;
                        }
                        '/' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::SelfClosingStartTag);
                            }
                            break;
                        }
                        '>' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::Data);
                                tag_completed = true;
                            }
                            break;
                        }
                        c if Self::is_ascii_alpha(c) => {
                            self.consume();
                        }
                        _ => break,
                    }
                }

                if is_appropriate {
                    self.state = switch_state.unwrap();
                    
                    if let Some(tag) = self.current_tag.as_mut() {
                        if tag.name.is_none() {
                            let mut name_slice = &self.input[self.mark..self.pos];
                            name_slice = name_slice.trim_end_matches(&['\t', '\n', '\x0C', ' ', '/', '>'][..]);
                            tag.name = Some(Self::to_lower_cow(name_slice));
                        }
                    }

                    if tag_completed {
                        return Some(HtmlToken::EndTag(self.current_tag.take().unwrap()));
                    }
                } else {
                    let emit_slice_start = self.mark - 2; 
                    let buffered_slice = &self.input[emit_slice_start..self.pos];
                    
                    self.state = TokenizationState::ScriptData;
                    self.current_tag = None;
                    return Some(HtmlToken::Character(Cow::Borrowed(buffered_slice)));
                }
            },

            TokenizationState::ScriptDataEscapeStart => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapeStartDash;
                        return Some(HtmlToken::Character(Cow::Borrowed("-")));
                    }
                    _ => {
                        self.state = TokenizationState::ScriptData;
                    }
                }
            },

            TokenizationState::ScriptDataEscapeStartDash => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapedDashDash;
                        return Some(HtmlToken::Character(Cow::Borrowed("-")));
                    }
                    _ => {
                        self.state = TokenizationState::ScriptData;
                    }
                }
            },

            TokenizationState::ScriptDataEscaped => {
                let start = self.pos;

                while let Some(c) = self.peek() {
                    match c {
                        '-' | '<' | '\0' => break,
                        _ => { self.consume(); }
                    }
                }
                
                if self.pos > start {
                    return Some(HtmlToken::Character(Cow::Borrowed(&self.input[start..self.pos])));
                }

                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapedDash;
                        return Some(HtmlToken::Character(Cow::Borrowed("-")));
                    }
                    Some('<') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapedLessThanSign;
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                    }
                    None => {
                        self.errors.push(Error::EofInScriptHtmlCommentLikeText);
                        return Some(HtmlToken::EndOfFile);
                    }
                    _ => unreachable!(),
                }
            },

            TokenizationState::ScriptDataEscapedDash => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapedDashDash;
                        return Some(HtmlToken::Character(Cow::Borrowed("-")));
                    }
                    Some('<') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapedLessThanSign;
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                    }
                    None => {
                        self.errors.push(Error::EofInScriptHtmlCommentLikeText);
                        return Some(HtmlToken::EndOfFile);
                    }
                    Some(_) => {
                        let start = self.pos;
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[start..self.pos])));
                    }
                }
            },


            TokenizationState::ScriptDataEscapedDashDash => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        return Some(HtmlToken::Character(Cow::Borrowed("-")));
                    }
                    Some('<') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapedLessThanSign;
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::ScriptData;
                        return Some(HtmlToken::Character(Cow::Borrowed(">")));
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                    }
                    None => {
                        self.errors.push(Error::EofInScriptHtmlCommentLikeText);
                        return Some(HtmlToken::EndOfFile);
                    }
                    Some(_) => {
                        let start = self.pos;
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[start..self.pos])));
                    }
                }
            },

            TokenizationState::ScriptDataEscapedLessThanSign => {
                match self.peek() {
                    Some('/') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataEscapedEndTagOpen;
                    }
                    Some(c) if Self::is_ascii_alpha(c) => {
                        self.mark = self.pos;
                        self.state = TokenizationState::ScriptDataDoubleEscapeStart;
                        return Some(HtmlToken::Character(Cow::Borrowed("<")));
                    }
                    _ => {
                        self.state = TokenizationState::ScriptDataEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed("<")));
                    }
                }
            },

            TokenizationState::ScriptDataEscapedEndTagOpen => {
                match self.peek() {
                    Some(c) if Self::is_ascii_alpha(c) => {
                        self.current_tag = Some(Tag {
                            name: None,
                            self_closing_tag: Some(false),
                            attributes: Vec::new(),
                        });
                        self.is_current_tag_end = true;
                        self.mark = self.pos;
                        self.state = TokenizationState::ScriptDataEscapedEndTagName;
                    }
                    _ => {
                        self.state = TokenizationState::ScriptDataEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed("</")));
                    }
                }
            },

            TokenizationState::ScriptDataEscapedEndTagName => {
                let mut is_appropriate = false;
                let mut tag_completed = false;
                let mut switch_state = None;

                while let Some(c) = self.peek() {
                    match c {
                        '\t' | '\n' | '\x0C' | ' ' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::BeforeAttributeName);
                            }
                            break;
                        }
                        '/' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::SelfClosingStartTag);
                            }
                            break;
                        }
                        '>' => {
                            is_appropriate = self.is_appropriate_end_tag();
                            if is_appropriate {
                                self.consume();
                                switch_state = Some(TokenizationState::Data);
                                tag_completed = true;
                            }
                            break;
                        }
                        c if Self::is_ascii_alpha(c) => {
                            self.consume();
                        }
                        _ => break,
                    }
                }

                if is_appropriate {
                    self.state = switch_state.unwrap();
                    
                    if let Some(tag) = self.current_tag.as_mut() {
                        if tag.name.is_none() {
                            let mut name_slice = &self.input[self.mark..self.pos];
                            name_slice = name_slice.trim_end_matches(&['\t', '\n', '\x0C', ' ', '/', '>'][..]);
                            tag.name = Some(Self::to_lower_cow(name_slice));
                        }
                    }

                    if tag_completed {
                        return Some(HtmlToken::EndTag(self.current_tag.take().unwrap()));
                    }
                } else {
                    let emit_slice_start = self.mark - 2; 
                    let buffered_slice = &self.input[emit_slice_start..self.pos];
                    
                    self.state = TokenizationState::ScriptDataEscaped;
                    self.current_tag = None;
                    return Some(HtmlToken::Character(Cow::Borrowed(buffered_slice)));
                }
            },


            TokenizationState::ScriptDataDoubleEscapeStart => {
                let mut switch_state = None;
                
                while let Some(c) = self.peek() {
                    match c {
                        '\t' | '\n' | '\x0C' | ' ' | '/' | '>' => {
                            let temp_buffer = &self.input[self.mark..self.pos];
                            let is_script = temp_buffer.eq_ignore_ascii_case("script");
                            
                            switch_state = Some(if is_script {
                                TokenizationState::ScriptDataDoubleEscaped
                            } else {
                                TokenizationState::ScriptDataEscaped
                            });
                            
                            self.consume();
                            break;
                        }
                        c if Self::is_ascii_alpha(c) => {
                            self.consume();
                        }
                        _ => {
                            switch_state = Some(TokenizationState::ScriptDataEscaped);
                            break;
                        }
                    }
                }
                
                if let Some(new_state) = switch_state {
                    self.state = new_state;
                    // Emits the text buffer AND the terminating character at the same time
                    return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                }
            },


            TokenizationState::ScriptDataDoubleEscaped => {
                // High-performance batching again
                let start = self.pos;
                while let Some(c) = self.peek() {
                    match c {
                        '-' | '<' | '\0' => break,
                        _ => { self.consume(); },
                    }
                }
                
                if self.pos > start {
                    return Some(HtmlToken::Character(Cow::Borrowed(&self.input[start..self.pos])));
                }

                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataDoubleEscapedDash;
                        return Some(HtmlToken::Character(Cow::Borrowed("-")));
                    }
                    Some('<') => {
                        self.consume();
                        // Note: The spec refers to ScriptDataDoubleEscapedLessThanSign here
                        self.state = TokenizationState::ScriptDataDoubleEscapedLessThanSign;
                        return Some(HtmlToken::Character(Cow::Borrowed("<")));
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                    }
                    None => {
                        self.errors.push(Error::EofInScriptHtmlCommentLikeText);
                        return Some(HtmlToken::EndOfFile);
                    }
                    _ => unreachable!(),
                }
            },


            TokenizationState::ScriptDataDoubleEscapedDash => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataDoubleEscapedDashDash;
                        return Some(HtmlToken::Character(Cow::Borrowed("-")));
                    }
                    Some('<') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataDoubleEscapedLessThanSign;
                        return Some(HtmlToken::Character(Cow::Borrowed("<")));
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        self.state = TokenizationState::ScriptDataDoubleEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                    }
                    None => {
                        self.errors.push(Error::EofInScriptHtmlCommentLikeText);
                        return Some(HtmlToken::EndOfFile);
                    }
                    Some(_) => {
                        let start = self.pos;
                        self.consume();
                        self.state = TokenizationState::ScriptDataDoubleEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[start..self.pos])));
                    }
                }
            },

            TokenizationState::ScriptDataDoubleEscapedDashDash => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        return Some(HtmlToken::Character(Cow::Borrowed("-")));
                    }
                    Some('<') => {
                        self.consume();
                        self.state = TokenizationState::ScriptDataDoubleEscapedLessThanSign;
                        return Some(HtmlToken::Character(Cow::Borrowed("<")));
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::ScriptData;
                        return Some(HtmlToken::Character(Cow::Borrowed(">")));
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        self.state = TokenizationState::ScriptDataDoubleEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                    }
                    None => {
                        self.errors.push(Error::EofInScriptHtmlCommentLikeText);
                        return Some(HtmlToken::EndOfFile);
                    }
                    Some(_) => {
                        let start = self.pos;
                        self.consume();
                        self.state = TokenizationState::ScriptDataDoubleEscaped;
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[start..self.pos])));
                    }
                }
            },

            TokenizationState::ScriptDataDoubleEscapedLessThanSign => {
                match self.peek() {
                    Some('/') => {
                        self.consume();
                        self.mark = self.pos;
                        self.state = TokenizationState::ScriptDataDoubleEscapeEnd;
                        return Some(HtmlToken::Character(Cow::Borrowed("/")));
                    }
                    _ => {
                        self.state = TokenizationState::ScriptDataDoubleEscaped;
                    }
                }
            },

            TokenizationState::ScriptDataDoubleEscapeEnd => {
                let mut switch_state = None;
                
                while let Some(c) = self.peek() {
                    match c {
                        '\t' | '\n' | '\x0C' | ' ' | '/' | '>' => {
                            let temp_buffer = &self.input[self.mark..self.pos];
                            let is_script = temp_buffer.eq_ignore_ascii_case("script");
                            
                            switch_state = Some(if is_script {
                                TokenizationState::ScriptDataEscaped
                            } else {
                                TokenizationState::ScriptDataDoubleEscaped
                            });
                            
                            self.consume();
                            break;
                        }
                        c if Self::is_ascii_alpha(c) => {
                            self.consume();
                        }
                        _ => {
                            switch_state = Some(TokenizationState::ScriptDataDoubleEscaped);
                            break;
                        }
                    }
                }
                
                if let Some(new_state) = switch_state {
                    self.state = new_state;
                    return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                }
            },

            TokenizationState::BeforeAttributeName => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                    }
                    Some('/') | Some('>') | None => {
                        self.state = TokenizationState::AfterAttributeName;
                    }
                    Some('=') => {
                        self.errors.push(Error::UnexpectedEqualsSignBeforeAttributeName);
                        self.consume();
                        
                        let mut tag = self.current_tag.as_mut().unwrap();
                        tag.attributes.push(Attribute {
                            name: Some(Cow::Borrowed("=")),
                            value: Some(Cow::Borrowed("")),
                        });
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeName;
                    }
                    Some(_) => {
                        let mut tag = self.current_tag.as_mut().unwrap();
                        tag.attributes.push(Attribute {
                            name: None,
                            value: None,
                        });
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeName;
                    }
                }
            },

            TokenizationState::AttributeName=> {
                let mut attribute_completed = false;
                let mut switch_state = None;

                while let Some(c) = self.peek() {
                    match c {
                        '\t' | '\n' | '\x0C' | ' ' | '/' | '>' | '\0' => {
                            if c == '\0' {
                                self.errors.push(Error::UnexpectedNullCharacter);
                            }
                            switch_state = Some(TokenizationState::AfterAttributeName);
                            attribute_completed = true;
                            break;
                        }
                        '=' => {
                            self.consume();
                            switch_state = Some(TokenizationState::BeforeAttributeValue);
                            attribute_completed = true;
                            break;
                        }
                        '"' | '\'' | '<' => {
                            self.errors.push(Error::UnexpectedCharacterInAttributeName);
                            self.consume();
                        }
                        _ => {
                            self.consume();
                        }
                    }
                }

                if self.pos >= self.input.len() && !attribute_completed {
                    switch_state = Some(TokenizationState::AfterAttributeName);
                    attribute_completed = true;
                }

                if attribute_completed {
                    if let Some(state) = switch_state {
                        self.state = state;
                    }
                    
                    let tag = self.current_tag.as_mut().unwrap();
                    let name_slice = &self.input[self.mark..self.pos];
                    
                    let final_name = if name_slice.contains('\0') {
                        Cow::Owned(name_slice.replace('\0', "\u{FFFD}").to_ascii_lowercase())
                    } else {
                        Self::to_lower_cow(name_slice)
                    };

                    // Check for duplicate attributes
                    let is_duplicate = tag.attributes[..tag.attributes.len() - 1]
                        .iter()
                        .any(|attr| attr.name.as_ref() == Some(&final_name));

                    if is_duplicate {
                        self.errors.push(Error::DuplicateAttribute);
                        tag.attributes.pop(); // Discard the duplicate
                    } else {
                        if let Some(last_attr) = tag.attributes.last_mut() {
                            last_attr.name = Some(final_name);
                            // Default value to empty string in case it never gets one
                            last_attr.value = Some(Cow::Borrowed("")); 
                        }
                    }
                }
            },

            TokenizationState::AfterAttributeName=> {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                    }
                    Some('/') => {
                        self.consume();
                        self.state = TokenizationState::SelfClosingStartTag;
                    }
                    Some('=') => {
                        self.consume();
                        self.state = TokenizationState::BeforeAttributeValue;
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::StartTag(self.current_tag.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInTag);
                        return Some(HtmlToken::EndOfFile);
                    }
                    Some(_) => {
                        let mut tag = self.current_tag.as_mut().unwrap();
                        tag.attributes.push(Attribute {
                            name: None,
                            value: None,
                        });
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeName;
                    }
                }
            },

            TokenizationState::BeforeAttributeValue=> {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                    }
                    Some('"') => {
                        self.consume();
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeValueDoubleQuoted;
                    }
                    Some('\'') => {
                        self.consume();
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeValueSingleQuoted;
                    }
                    Some('>') => {
                        self.errors.push(Error::MissingAttributeValue);
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::StartTag(self.current_tag.take().unwrap()));
                    }
                    Some(_) => {
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeValueUnquoted;
                    }
                    None => { }
                }
            },

            TokenizationState::AttributeValueDoubleQuoted => {
                let mut value_completed = false;
                
                while let Some(c) = self.peek() {
                    match c {
                        '"' => {
                            value_completed = true;
                            self.consume();
                            self.state = TokenizationState::AfterAttributeValueQuoted;
                            break;
                        }
                        '&' => {
                            self.return_state = TokenizationState::AttributeValueDoubleQuoted;
                            self.state = TokenizationState::CharacterReference;
                            break;
                        }
                        '\0' => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            self.consume();
                        }
                        _ => {
                            self.consume();
                        }
                    }
                }

                if value_completed {
                    let mut value_slice = &self.input[self.mark..self.pos - 1];
                    let final_value = if value_slice.contains('\0') {
                        Cow::Owned(value_slice.replace('\0', "\u{FFFD}"))
                    } else {
                        Cow::Borrowed(value_slice)
                    };

                    if let Some(tag) = self.current_tag.as_mut() {
                        if let Some(last_attr) = tag.attributes.last_mut() {
                            last_attr.value = Some(final_value);
                        }
                    }
                } else if self.pos >= self.input.len() {
                    self.errors.push(Error::EofInTag);
                    return Some(HtmlToken::EndOfFile);
                }
            },

            TokenizationState::AttributeValueSingleQuoted => {
                let mut value_completed = false;
                
                while let Some(c) = self.peek() {
                    match c {
                        '\'' => {
                            value_completed = true;
                            self.consume();
                            self.state = TokenizationState::AfterAttributeValueQuoted;
                            break;
                        }
                        '&' => {
                            self.return_state = TokenizationState::AttributeValueSingleQuoted;
                            self.state = TokenizationState::CharacterReference;
                            break;
                        }
                        '\0' => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            self.consume();
                        }
                        _ => {
                            self.consume();
                        }
                    }
                }

                if value_completed {
                    let mut value_slice = &self.input[self.mark..self.pos - 1];
                    let final_value = if value_slice.contains('\0') {
                        Cow::Owned(value_slice.replace('\0', "\u{FFFD}"))
                    } else {
                        Cow::Borrowed(value_slice)
                    };

                    if let Some(tag) = self.current_tag.as_mut() {
                        if let Some(last_attr) = tag.attributes.last_mut() {
                            last_attr.value = Some(final_value);
                        }
                    }
                } else if self.pos >= self.input.len() {
                    self.errors.push(Error::EofInTag);
                    return Some(HtmlToken::EndOfFile);
                }
            },

            TokenizationState::AttributeValueUnquoted => {
                let mut value_completed = false;
                let mut switch_state = None;
                
                while let Some(c) = self.peek() {
                    match c {
                        '\t' | '\n' | '\x0C' | ' ' => {
                            self.consume();
                            switch_state = Some(TokenizationState::BeforeAttributeName);
                            value_completed = true;
                            break;
                        }
                        '&' => {
                            self.return_state = TokenizationState::AttributeValueUnquoted;
                            self.state = TokenizationState::CharacterReference;
                            break;
                        }
                        '>' => {
                            self.consume();
                            switch_state = Some(TokenizationState::Data);
                            value_completed = true;
                            break;
                        }
                        '\0' => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            self.consume();
                        }
                        '"' | '\'' | '<' | '=' | '`' => {
                            self.errors.push(Error::UnexpectedCharacterInUnquotedAttributeValue);
                            self.consume();
                        }
                        _ => {
                            self.consume();
                        }
                    }
                }

                if self.pos >= self.input.len() && !value_completed {
                    self.errors.push(Error::EofInTag);
                    return Some(HtmlToken::EndOfFile);
                }

                if value_completed {
                    if let Some(state) = switch_state {
                        self.state = state;
                    }
                    
                    let mut value_slice = &self.input[self.mark..if self.state == TokenizationState::Data { self.pos - 1 } else { self.pos }];
                    value_slice = value_slice.trim_end_matches(&['\t', '\n', '\x0C', ' '][..]);
                    
                    let final_value = if value_slice.contains('\0') {
                        Cow::Owned(value_slice.replace('\0', "\u{FFFD}"))
                    } else {
                        Cow::Borrowed(value_slice)
                    };

                    if let Some(tag) = self.current_tag.as_mut() {
                        if let Some(last_attr) = tag.attributes.last_mut() {
                            last_attr.value = Some(final_value);
                        }
                    }
                    
                    // If we transitioned to Data, emit the token now
                    if self.state == TokenizationState::Data {
                        return Some(HtmlToken::StartTag(self.current_tag.take().unwrap()));
                    }
                }
            },

            TokenizationState::AfterAttributeValueQuoted => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = TokenizationState::BeforeAttributeName;
                    }
                    Some('/') => {
                        self.consume();
                        self.state = TokenizationState::SelfClosingStartTag;
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::StartTag(self.current_tag.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInTag);
                        return Some(HtmlToken::EndOfFile);
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingWhitespaceBetweenAttributes);
                        self.state = TokenizationState::BeforeAttributeName;
                    }
                }
            },

            TokenizationState::SelfClosingStartTag => {
                match self.peek() {
                    Some('>') => {
                        self.consume();
                        if let Some(tag) = self.current_tag.as_mut() {
                            tag.self_closing_tag = Some(true);
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::StartTag(self.current_tag.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInTag);
                        return Some(HtmlToken::EndOfFile);
                    }
                    Some(_) => {
                        self.errors.push(Error::UnexpectedSolidusInTag);
                        self.state = TokenizationState::BeforeAttributeName;
                    }
                }
            },

            TokenizationState::BogusComment => {
                let mut comment_completed = false;
                
                while let Some(c) = self.peek() {
                    match c {
                        '>' => {
                            self.consume();
                            self.state = TokenizationState::Data;
                            comment_completed = true;
                            break;
                        }
                        '\0' => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            self.consume();
                        }
                        _ => {
                            self.consume();
                        }
                    }
                }

                // Eof or '>' reached
                let end_offset = if comment_completed { 1 } else { 0 };
                let comment_text = &self.input[self.mark..self.pos - end_offset];
                
                let data = if comment_text.contains('\0') {
                    Cow::Owned(comment_text.replace('\0', "\u{FFFD}"))
                } else {
                    Cow::Borrowed(comment_text)
                };

                return Some(HtmlToken::Comment(data));
            },

            TokenizationState::MarkupDeclarationOpen => {
                let remaining = &self.input[self.pos..];
                
                if remaining.starts_with("--") {
                    self.consume();
                    self.consume();
                    self.mark = self.pos;
                    self.state = TokenizationState::CommentStart;
                } else if remaining.len() >= 7 && remaining[..7].eq_ignore_ascii_case("doctype") {
                    for _ in 0..7 { self.consume(); }
                    self.state = TokenizationState::Doctype;
                } else if remaining.starts_with("[CDATA[") {
                    for _ in 0..7 { self.consume(); }
                    
                    // HTML namespace behavior
                    self.errors.push(Error::CdataInHtmlContent);
                    self.mark = self.pos - 7; 
                    self.state = TokenizationState::BogusComment;
                } else {
                    self.errors.push(Error::IncorrectlyOpenedComment);
                    self.mark = self.pos;
                    self.state = TokenizationState::BogusComment;
                }
            },

            TokenizationState::CommentStart => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentStartDash;
                    }
                    Some('>') => {
                        self.errors.push(Error::AbruptClosingOfEmptyComment);
                        self.consume();
                        self.state = TokenizationState::Data;
                        
                        return Some(HtmlToken::Comment(Cow::Borrowed(&self.input[self.mark..self.pos - 1])));
                    }
                    _ => {
                        self.state = TokenizationState::Comment;
                    }
                }
            },

            TokenizationState::CommentStartDash => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentEnd;
                    }
                    Some('>') => {
                        self.errors.push(Error::AbruptClosingOfEmptyComment);
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Comment(Cow::Borrowed(&self.input[self.mark..self.pos - 2])));
                    }
                    None => {
                        self.errors.push(Error::EofInComment);
                        return Some(HtmlToken::Comment(Cow::Borrowed(&self.input[self.mark..self.pos - 1])));
                    }
                    Some(_) => {
                        self.state = TokenizationState::Comment;
                    }
                }
            },

            TokenizationState::Comment => {
                let start = self.pos;
                while let Some(c) = self.peek() {
                    match c {
                        '<' | '-' | '\0' => break,
                        _ => { self.consume(); },
                    }
                }

                match self.peek() {
                    Some('<') => {
                        self.consume();
                        self.state = TokenizationState::CommentLessThanSign;
                    }
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentEndDash;
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                    }
                    None => {
                        self.errors.push(Error::EofInComment);
                        
                        let comment_text = &self.input[self.mark..self.pos];
                        let data = if comment_text.contains('\0') {
                            Cow::Owned(comment_text.replace('\0', "\u{FFFD}"))
                        } else {
                            Cow::Borrowed(comment_text)
                        };
                        return Some(HtmlToken::Comment(data));
                    }
                    _ => unreachable!(),
                }
            },

            TokenizationState::CommentLessThanSign => {
                match self.peek() {
                    Some('!') => {
                        self.consume();
                        self.state = TokenizationState::CommentLessThanSignBang;
                    }
                    Some('<') => {
                        self.consume();
                    }
                    _ => {
                        self.state = TokenizationState::Comment;
                    }
                }
            },

            TokenizationState::CommentLessThanSignBang => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentLessThanSignBangDash;
                    }
                    _ => {
                        self.state = TokenizationState::Comment;
                    }
                }
            },

            TokenizationState::CommentLessThanSignBangDash => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentLessThanSignBangDashDash;
                    }
                    _ => {
                        self.state = TokenizationState::CommentEndDash;
                    }
                }
            },

            TokenizationState::CommentLessThanSignBangDashDash => {
                match self.peek() {
                    Some('>') | None => {
                        self.state = TokenizationState::CommentEnd;
                    }
                    Some(_) => {
                        self.errors.push(Error::NestedComment);
                        self.state = TokenizationState::CommentEnd;
                    }
                }
            },

            TokenizationState::CommentEndDash => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentEnd;
                    }
                    None => {
                        self.errors.push(Error::EofInComment);
                        
                        let comment_text = &self.input[self.mark..self.pos - 1];
                        let data = if comment_text.contains('\0') {
                            Cow::Owned(comment_text.replace('\0', "\u{FFFD}"))
                        } else {
                            Cow::Borrowed(comment_text)
                        };
                        return Some(HtmlToken::Comment(data));
                    }
                    Some(_) => {
                        self.state = TokenizationState::Comment;
                    }
                }
            },

            TokenizationState::CommentEnd => {
                match self.peek() {
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        
                        let comment_text = &self.input[self.mark..self.pos - 3];
                        let data = if comment_text.contains('\0') {
                            Cow::Owned(comment_text.replace('\0', "\u{FFFD}"))
                        } else {
                            Cow::Borrowed(comment_text)
                        };
                        return Some(HtmlToken::Comment(data));
                    }
                    Some('!') => {
                        self.consume();
                        self.state = TokenizationState::CommentEndBang;
                    }
                    Some('-') => {
                        self.consume();
                    }
                    None => {
                        self.errors.push(Error::EofInComment);
                        
                        let comment_text = &self.input[self.mark..self.pos - 2];
                        let data = if comment_text.contains('\0') {
                            Cow::Owned(comment_text.replace('\0', "\u{FFFD}"))
                        } else {
                            Cow::Borrowed(comment_text)
                        };
                        return Some(HtmlToken::Comment(data));
                    }
                    Some(_) => {
                        self.state = TokenizationState::Comment;
                    }
                }
            },

            TokenizationState::CommentEndBang => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentEndDash;
                    }
                    Some('>') => {
                        self.errors.push(Error::IncorrectlyClosedComment);
                        self.consume();
                        self.state = TokenizationState::Data;
                        
                        let comment_text = &self.input[self.mark..self.pos - 4];
                        let data = if comment_text.contains('\0') {
                            Cow::Owned(comment_text.replace('\0', "\u{FFFD}"))
                        } else {
                            Cow::Borrowed(comment_text)
                        };
                        return Some(HtmlToken::Comment(data));
                    }
                    None => {
                        self.errors.push(Error::EofInComment);
                        
                        let comment_text = &self.input[self.mark..self.pos - 3];
                        let data = if comment_text.contains('\0') {
                            Cow::Owned(comment_text.replace('\0', "\u{FFFD}"))
                        } else {
                            Cow::Borrowed(comment_text)
                        };
                        return Some(HtmlToken::Comment(data));
                    }
                    Some(_) => {
                        self.state = TokenizationState::Comment;
                    }
                }
            },
            

            TokenizationState::Doctype => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = TokenizationState::BeforeDoctypeName;
                    }
                    Some('>') => {
                        self.state = TokenizationState::BeforeDoctypeName;
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        return Some(HtmlToken::Doctype(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: true,
                        }));
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingWhitespaceBeforeDoctypeName);
                        self.state = TokenizationState::BeforeDoctypeName;
                    }
                }
            },


            TokenizationState::BeforeDoctypeName => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                    }
                    Some('>') => {
                        self.errors.push(Error::MissingDoctypeName);
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: true,
                        }));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        return Some(HtmlToken::Doctype(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: true,
                        }));
                    }
                    Some(_) => {
                        self.current_doctype = Some(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: false,
                        });
                        self.mark = self.pos;
                        self.state = TokenizationState::DoctypeName;
                    }
                }
            },

            TokenizationState::DoctypeName => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = TokenizationState::AfterDoctypeName;
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.name
                                .get_or_insert_with(|| Cow::Owned(String::new()))
                                .to_mut()
                                .push('\u{FFFD}');
                        }
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(c) => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.name
                                .get_or_insert_with(|| Cow::Owned(String::new()))
                                .to_mut()
                                .push(c.to_ascii_lowercase());
                        }
                    }
                }
            },



            TokenizationState::AfterDoctypeName => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume(); // Ignore
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        let remaining = &self.input[self.pos..];
                        if remaining.len() >= 6 && remaining[..6].eq_ignore_ascii_case("PUBLIC") {
                            for _ in 0..6 { self.consume(); }
                            self.state = TokenizationState::AfterDoctypePublicKeyword;
                        } else if remaining.len() >= 6 && remaining[..6].eq_ignore_ascii_case("SYSTEM") {
                            for _ in 0..6 { self.consume(); }
                            self.state = TokenizationState::AfterDoctypeSystemKeyword;
                        } else {
                            self.errors.push(Error::InvalidCharacterSequenceAfterDoctypeName);
                            if let Some(dt) = self.current_doctype.as_mut() {
                                dt.force_quirks_flag = true;
                            }
                            self.state = TokenizationState::BogusDoctype;
                        }
                    }
                }
            },


            TokenizationState::AfterDoctypePublicKeyword => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = TokenizationState::BeforeDoctypePublicIdentifier;
                    }
                    Some('"') => {
                        self.errors.push(Error::MissingWhitespaceAfterDoctypePublicKeyword);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.public_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypePublicIdentifierDoubleQuoted;
                    }
                    Some('\'') => {
                        self.errors.push(Error::MissingWhitespaceAfterDoctypePublicKeyword);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.public_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypePublicIdentifierSingleQuoted;
                    }
                    Some('>') => {
                        self.errors.push(Error::MissingDoctypePublicIdentifier);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingQuoteBeforeDoctypePublicIdentifier);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::BogusDoctype;
                    }
                }
            },


            TokenizationState::BeforeDoctypePublicIdentifier => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume(); // Ignore
                    }
                    Some('"') => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.public_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypePublicIdentifierDoubleQuoted;
                    }
                    Some('\'') => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.public_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypePublicIdentifierSingleQuoted;
                    }
                    Some('>') => {
                        self.errors.push(Error::MissingDoctypePublicIdentifier);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingQuoteBeforeDoctypePublicIdentifier);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::BogusDoctype;
                    }
                }
            },


            TokenizationState::DoctypePublicIdentifierDoubleQuoted => {
                match self.peek() {
                    Some('"') => {
                        self.consume();
                        self.state = TokenizationState::AfterDoctypePublicIdentifier;
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.public_identifier.as_mut().unwrap().to_mut().push('\u{FFFD}');
                        }
                    }
                    Some('>') => {
                        self.errors.push(Error::AbruptDoctypePublicIdentifier);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(c) => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.public_identifier.as_mut().unwrap().to_mut().push(c);
                        }
                    }
                }
            },


            TokenizationState::DoctypePublicIdentifierSingleQuoted => {
                match self.peek() {
                    Some('\'') => {
                        self.consume();
                        self.state = TokenizationState::AfterDoctypePublicIdentifier;
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.public_identifier.as_mut().unwrap().to_mut().push('\u{FFFD}');
                        }
                    }
                    Some('>') => {
                        self.errors.push(Error::AbruptDoctypePublicIdentifier);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(c) => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.public_identifier.as_mut().unwrap().to_mut().push(c);
                        }
                    }
                }
            },

            TokenizationState::AfterDoctypePublicIdentifier => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = TokenizationState::BetweenDoctypePublicAndSystemIdentifiers;
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some('"') => {
                        self.errors.push(Error::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypeSystemIdentifierDoubleQuoted;
                    }
                    Some('\'') => {
                        self.errors.push(Error::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypeSystemIdentifierSingleQuoted;
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingQuoteBeforeDoctypeSystemIdentifier);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::BogusDoctype;
                    }
                }
            },


            TokenizationState::BetweenDoctypePublicAndSystemIdentifiers => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume(); // Ignore
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some('"') => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypeSystemIdentifierDoubleQuoted;
                    }
                    Some('\'') => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypeSystemIdentifierSingleQuoted;
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingQuoteBeforeDoctypeSystemIdentifier);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::BogusDoctype;
                    }
                }
            },


            TokenizationState::AfterDoctypeSystemKeyword => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = TokenizationState::BeforeDoctypeSystemIdentifier;
                    }
                    Some('"') => {
                        self.errors.push(Error::MissingWhitespaceAfterDoctypeSystemKeyword);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypeSystemIdentifierDoubleQuoted;
                    }
                    Some('\'') => {
                        self.errors.push(Error::MissingWhitespaceAfterDoctypeSystemKeyword);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypeSystemIdentifierSingleQuoted;
                    }
                    Some('>') => {
                        self.errors.push(Error::MissingDoctypeSystemIdentifier);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingQuoteBeforeDoctypeSystemIdentifier);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::BogusDoctype;
                    }
                }
            },

            TokenizationState::BeforeDoctypeSystemIdentifier => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume(); // Ignore
                    }
                    Some('"') => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypeSystemIdentifierDoubleQuoted;
                    }
                    Some('\'') => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier = Some(Cow::Borrowed(""));
                        }
                        self.state = TokenizationState::DoctypeSystemIdentifierSingleQuoted;
                    }
                    Some('>') => {
                        self.errors.push(Error::MissingDoctypeSystemIdentifier);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingQuoteBeforeDoctypeSystemIdentifier);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::BogusDoctype;
                    }
                }
            },


            TokenizationState::DoctypeSystemIdentifierDoubleQuoted => {
                match self.peek() {
                    Some('"') => {
                        self.consume();
                        self.state = TokenizationState::AfterDoctypeSystemIdentifier;
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier.as_mut().unwrap().to_mut().push('\u{FFFD}');
                        }
                    }
                    Some('>') => {
                        self.errors.push(Error::AbruptDoctypeSystemIdentifier);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(c) => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier.as_mut().unwrap().to_mut().push(c);
                        }
                    }
                }
            },


            TokenizationState::DoctypeSystemIdentifierSingleQuoted => {
                match self.peek() {
                    Some('\'') => {
                        self.consume();
                        self.state = TokenizationState::AfterDoctypeSystemIdentifier;
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier.as_mut().unwrap().to_mut().push('\u{FFFD}');
                        }
                    }
                    Some('>') => {
                        self.errors.push(Error::AbruptDoctypeSystemIdentifier);
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(c) => {
                        self.consume();
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.system_identifier.as_mut().unwrap().to_mut().push(c);
                        }
                    }
                }
            },

            TokenizationState::AfterDoctypeSystemIdentifier => {
                match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume(); // Ignore
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        if let Some(dt) = self.current_doctype.as_mut() {
                            dt.force_quirks_flag = true;
                        }
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        self.errors.push(Error::UnexpectedCharacterAfterDoctypeSystemIdentifier);
                        self.state = TokenizationState::BogusDoctype;
                    }
                }
            },


            TokenizationState::BogusDoctype => {
                match self.peek() {
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.consume();
                    }
                    None => {
                        return Some(HtmlToken::Doctype(self.current_doctype.take().unwrap()));
                    }
                    Some(_) => {
                        self.consume();
                    }
                }
            },

            TokenizationState::CdataSection => {
                    self.mark = self.pos;

                    while let Some(c) = self.peek() {
                        if c == ']' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        // Null characters (\0) are handled in tree construction for CDATA
                        return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                    }

                    match self.consume() {
                        Some(']') => {
                            self.state = TokenizationState::CdataSectionBracket;
                        }
                        None => {
                            self.errors.push(Error::EofInCdata);
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => unreachable!(),
                    }
                },


                TokenizationState::CdataSectionBracket => {
                    match self.peek() {
                        Some(']') => {
                            self.consume();
                            self.state = TokenizationState::CdataSectionEnd;
                        }
                        _ => {
                            self.state = TokenizationState::CdataSection;
                            return Some(HtmlToken::Character(Cow::Borrowed("]")));
                        }
                    }
                },


                TokenizationState::CdataSectionEnd => {
                    match self.peek() {
                        Some(']') => {
                            self.consume();
                            return Some(HtmlToken::Character(Cow::Borrowed("]")));
                        }
                        Some('>') => {
                            self.consume();
                            self.state = TokenizationState::Data;
                        }
                        _ => {
                            self.state = TokenizationState::CdataSection;
                            return Some(HtmlToken::Character(Cow::Borrowed("]]")));
                        }
                    }
                },


                TokenizationState::ProcessingInstructionOpen => {
                    match self.peek() {
                        Some(c) if Self::is_ascii_alpha(c) || c == '_' => {
                            self.mark = self.pos;
                            self.state = TokenizationState::ProcessingInstructionTarget;
                        }
                        None => {
                            self.errors.push(Error::EofInProcessingInstruction);
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => {
                            self.errors.push(Error::InvalidFirstCharacterOfProcessingInstructionTarget);
                            self.state = TokenizationState::BogusComment;
                        }
                    }
                },


                TokenizationState::ProcessingInstructionTarget => {
                    match self.peek() {
                        Some('\t') | Some('\n') | Some('\x0C') | Some(' ') | Some('?') | Some('>') => {
                            let target = &self.input[self.mark..self.pos];
                            
                            if target.eq_ignore_ascii_case("xml") 
                                || target.eq_ignore_ascii_case("xml-stylesheet") 
                            {
                                self.errors.push(Error::DisallowedProcessingInstructionTarget);
                                self.state = TokenizationState::BogusComment;
                            } else {
                                self.state = TokenizationState::AfterProcessingInstructionTarget;
                            }
                        }
                        Some(c) if c.is_ascii_alphanumeric() || c == '-' || c == '_' => {
                            self.consume();
                        }
                        None => {
                            self.errors.push(Error::EofInProcessingInstruction);
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => {
                            self.errors.push(Error::InvalidProcessingInstructionTarget);
                            self.state = TokenizationState::BogusComment;
                        }
                    }
                },


                TokenizationState::AfterProcessingInstructionTarget => {
                    match self.peek() {
                        Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                            self.consume();
                        }
                        _ => {
                            self.mark = self.pos;
                            self.state = TokenizationState::ProcessingInstructionData;
                        }
                    }
                },


                TokenizationState::ProcessingInstructionData => {
                    while let Some(c) = self.peek() {
                        if c == '?' || c == '>' {
                            break;
                        }
                        self.consume();
                    }

                    match self.consume() {
                        Some('?') => {
                            self.state = TokenizationState::ProcessingInstructionQuestionable;
                        }
                        Some('>') => {
                            self.state = TokenizationState::Data;
                            // return Some(HtmlToken::ProcessingInstruction(...));
                        }
                        None => {
                            self.errors.push(Error::EofInProcessingInstruction);
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => unreachable!()
                    }
                },


                TokenizationState::ProcessingInstructionQuestionable => {
                    match self.peek() {
                        Some('>') => {
                            self.consume();
                            self.state = TokenizationState::Data;
                            // return Some(HtmlToken::ProcessingInstruction(...));
                        }
                        None => {
                            self.errors.push(Error::EofInProcessingInstruction);
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => {
                            self.state = TokenizationState::ProcessingInstructionData;
                        }
                    }
                },


                TokenizationState::CharacterReference => {
                    self.mark = self.pos - 1;
                    
                    match self.peek() {
                        Some(c) if c.is_ascii_alphanumeric() => {
                            self.state = TokenizationState::NamedCharacterReference;
                        }
                        Some('#') => {
                            self.consume();
                            self.state = TokenizationState::NumericCharacterReference;
                        }
                        _ => {
                            self.state = self.return_state;
                            return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                        }
                    }
                },


                TokenizationState::NamedCharacterReference => {
                    self.state = TokenizationState::AmbiguousAmpersand;
                },


                TokenizationState::AmbiguousAmpersand => {
                    match self.peek() {
                        Some(c) if c.is_ascii_alphanumeric() => {
                            self.consume();
                        }
                        Some(';') => {
                            self.errors.push(Error::UnknownNamedCharacterReference);
                            self.consume();
                            self.state = self.return_state;
                            return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                        }
                        _ => {
                            self.state = self.return_state;
                            return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                        }
                    }
                },


                TokenizationState::NumericCharacterReference => {
                    self.character_reference_code = 0;
                    
                    match self.peek() {
                        Some('x') | Some('X') => {
                            self.consume();
                            self.state = TokenizationState::HexadecimalCharacterReferenceStart;
                        }
                        Some(c) if c.is_ascii_digit() => {
                            self.state = TokenizationState::DecimalCharacterReference;
                        }
                        _ => {
                            self.errors.push(Error::AbsenceOfDigitsInNumericCharacterReference);
                            self.state = self.return_state;
                            return Some(HtmlToken::Character(Cow::Borrowed(&self.input[self.mark..self.pos])));
                        }
                    }
                },


                TokenizationState::HexadecimalCharacterReferenceStart => {
                    match self.peek() {
                        Some(c) if c.is_ascii_hexdigit() => {
                            self.state = TokenizationState::HexadecimalCharacterReference;
                        }
                        _ => {
                            self.errors.push(Error::AbsenceOfDigitsInNumericCharacterReference);
                            // Flush code points
                            self.state = self.return_state;
                        }
                    }
                },


                TokenizationState::HexadecimalCharacterReference => {
                    match self.peek() {
                        Some(c) if c.is_ascii_digit() => {
                            self.consume();
                            self.character_reference_code = self.character_reference_code.wrapping_mul(16);
                            self.character_reference_code += (c as u32) - 0x0030;
                        }
                        Some(c) if c.is_ascii_uppercase() && c.is_ascii_hexdigit() => {
                            self.consume();
                            self.character_reference_code = self.character_reference_code.wrapping_mul(16);
                            self.character_reference_code += (c as u32) - 0x0037;
                        }
                        Some(c) if c.is_ascii_lowercase() && c.is_ascii_hexdigit() => {
                            self.consume();
                            self.character_reference_code = self.character_reference_code.wrapping_mul(16);
                            self.character_reference_code += (c as u32) - 0x0057;
                        }
                        Some(';') => {
                            self.consume();
                            self.state = TokenizationState::NumericCharacterReferenceEnd;
                        }
                        _ => {
                            self.errors.push(Error::MissingSemicolonAfterCharacterReference);
                            self.state = TokenizationState::NumericCharacterReferenceEnd;
                        }
                    }
                },


                TokenizationState::DecimalCharacterReference => {
                    match self.peek() {
                        Some(c) if c.is_ascii_digit() => {
                            self.consume();
                            self.character_reference_code = self.character_reference_code.wrapping_mul(10);
                            self.character_reference_code += (c as u32) - 0x0030;
                        }
                        Some(';') => {
                            self.consume();
                            self.state = TokenizationState::NumericCharacterReferenceEnd;
                        }
                        _ => {
                            self.errors.push(Error::MissingSemicolonAfterCharacterReference);
                            self.state = TokenizationState::NumericCharacterReferenceEnd;
                        }
                    }
                },

                TokenizationState::NumericCharacterReferenceEnd => {
                    let mut code = self.character_reference_code;

                    if code == 0x00 {
                        self.errors.push(Error::NullCharacterReference);
                        code = 0xFFFD;
                    } else if code > 0x10FFFF {
                        self.errors.push(Error::CharacterReferenceOutsideUnicodeRange);
                        code = 0xFFFD;
                    } else if (0xD800..=0xDFFF).contains(&code) {
                        self.errors.push(Error::SurrogateCharacterReference);
                        code = 0xFFFD;
                    } else if (0xFDD0..=0xFDEF).contains(&code) || matches!(code & 0xFFFF, 0xFFFE | 0xFFFF) {
                        self.errors.push(Error::NoncharacterCharacterReference);
                    } else if code == 0x0D || ((code <= 0x1F || (0x7F..=0x9F).contains(&code)) && !matches!(code, 0x09 | 0x0A | 0x0C | 0x20)) {
                        self.errors.push(Error::ControlCharacterReference);
                        
                        code = match code {
                            0x80 => 0x20AC, // EURO SIGN (€)
                            0x82 => 0x201A, // SINGLE LOW-9 QUOTATION MARK (‚)
                            0x83 => 0x0192, // LATIN SMALL LETTER F WITH HOOK (ƒ)
                            0x84 => 0x201E, // DOUBLE LOW-9 QUOTATION MARK („)
                            0x85 => 0x2026, // HORIZONTAL ELLIPSIS (…)
                            0x86 => 0x2020, // DAGGER (†)
                            0x87 => 0x2021, // DOUBLE DAGGER (‡)
                            0x88 => 0x02C6, // MODIFIER LETTER CIRCUMFLEX ACCENT (ˆ)
                            0x89 => 0x2030, // PER MILLE SIGN (‰)
                            0x8A => 0x0160, // LATIN CAPITAL LETTER S WITH CARON (Š)
                            0x8B => 0x2039, // SINGLE LEFT-POINTING ANGLE QUOTATION MARK (‹)
                            0x8C => 0x0152, // LATIN CAPITAL LIGATURE OE (Œ)
                            0x8E => 0x017D, // LATIN CAPITAL LETTER Z WITH CARON (Ž)
                            0x91 => 0x2018, // LEFT SINGLE QUOTATION MARK (‘)
                            0x92 => 0x2019, // RIGHT SINGLE QUOTATION MARK (’)
                            0x93 => 0x201C, // LEFT DOUBLE QUOTATION MARK (“)
                            0x94 => 0x201D, // RIGHT DOUBLE QUOTATION MARK (”)
                            0x95 => 0x2022, // BULLET (•)
                            0x96 => 0x2013, // EN DASH (–)
                            0x97 => 0x2014, // EM DASH (—)
                            0x98 => 0x02DC, // SMALL TILDE (˜)
                            0x99 => 0x2122, // TRADE MARK SIGN (™)
                            0x9A => 0x0161, // LATIN SMALL LETTER S WITH CARON (š)
                            0x9B => 0x203A, // SINGLE RIGHT-POINTING ANGLE QUOTATION MARK (›)
                            0x9C => 0x0153, // LATIN SMALL LIGATURE OE (œ)
                            0x9E => 0x017E, // LATIN SMALL LETTER Z WITH CARON (ž)
                            0x9F => 0x0178, // LATIN CAPITAL LETTER Y WITH DIAERESIS (Ÿ)
                            _ => code,
                        };
                    }

                    let resolved_char = std::char::from_u32(code).unwrap_or('\u{FFFD}');

                    self.state = self.return_state;

                    return Some(HtmlToken::Character(Cow::Owned(resolved_char.to_string())));
                }




            }

        }
    }

}
