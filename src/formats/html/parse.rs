use crate::formats::html::tokens::{HtmlToken, HtmlTokenizer};


pub struct DisplayHtmlTokenStream<'a> {
    tokenizer: HtmlTokenizer<'a>,
}

impl<'a> From<HtmlTokenizer<'a>> for DisplayHtmlTokenStream<'a> {
   fn from(value: HtmlTokenizer<'a>) -> Self {
        Self { tokenizer: value }
    } 
    
}

impl<'a> DisplayHtmlTokenStream<'a> {

    pub fn render_tokens<W: std::io::Write>(&mut self, mut writer: W) -> std::io::Result<()> {
        
        let mut ignore_tokens = false;
        let mut after_title = false;
        let mut indent = 0;

        for token_result in self.tokenizer.by_ref() {

            let token = match token_result {
                Ok(t) => t,
                Err(_) => continue,
            };

            match token {
                // when to start
                HtmlToken::StartTag(name) if name == b"title" => after_title = true,

                // Tokens to ignore 
                HtmlToken::StartTag(name) if name == b"script" => ignore_tokens = true,
                HtmlToken::EndTag(name) if name == b"script" => ignore_tokens = false,
                HtmlToken::StartTag(name) if name == b"style" => ignore_tokens = true,
                HtmlToken::EndTag(name) if name == b"style" => ignore_tokens = false,

                // Formatting
                HtmlToken::StartTag(name) if name == b"ul" => indent += 1,
                HtmlToken::EndTag(name) if name == b"ul" => indent -= 1,



                HtmlToken::Text(bytes) if !ignore_tokens && after_title => {
                    let text = String::from_utf8_lossy(bytes);
                    
                    // Collapse consecutive whitespaces.
                    let words: Vec<&str> = text.split_whitespace().collect();
                    if words.is_empty() {
                        continue;
                    }

                    let cleaned_text = words.join(" ");
                    writeln!(writer, "{}{}", "    ".repeat(indent), cleaned_text)?;

                },


                _ => {}

            }




        }
        writeln!(writer)?;
        Ok(())
    }

}
