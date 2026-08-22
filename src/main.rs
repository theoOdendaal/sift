// TODO: Use the below repo to learn about html escape characters,
// specifically to properly parse arch news.
// https://github.com/magiclen/html-escape

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

    let content = _arch_rss_content()?;
    //let content = _bbc_news_content()?;
    
    let mut xml_tokenizer = sift::xml::tokens::Tokenizer::new(&content);

    let rss_feed = sift::rss::RssFeed::from_tokenizer(&mut xml_tokenizer)?;

    println!("{:?}", rss_feed.channel.title);
    println!("{:?}", rss_feed.channel.link);
    println!("{:?}", rss_feed.channel.description);
    println!("{:?}", rss_feed.channel.language);
   
    let body = rss_feed.items[0].follow_link()?;
     
    /*let html_tokenizer = sift::html::tokens::HtmlTokenizer::new(&body);
    for t in html_tokenizer {
        println!("{:?}", t);
    }*/

    Ok(())
}
