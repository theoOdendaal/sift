//: TODO: Use the below repo to learn about html escape characters,
// specifically to properly parse arch news.
// https://github.com/magiclen/html-escape

use std::io::{Read, Write};

use std::time::Instant;

fn _get_content_from_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut response = ureq::get(url).call()?;
    let body: String = response.body_mut().read_to_string()?;
    Ok(body)
}

fn _get_content_from_fs(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(std::fs::read_to_string(path)?)
}

fn _write_content_to_fs(content: String, name: &str) -> std::io::Result<()> {
    std::fs::write(format!("test_files/{}", name), content)
}

fn _get_rss_titles(url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let feed = _get_content_from_url(url)?;
    let mut xml_tokenizer = sift::xml::tokens::XmlTokenizer::new(&feed);
    let rss_feed = sift::rss::RssFeed::from_tokenizer(&mut xml_tokenizer)?;
    let list: Vec<String> = rss_feed
        .items
        .iter()
        .map(|i| i.title.clone().unwrap().clone().to_string())
        .collect();
    Ok(list)
}

fn _get_rss_titles_from_fs(file: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let feed = _get_content_from_fs(file)?;
    let mut xml_tokenizer = sift::xml::tokens::XmlTokenizer::new(&feed);
    let rss_feed = sift::rss::RssFeed::from_tokenizer(&mut xml_tokenizer)?;
    let list: Vec<String> = rss_feed
        .items
        .iter()
        .map(|i| i.title.clone().unwrap().clone().to_string())
        .collect();
    Ok(list)
}

fn _test_rss_feed() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("5JaZzppv.rss")?;

    let mut xml_tokenizer = sift::xml::tokens::XmlTokenizer::new(&content);

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

fn run_interface() -> Result<(), Box<dyn std::error::Error>> {
    let (w, h) = sift::interface::get_terminal_size()?;
    let mut buffer = sift::interface::TerminalBuffer::new(w, h);

    /*let urls: Vec<&str> = vec![
        //"https://feeds.bbci.co.uk/news/rss.xml?edition=uk",
        //"https://www.moneyweb.co.za/feed/",
        //"https://www.gov.za/news-feed",
        "https://rss.nytimes.com/services/xml/rss/nyt/World.xml",
        //"https://archlinux.org/feeds/news/",
    ];*/

    /*let feeds: Vec<sift::interface::Feed> = urls.iter().map(|f| {
        let display_name = f;
        let articles = _get_rss_titles(f).unwrap();
        sift::interface::Feed::new(&display_name, articles)
    }).collect();*/

    let files = [
        "test_files/bbc-news-uk.xml",
        "test_files/moneyweb.xml",
        "test_files/gov-za.xml",
        "test_files/nytimes-world.xml",
        "test_files/archlinux-news.xml",
    ];

    let feeds: Vec<sift::interface::Feed> = files
        .iter()
        .map(|f| {
            let display_name = f;
            let articles = _get_rss_titles_from_fs(f).unwrap();
            sift::interface::Feed::new(display_name, articles)
        })
        .collect();

    let mut subscriptions = sift::interface::Subscriptions::new(feeds);

    let mut stdin_lock = std::io::stdin().lock();
    let mut buf = [0u8; 1];

    sift::interface::draw_bottom_bar(&mut buffer)?;

    loop {
        sift::interface::draw_subscriptions(&mut buffer, 3, 3, 1, &subscriptions);
        sift::interface::draw_feed_articles(&mut buffer, 50, 3, 1, subscriptions.get_idx_mut());

        buffer.flush_to_screen()?;

        if stdin_lock.read_exact(&mut buf).is_err() {
            break;
        }

        match buf[0] {
            b'q' => break,
            b'l' => {
                subscriptions.move_in_articles();
            }
            b'h' => {
                subscriptions.move_out_articles();
            }
            b'j' => {
                subscriptions.next();
            }
            b'k' => {
                subscriptions.previous();
            }
            _ => {}
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //_write_content_to_fs(_get_content_from_url("https://feeds.bbci.co.uk/news/rss.xml?edition=uk")?, "bbc-news-uk.xml")?;
    //_write_content_to_fs(_get_content_from_url("https://www.moneyweb.co.za/feed/")?, "moneyweb.xml")?;
    //_write_content_to_fs(_get_content_from_url("https://www.gov.za/news-feed")?, "gov-za.xml")?;
    //_write_content_to_fs(_get_content_from_url("https://rss.nytimes.com/services/xml/rss/nyt/World.xml")?, "nytimes-world.xml")?;
    //_write_content_to_fs(_get_content_from_url("https://archlinux.org/feeds/news/")?, "archlinux-news.xml")?;
   

    let content = _get_content_from_fs("test_files/nytimes-world.xml")?;

    let start = Instant::now();
    let token_count = sift::xml::byte_token::XmlTokenizer::from(content.as_str()).count();
    let duration = start.elapsed();
    println!("Processed {} tokens in {:?}, using byte_token", token_count, duration);

    let start = Instant::now();
    let token_count = sift::xml::tokens::XmlTokenizer::new(&content).count();
    let duration = start.elapsed();
    println!("Processed {} tokens in {:?}, using tokens", token_count, duration);

    let start = Instant::now();
    let token_count = sift::xml::new_tokens::XmlTokenizer::from(content.as_str()).count();
    let duration = start.elapsed();
    println!("Processed {} tokens in {:?}, using new_tokens", token_count, duration);
    


    //let tokenizer = sift::xml::new_tokens::XmlTokenizer::from(content.as_str());
    //let tokenizer = sift::xml::tokens::XmlTokenizer::new(&content);
    //let tokenizer = sift::xml::byte_token::XmlTokenizer::from(content.as_str());

    /*println!("{:?}", &content);
    //for token in tokenizer.into_iter().take(50) {
    for token in tokenizer {
        println!("{}", token?);
    }
    println!("{:?}", content.len());*/

   




    /*let mut raw_guard = sift::interface::RawModeGuard::enable()?;

    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut stdout = std::io::stdout();
        let _ = write!(stdout, "\x1B[?25h\x1B[?1049l");
        let _ = stdout.flush();
        default_panic(info);
    }));

    if let Err(err) = run_interface() {
        eprintln!("Application error: {}\r", err);
        std::process::exit(1);
    }

    raw_guard.disable();*/
    Ok(())
}
