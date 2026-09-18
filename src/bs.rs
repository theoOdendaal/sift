// byte scanner

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

/// All of the below consume fn assume
/// that the byte at self.pos represent the start
/// of the slice that has to be consumed. The fn's will
/// also not consume the byte that broke the consumption.
impl<'a> Scanner<'a> {
    
    #[inline]
    fn consume_until(&mut self, stop: impl Fn(u8) -> bool) -> Option<&'a [u8]> {
        let start = self.pos;
        let len = self.bytes[start..].iter().position(|&b| stop(b))?;
        self.pos = start + len;
        Some(&self.bytes[start..self.pos])
    }

    #[inline]
    fn consume_until_sequence(&mut self, delim: &[u8]) -> Option<&'a [u8]> {
        let start = self.pos;
        let len = self.bytes[start..].windows(delim.len()).position(|w| w == delim)?;
        self.pos = start + len;
        Some(&self.bytes[start..self.pos])
    }

    #[inline]
    fn consume_until_or_eof(&mut self, stop: impl Fn(u8) -> bool) -> &'a [u8] {
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

    #[inline]
    pub fn consume_comment(&mut self) -> Result<&'a [u8], Error> {
        if let Some(value) = self.consume_until_sequence(b"-->") {
            Ok(value)
        } else {
            Err(Error::UnexpectedEndOfFile)
        }
    }

    #[inline]
    pub fn consume_cdata(&mut self) -> Result<&'a [u8], Error> {
        if let Some(value) = self.consume_until_sequence(b"]]>") {
            Ok(value)
        } else {
            Err(Error::UnexpectedEndOfFile)
        }
    }

}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_advance_past_whitespaces1() {
        let input = b" theo";
        let mut scanner = Scanner::from(input.as_slice());
        scanner.advance_past_whitespaces();
        assert_eq!(scanner.pos, 1)
    }

    #[test]
    fn test_advance_past_whitespaces2() {
        let input = b"  theo";
        let mut scanner = Scanner::from(input.as_slice());
        scanner.advance_past_whitespaces();
        assert_eq!(scanner.pos, 2)
    }

    #[test]
    fn test_advance_past_whitespaces3() {
        let input = b"theo";
        let mut scanner = Scanner::from(input.as_slice());
        scanner.advance_past_whitespaces();
        assert_eq!(scanner.pos, 0)
    }

    #[test]
    fn test_advance_past_whitespaces4() {
        let input = b"      \t      \n    \r  ";
        let mut scanner = Scanner::from(input.as_slice());
        scanner.advance_past_whitespaces();
        assert_eq!(scanner.pos, input.len())
    }

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
    fn test_consume_attribute_name1() {}
    
    #[test]
    fn test_consume_quoted_attribute_value1() {}

    #[test]
    fn test_consume_unquoted_attribute_value1() {}
    
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
    fn test_consume_comment1() {
        let input = b"hello world, my name is theo-->";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_comment();
        assert_eq!(slice, Ok(b"hello world, my name is theo".as_slice()))
    }

    #[test]
    fn test_consume_comment2() {
        let input = b"hello world, my name is theo--";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_comment();
        assert_eq!(slice, Err(Error::UnexpectedEndOfFile))
    }
    
    #[test]
    fn test_consume_cdata1() {
        let input = b"hello world, my name is theo]]>";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_cdata();
        assert_eq!(slice, Ok(b"hello world, my name is theo".as_slice()))
    }

    #[test]
    fn test_consume_cdata2() {
        let input = b"hello world, my name is theo] ]>";
        let mut scanner = Scanner::from(input.as_slice());
        let slice = scanner.consume_cdata();
        assert_eq!(slice, Err(Error::UnexpectedEndOfFile))
    }

}
