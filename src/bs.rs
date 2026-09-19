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
/// of the slice that has to be consumed.
///
/// Closing sequences (e.g., --> or ]]>) are consumed,
/// however break bytes (e.g, >, =, ") are not consumed
/// as they might have different interpretation.
impl<'a> Scanner<'a> {

    #[inline]
    pub fn consume_byte(&mut self) {
        self.pos += 1;
    }

    #[inline]
    pub fn consume_byte_if(&mut self, byte: u8) -> bool {
        if self.is_byte(byte) {
            self.pos +=1;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn consume_n_bytes(&mut self, n: usize) {
        self.pos += n;
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    #[inline]
    pub fn get_byte(&self) -> u8 {
        self.bytes[self.pos]
    }
    
    // Return a reference to `bytes` from self.bytes,
    // if `bytes` is the next few bytes.
    #[inline]
    pub fn get_bytes_if(&self, bytes: &[u8]) -> Option<&'a [u8]> {
        if self.starts_with(bytes) {
            let value = &self.bytes[self.pos..self.pos+bytes.len()];
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_checked_nth_byte(&self, n: usize) -> Option<&'a u8> {
        if self.pos + n < self.bytes.len() {
            Some(&self.bytes[self.pos+n])
        } else {
            None
        }
    }

    #[inline]
    pub fn is_byte(&self, byte: u8) -> bool {
        self.bytes.get(self.pos) == Some(&byte)
    }

    #[inline]
    pub fn starts_with(&self, bytes: &[u8]) -> bool {
        self.bytes[self.pos..].starts_with(bytes)
    }
    
    #[inline]
    pub fn consume_until(&mut self, stop: impl Fn(u8) -> bool) -> Option<&'a [u8]> {
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
    pub fn advance_past_whitespaces(&mut self) {
        let mut len = self.pos;
        while len < self.bytes.len() {
            match self.bytes[len] {
                b' ' | b'\t' | b'\n' | b'\r' => len += 1,
                _ => break,
            }
        }
        self.pos = len;
    }
    
    #[inline]
    pub fn consume_tag_name(&mut self) -> Result<&'a [u8], Error> {
        let mut len = self.pos;
        while len < self.bytes.len() {
            match self.bytes[len] {
                b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/' | b'?' => break,
                _ => len += 1
            }
        }

        if len >= self.bytes.len() {
            return Err(Error::UnexpectedEndOfFile);
        }

        let name = &self.bytes[self.pos..len];
        self.pos = len;
        Ok(name)
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
        let mut len = self.pos;
        while len < self.bytes.len() && self.bytes[len] != b'<' {
            len += 1;
        }
        let text = &self.bytes[self.pos..len];
        self.pos = len;
        text
    }

    #[inline]
    pub fn consume_comment(&mut self) -> Result<&'a [u8], Error> {
        if let Some(value) = self.consume_until_sequence(b"-->") {
            self.pos += 3;
            Ok(value)
        } else {
            Err(Error::UnexpectedEndOfFile)
        }
    }

    #[inline]
    pub fn consume_cdata(&mut self) -> Result<&'a [u8], Error> {
        if let Some(value) = self.consume_until_sequence(b"]]>") {
            self.pos += 3;
            Ok(value)
        } else {
            Err(Error::UnexpectedEndOfFile)
        }
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    //fn consume_byte()

    //fn consume_byte_if()

    //pub fn consume_n_bytes()

    //pub fn is_eof()

    //pub fn get_byte()
   
    #[test]
    fn test_get_bytes_if1() {
        let input = b"hello darkness my old friend";
        let scanner = Scanner::from(input.as_slice());
        let bytes = scanner.get_bytes_if(b"hello darkness");
        assert_eq!(bytes, Some(b"hello darkness".as_slice()))
    }

    #[test]
    fn test_get_bytes_if2() {
        let input = b"hello darkness my old friend";
        let scanner = Scanner::from(input.as_slice());
        let bytes = scanner.get_bytes_if(b"theo");
        assert_eq!(bytes, None)
    }

    //pub fn get_checked_nth_byte(&self, n: usize) -> Option<&'a u8> {

    //pub fn is_byte(&self, byte: u8) -> bool {

    //pub fn starts_with(&self, bytes: &[u8]) -> bool {

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
