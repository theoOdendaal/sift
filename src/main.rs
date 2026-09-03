// TODO: Use the below repo to learn about html escape characters,
// specifically to properly parse arch news.
// https://github.com/magiclen/html-escape

use std::io::Read;

fn get_content(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut response = ureq::get(url).call()?;
    let body: String = response.body_mut().read_to_string()?;
    Ok(body)
}

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

fn get_rss_titles(url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let feed = get_content(url)?;
    let mut xml_tokenizer = sift::xml::tokens::Tokenizer::new(&feed);
    let rss_feed = sift::rss::RssFeed::from_tokenizer(&mut xml_tokenizer)?;
    let list: Vec<String> = rss_feed
        .items
        .iter()
        .map(|i| i.title.clone().take().unwrap().to_owned().to_string())
        .collect();
    Ok(list)
}

fn _test_rss_feed() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _raw_guard = sift::interface::RawModeGuard::enable()?;

    let (w, h) = sift::interface::get_terminal_size()?;
    let mut buffer = sift::interface::TerminalBuffer::new(w, h);

    let feeds: Vec<&str> = vec![
        //"https://feeds.bbci.co.uk/news/rss.xml?edition=uk",
        //"https://www.moneyweb.co.za/feed/",
        "https://archlinux.org/feeds/news/",
        "https://archlinux.org/feeds/news/",
        "https://archlinux.org/feeds/news/",
    ];

    let feed_titles: Vec<Vec<String>> = feeds
        .iter()
        .map(|url| get_rss_titles(url))
        .collect::<Result<Vec<_>, _>>()?;
    
    let articles: Vec<sift::interface::VerticalList> = feed_titles
        .iter()
        .map(|titles| {
            let items: Vec<&str> = titles.iter().map(|s| s.as_str()).collect();
            sift::interface::VerticalList::new(items, false)
        })
        .collect();

    //let mut feeds_list = sift::interface::VerticalList::new(&feeds, true);
    let mut panels = sift::interface::HorizontalList::new(articles);
    panels.get_mut_idx().set_active();

    let mut stdin_lock = std::io::stdin().lock();
    let mut buf = [0u8; 1];

    sift::interface::draw_bottom_bar(&mut buffer)?;

    loop {
        
        sift::interface::draw_horizontal_list(&mut buffer, 5, 3, 50, 1, &mut panels);

        //sift::interface::draw_list(&mut buffer, 90, 3, 2, article_list);

        buffer.flush_to_screen()?;

        if stdin_lock.read_exact(&mut buf).is_err() {
            break;
        }

        match buf[0] {
            b'q' => break,
            b'l' => { panels.next_item(); },
            b'h' => { panels.previous_item(); },
            b'j' => { panels.get_mut_idx().next_item(); },
            b'k' => { panels.get_mut_idx().previous_item(); },
            _ => {}
        }
    }

    Ok(())
}
