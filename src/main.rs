// TODO: Use the below repo to learn about html escape characters,
// specifically to properly parse arch news.
// https://github.com/magiclen/html-escape


use std::io::{Read, Write};

fn _bbc_news_content() -> Result<String, Box<dyn std::error::Error>> {
    let mut response = ureq::get("https://feeds.bbci.co.uk/news/rss.xml?edition=uk").call()?;
    let body: String = response.body_mut().read_to_string()?;
    Ok(body)
}

fn _moneyweb_content() -> Result<String, Box<dyn std::error::Error>> {
    let mut response = ureq::get("https://www.moneyweb.co.za/feed/").call()?;
    let body: String = response.body_mut().read_to_string()?;
    Ok(body)
}

fn _arch_rss_content() -> Result<String, Box<dyn std::error::Error>> {
    //let mut response = ureq::get("https://archlinux.org/feeds/news/").call()?;
    //let body: String = response.body_mut().read_to_string()?;

    let content = std::fs::read_to_string("5JaZzppv.rss")?;
    //let content = std::fs::read_to_string("kiJlNXq5.rss")?;

    Ok(content)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Entering raw mode
    let _raw_guard = sift::interface::RawModeGuard::enable()?;

    sift::interface::draw_box(5, 3, 40, 10, " Integrated TUI ")?;

    let mut stdout = std::io::stdout();
    write!(
        stdout,
        "\x1B[5;7H\x1B[1;32mRaw mode + Alternate Screen active!\x1B[0m"
    )?;
    write!(stdout, "\x1B[7;7HPress 'q' to exit...")?;
    stdout.flush()?;

    let mut stdin = std::io::stdin();
    let mut buf = [0u8; 1];

    // Read bytes instantly without needing Enter
    loop {
        if stdin.read_exact(&mut buf).is_ok() && buf[0] == b'q' {
            break;
        }
    }

    /*
    let content = _arch_rss_content()?;
    //let content = _bbc_news_content()?;

    let mut xml_tokenizer = sift::xml::tokens::Tokenizer::new(&content);

    let rss_feed = sift::rss::RssFeed::from_tokenizer(&mut xml_tokenizer)?;

    println!("{:?}", rss_feed.channel.title);
    println!("{:?}", rss_feed.channel.link);
    println!("{:?}", rss_feed.channel.description);
    println!("{:?}", rss_feed.channel.language);

    for item in rss_feed.items {
        println!("{:?}", item.title);
    }

    //let body = rss_feed.items[0].follow_link()?;
    //std::fs::write("test.html", &body)?;

    let body = std::fs::read_to_string("test.html")?;

    let mut html_tokenizer = sift::html::tokens::HtmlTokenizer::new(&body);
    //let mut tokens = Vec::new();
    while let Some(token) = html_tokenizer.next_token() {
        println!("{:?}", token);
        if token == sift::html::tokens::HtmlToken::EndOfFile {
            break;
        }
        //tokens.push(token);
    }
    */

    Ok(())
}
