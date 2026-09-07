// https://html.spec.whatwg.org/#tokenization

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedNullCharacter,

}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedNullCharacter => write!(f, "Unexpected null character encountered"),
        }
    }
}


impl std::error::Error for Error {}

#[derive(Debug, PartialEq)]
pub struct Doctype<'a> {
    name: Option<&'a [u8]>,
    public_identifier: Option<&'a [u8]>,
    system_identifier: Option<&'a [u8]>,
    force_quirks: bool
}

#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    name: &'a [u8],
    self_closing: bool,
    attributes: Vec<&'a [u8]>,
}


#[derive(Debug, PartialEq)]
pub enum HtmlToken<'a> {
    
    Doctype(Doctype<'a>),

    StartTag(Tag<'a>),

    EndTag(Tag<'a>),

    Comment(&'a [u8]),

    Character(&'a [u8]),

}

enum HtmlState {
    Data,

    RcData,

    RawText,

    ScriptData,

    PlainText,

}

pub struct HtmlTokenizer<'a> {
    bytes: &'a [u8],
    state: HtmlState,
    return_state: HtmlState,
    pos: usize,
    mark: usize,
    current_tag_buffer: Option<Tag<'a>>,
    last_start_tag_name : Option<&'a [u8]>,

}

impl<'a> From<&'a [u8]> for HtmlTokenizer<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self {
            bytes: value,
            state: HtmlState::Data,
            return_state: HtmlState::Data,
            pos: 0,
            mark: 0,
            current_tag_buffer: None,
            last_start_tag_name: None,
        } 
    }
}

impl<'a> HtmlTokenizer<'a> {

    fn override_state(&mut self, state: HtmlState) {
        self.state = state;
    }

    fn set_last_start_tag_name(&mut self, name: &'a [u8]) {
        self.last_start_tag_name = Some(name);
    }

    #[inline]
    fn is_appropriate_end_tag(&self, name: &'a [u8]) -> bool {
        if let Some(previous) = self.last_start_tag_name {
            return previous.eq_ignore_ascii_case(name)
        }
        false
    }
    
    // https://html.spec.whatwg.org/#rcdata-state
    // https://html.spec.whatwg.org/#rcdata-less-than-sign-state
    // https://html.spec.whatwg.org/#rcdata-end-tag-open-state
    // https://html.spec.whatwg.org/#rcdata-end-tag-name-state
    fn consume_rcdata(&mut self) -> Result<&'a [u8], Error> {
        
        let mut len = self.pos;

        while len < self.bytes.len() {
            let remaining = &self.bytes[len..];

            if remaining.starts_with(b"</") {

                match self.bytes[len+2..].iter().position(|&b| b == b'>') {
                    Some(idx) => if self.is_appropriate_end_tag(&self.bytes[len+2..idx]) {
                         
                        break;
                    }
                    None => { len += 2; },
                }
            }
            len += 1;

        }

        let data = &self.bytes[self.pos..len];
        self.pos = len;

        return Ok(data);


    }

}

impl<'a> Iterator for HtmlTokenizer<'a> {
    type Item = Result<HtmlToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.bytes.len() {
            return None;
        }

        match self.state {
            
            HtmlState::Data => {
                unimplemented!("Data")
            },

            HtmlState::RcData => {
                match self.consume_rcdata() {
                    Ok(data) => { return Some(Ok(HtmlToken::Character(data))); }
                    Err(e) => return  Some(Err(e)),
                }
            },

            HtmlState::RawText => {
                unimplemented!("RawText")
            },

            HtmlState::ScriptData => {
                unimplemented!("ScriptData")
            },

            HtmlState::PlainText => {
                unimplemented!("PlainText")
            },


        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! assert_html5lib_tests_tokenize {
        ($name:ident, $state:expr, $tag:expr, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut tokenizer = HtmlTokenizer::from($input.as_slice());
                tokenizer.override_state($state);
                tokenizer.set_last_start_tag_name($tag);

                let tokens: Vec<Result<HtmlToken, Error>> = tokenizer.into_iter().collect();
                assert_eq!($expected, tokens.as_slice());
            }
        };
    }
    
    // contentModelFlag.test
    assert_html5lib_tests_tokenize!(test_rcdata1, HtmlState::RcData, b"xmp", b"foo</xmp>", &[Ok(HtmlToken::Character(b"foo")), Ok(HtmlToken::EndTag(Tag { name: b"xmp", self_closing: true, attributes: Vec::new() }))]);
    assert_html5lib_tests_tokenize!(test_rcdata2, HtmlState::RcData, b"xmp", b"foo</xMp>", &[Ok(HtmlToken::Character(b"foo")), Ok(HtmlToken::EndTag(Tag { name: b"xmp", self_closing: true, attributes: Vec::new() }))]);
    assert_html5lib_tests_tokenize!(test_rcdata3, HtmlState::RcData, b"xmp", b"foo</xmp ", &[Ok(HtmlToken::Character(b"foo"))]);
    assert_html5lib_tests_tokenize!(test_rcdata4, HtmlState::RcData, b"xmp", b"foo</xmp", &[Ok(HtmlToken::Character(b"foo</xmp"))]);
    assert_html5lib_tests_tokenize!(test_rcdata5, HtmlState::RcData, b"xmp", b"foo</xmp/", &[Ok(HtmlToken::Character(b"foo"))]);
    assert_html5lib_tests_tokenize!(test_rcdata6, HtmlState::RcData, b"xmp", b"foo</xmp<", &[Ok(HtmlToken::Character(b"foo</xmp<"))]);



}
