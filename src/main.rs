//: TODO: Use the below repo to learn about html escape characters,
// specifically to properly parse arch news.
// https://github.com/magiclen/html-escape

use std::io::{Read, Write};

use std::time::Instant;

const URL_SUBSCRIPTIONS: [&str;5] = [
    "https://feeds.bbci.co.uk/news/rss.xml?edition=uk",
    "https://www.moneyweb.co.za/feed/",
    "https://www.gov.za/news-feed",
    "https://rss.nytimes.com/services/xml/rss/nyt/World.xml",
    "https://archlinux.org/feeds/news/",
];

const FS_SUBSCRIPTIONS: [&str;5] = [
    "test_files/bbc-news-uk.xml",
    "test_files/moneyweb.xml",
    "test_files/gov-za.xml",
    "test_files/nytimes-world.xml",
    "test_files/archlinux-news.xml",
];

fn _get_content_from_url(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut response = ureq::get(url).call()?;
    let body: Vec<u8> = response.body_mut().read_to_vec()?;
    Ok(body)
}

fn _write_content_to_fs(content: String, name: &str) -> std::io::Result<()> {
    std::fs::write(format!("test_files/{}", name), content)
}

fn retrieve_rss_bytes_from_fs(paths: &[&str]) -> Result<Vec<Vec<u8>>, std::io::Error> {
    paths.iter().map(|p| std::fs::read(p)).collect()
}

fn parse_rss_feeds<'a>(byte_feeds: &'a [Vec<u8>]) -> Result<Vec<sift::rss::RssParser<'a>>, Box<dyn std::error::Error>> {
    let mut parsed_feeds = Vec::new(); 

    for feed in byte_feeds {
        let tokenizer = sift::xml::tokens::XmlTokenizer::from(feed.as_slice());
        let mut feed = sift::rss::RssParser::new();
        for token in tokenizer {
            feed.handle_token(token?)?;
        }
        parsed_feeds.push(feed);
    }
    Ok(parsed_feeds)
}

fn run_interface() -> Result<(), Box<dyn std::error::Error>> {
    let (w, h) = sift::interface::get_terminal_size()?;
    let mut buffer = sift::interface::TerminalBuffer::new(w, h);


    let byte_feeds = retrieve_rss_bytes_from_fs(FS_SUBSCRIPTIONS.as_slice())?;

    let parsed_feeds = parse_rss_feeds(byte_feeds.as_slice())?;

    let feeds: Vec<sift::interface::Feed> = parsed_feeds
        .iter()
        .filter_map(|f| {
            f.feed.as_ref().map(|feed| {
                let display_name = feed
                    .get_channel_title()
                    .unwrap_or_else(|| "Untitled Feed");

                let articles = feed.get_item_titles();

                sift::interface::Feed::new(display_name, articles)
            })
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

    /*let start = Instant::now(); 
    let content = std::fs::read("test_files/bbc-news-uk.xml")?;
    let tokenizer = sift::xml::tokens::XmlTokenizer::from(content.as_slice());
    let mut parser = sift::rss::RssParser::new();
    for token in tokenizer {
        parser.handle_token(token?)?;
    }
    println!("{:?}", parser.feed);
    let duration = start.elapsed();
    println!("Constructed tree using byte tokens in {:?}", duration);*/
    
    
    let mut raw_guard = sift::interface::RawModeGuard::enable()?;

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

    raw_guard.disable();
    Ok(())
}
