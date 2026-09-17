#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedEndOfFile,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEndOfFile => {
                write!(f, "Unexpected EOF")
            }
        }
    }
}

impl std::error::Error for Error {}
pub struct Scanner<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> From<&'a [u8]> for Scanner<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self { bytes: value, pos: 0 }
    }
}

// All of the below consume fn assume
// that the byte at self.pos represent the start
// of the slice that has to be consumed.
impl<'a> Scanner<'a> {
    
    #[inline]
    pub fn consume_until(&mut self, stop: impl Fn(u8) -> bool) -> Option<&'a [u8]> {
        let start = self.pos;
        let len = self.bytes[start..].iter().position(|&b| stop(b))?;
        self.pos = start + len;
        Some(&self.bytes[start..self.pos])
    }

    #[inline]
    pub fn consume_until_or_eof(&mut self, stop: impl Fn(u8) -> bool) -> &'a [u8] {
        let start = self.pos;
        let len = match self.bytes[start..].iter().position(|&b| stop(b)) {
            Some(idx) => idx,
            None => self.bytes.len(),
        };
        self.pos = start + len;
        &self.bytes[start..self.pos]
    }
    
    #[inline]
    pub fn advance_past_whitespaces(&mut self) {
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
    pub fn consume_tag_name(&mut self) -> Result<&'a [u8], Error> {
        if let Some(value) = self.consume_until(|b| b.is_ascii_whitespace() || matches!(b,  b'>' | b'/' | b'?')) {
            Ok(value)
        } else {
            Err(Error::UnexpectedEndOfFile)
        }
    }
    
    //#[inline]
    //fn consume_processing_instruction_data(&mut self) -> Result<&'a [u8], Error>
    
    #[inline]
    pub fn consume_attribute_name(&mut self) -> Result<&'a [u8], Error> {
        if let Some(value) = self.consume_until(|b| b.is_ascii_whitespace() || matches!(b, b'=' | b'>' | b'/' | b'?')) {
            Ok(value)
        } else {
            Err(Error::UnexpectedEndOfFile)
        }
    }
    
    #[inline]
    pub fn consume_quoted_attribute_value(&mut self, quote_char: u8) -> Result<&'a [u8], Error> {
        if let Some(value) = self.consume_until(|b| b == quote_char) {
            Ok(value)
        } else {
            Err(Error::UnexpectedEndOfFile)
        }
    }
    #[inline] 
    pub fn consume_unquoted_attribute_value(&mut self) -> Result<&'a [u8], Error> {
        if let Some(value) = self.consume_until(|b| b.is_ascii_whitespace() || matches!(b, b'>' | b'/')) {
            Ok(value)
        } else {
            Err(Error::UnexpectedEndOfFile)
        }
    }

    #[inline]
    pub fn consume_text(&mut self) -> &'a [u8] {
        self.consume_until_or_eof(|b| b == b'<')
    }

    //#[inline]
    //fn consume_comment(&mut self) -> Result<&'a [u8], Error>

    //#[inline]
    //fn consume_cdata(&mut self) -> Result<&'a [u8], Error>

    //#[inline]
    //fn consume_entity_declaration(&mut self) -> Result<&'a [u8], Error>

}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_advance_past_whitespaces1() {}

    #[test]
    fn test_consume_tag_name1() {
        let input = b"theo>";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_tag_name();
        assert_eq!(slice, Ok(b"theo".as_slice()))
    }

    #[test]
    fn test_consume_tag_name2() {
        let input = b"<theo >";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_tag_name();
        assert_eq!(slice, Ok(b"<theo".as_slice()))
    }

    #[test]
    fn test_consume_tag_name3() {
        let input = b"<theo";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_tag_name();
        assert_eq!(slice, Err(Error::UnexpectedEndOfFile))
    }
    
    #[test]
    fn test_consume_processing_instruction_data() {}
    
    #[test]
    fn test_consume_attribute_name() {}
    
    #[test]
    fn test_consume_quoted_attribute_value() {}

    #[test]
    fn test_consume_unquoted_attribute_value() {}
    
    #[test]
    fn test_consume_text1() {
        let input = b"hello world, my name is theo>      \t<";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_text();
        assert_eq!(slice, b"hello world, my name is theo>      \t".as_slice())
    }

    #[test]
    fn test_consume_text2() {
        let input = b"hello world, my name is theo>      \t \n \r";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_text();
        assert_eq!(slice, b"hello world, my name is theo>      \t \n \r".as_slice())
    }
    
    #[test]
    fn test_consume_comment() {}
    
    #[test]
    fn test_consume_cdata() {}
    
    #[test]
    fn test_consume_entity_declaration() {}

}
