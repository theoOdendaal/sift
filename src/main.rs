// TODO: Use the below repo to learn about html escape characters,
// specifically to properly parse arch news.
// https://github.com/magiclen/html-escape

use std::io::Read;

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
        "https://feeds.bbci.co.uk/news/rss.xml?edition=uk",
        "https://www.moneyweb.co.za/feed/",
        "https://archlinux.org/feeds/news/",
    ];

    let articles: Vec<Vec<&str>> = vec![
        vec!["a1", "b1", "c1"],
        vec!["a2", "b2", "c2"],
        vec!["a3", "b3", "c3"],
    ];

    let mut feeds_list = sift::interface::SelectableList::new(feeds);
    let mut articles_list0 = sift::interface::SelectableList::new(articles[0].clone());
    let mut articles_list1 = sift::interface::SelectableList::new(articles[1].clone());
    let mut articles_list2 = sift::interface::SelectableList::new(articles[2].clone());

    let mut stdin_lock = std::io::stdin().lock();
    let mut buf = [0u8; 1];

    let mut current_panel = 0;

    sift::interface::draw_bottom_bar(&mut buffer)?;

    loop {
        sift::interface::draw_list(&mut buffer, 5, 3, 1, &mut feeds_list);

        let article_list = match feeds_list.idx() {
            0 => &mut articles_list0,
            1 => &mut articles_list1,
            2 => &mut articles_list2,
            _ => unreachable!(),
        };

        sift::interface::draw_list(&mut buffer, 90, 3, 1, article_list);

        buffer.flush_to_screen()?;

        if stdin_lock.read_exact(&mut buf).is_err() {
            break;
        }

        match buf[0] {
            b'q' => break,
            b'h' => {
                if current_panel == 1 {
                    current_panel = 0;
                } else {
                    current_panel = 1;
                }
            }
            b'l' => {
                if current_panel == 0 {
                    current_panel = 1
                } else {
                    current_panel = 0;
                }
            }
            b'j' => {
                if current_panel == 0 {
                    feeds_list.next_item();
                } else {
                    article_list.next_item();
                }
            }
            b'k' => {
                if current_panel == 0 {
                    feeds_list.previous_item();
                } else {
                    article_list.previous_item();
                }
            }
            _ => {}
        }
    }

    Ok(())
}
