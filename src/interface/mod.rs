use std::fmt::Write;


mod ansi;
mod tb;
mod ffi;

pub use tb::TerminalBuffer;
pub use ffi::{RawModeGuard, get_terminal_size};

use crate::formats::feeds::rss::RssItem;

// FIXME: Have more strict defined areas for all sections.
// And ensure there isn't overlap between them,

// FIXME: List to SIGWINCH to dynamically detect terminal size changes.


pub const DEFAULT_FG: &str = "\x1B[39m";
pub const DEFAULT_BG: &str = "\x1B[49m";
pub const RESET_ALL: &str = "\x1B[0m";

pub const SELECTED_FG: &str = "\x1B[38;5;208m";
pub const SELECTED_BG: &str = "\x1B[48;5;208m";



// TODO: I want to also be able to navigate using indexes.
pub struct Feed<'a> {
    display_name: &'a str,
    articles: Vec<&'a str>,
    pub idx: usize,
    active: bool,
}

pub struct Subscriptions<'a> {
    pub feeds: Vec<Feed<'a>>,
    pub idx: usize,
    pub in_articles: bool,
}

impl<'a> Feed<'a> {
    pub fn new(display_name: &'a str, articles: Vec<&'a str>) -> Self {
        Self {
            display_name,
            articles,
            idx: 0,
            active: false,
        }
    }

    fn set_active(&mut self) {
        self.active = true;
    }

    fn set_inactive(&mut self) {
        self.active = false;
    }

    pub fn next_article(&mut self) -> &mut Self {
        if !self.articles.is_empty() {
            self.idx = (self.idx + 1) % self.articles.len();
        }
        self
    }

    pub fn previous_article(&mut self) -> &mut Self {
        self.idx = if self.idx == 0 {
            self.articles.len() - 1
        } else {
            self.idx - 1
        };
        self
    }
}

impl<'a> Subscriptions<'a> {
    pub fn new(feeds: Vec<Feed<'a>>) -> Self {
        Self {
            feeds,
            idx: 0,
            in_articles: false,
        }
    }

    pub fn get_idx_mut(&mut self) -> &mut Feed<'a> {
        &mut self.feeds[self.idx]
    }

    pub fn move_in_articles(&mut self) {
        self.get_idx_mut().set_active();
        self.in_articles = true;
    }

    pub fn move_out_articles(&mut self) {
        self.get_idx_mut().set_inactive();
        self.in_articles = false;
    }

    pub fn next_feed(&mut self) -> &mut Self {
        if !self.feeds.is_empty() && !self.in_articles {
            self.idx = (self.idx + 1) % self.feeds.len();
        } else if self.in_articles {
            self.get_idx_mut().next_article();
        }
        self
    }

    pub fn previous_feed(&mut self) -> &mut Self {
        if self.in_articles {
            self.get_idx_mut().previous_article();
        } else {
            self.idx = if self.idx == 0 {
                self.feeds.len() - 1
            } else {
                self.idx - 1
            };
        }
        self
    }
}

pub fn draw_subscriptions(
    buffer: &mut TerminalBuffer,
    x: u16,
    y: u16,
    y_spacing: u16,
    subscriptions: &Subscriptions,
) {
    let mut current_y = y;
    for (i, item) in subscriptions.feeds.iter().enumerate() {
        if i == subscriptions.idx {
            let mut prefix = if subscriptions.in_articles {
                String::from("  ")
            } else {
                String::from("> ")
            };
            prefix.push_str(&i.to_string());
            prefix.push_str(" - ");
            prefix.push_str(item.display_name);
            buffer.print_str(x, current_y, &prefix, SELECTED_FG, DEFAULT_BG);
        } else {
            let mut prefix = String::from("  ");
            prefix.push_str(&i.to_string());
            prefix.push_str(" - ");
            prefix.push_str(item.display_name);
            buffer.print_str(x, current_y, &prefix, "\x1B[37m", DEFAULT_BG);
        }
        current_y += y_spacing;
    }
}

// FIXME: Currently, the articles are rendered correctly when moving down,
// but when moving up it will stay in place until the the article at index
// 0 is at the top. This should not happen. It should only shift the list down
// when the idx will become smaller than the first rendered item.
pub fn draw_feed_articles(
    buffer: &mut TerminalBuffer,
    x: u16,
    y: u16,
    y_spacing: u16,
    feed: &mut Feed,
) {
    let mut current_y = y;

    let mut max_width = 0;

    // buffer height - starting y - bottom bar offset - height - padding
    let allowed_height = buffer
        .height
        .saturating_sub(y)
        .saturating_sub(3)
        .saturating_sub(1) as usize;
    let starting_idx = feed.idx.saturating_sub(allowed_height);
    let eligible_count = feed.articles.len().min(allowed_height + 1);
    let ending_idx = starting_idx + eligible_count;

    let mut formatted_line = String::with_capacity(128);

    if let Some(visible_articles) = feed.articles.get(starting_idx..ending_idx) {
        for (offset, item) in visible_articles.iter().enumerate() {
            let i = starting_idx + offset;
            let is_selected = i == feed.idx;

            let prefix = if is_selected && feed.active {
                "> "
            } else {
                "  "
            };

            formatted_line.clear();

            let _ = write!(formatted_line, "{prefix}{i} - {item}");

            let width = x + formatted_line.len() as u16;
            if width > max_width {
                max_width = width
            };

            let fg_color = if is_selected { SELECTED_FG } else { "\x1B[37m" };
            let bg_color = DEFAULT_BG;

            buffer.print_str_padded(
                x,
                current_y,
                &formatted_line,
                fg_color,
                bg_color,
                width.max(buffer.previous_max_width),
            );

            current_y += y_spacing;
        }
    }

    if max_width > buffer.previous_max_width {
        buffer.previous_max_width = max_width
    };

    let last_drawn_y = current_y.saturating_sub(y_spacing);
    if last_drawn_y < buffer.previous_max_height {
        for padded_y in current_y..=buffer.previous_max_height {
            buffer.print_str_padded(x, padded_y, " ", DEFAULT_FG, DEFAULT_BG, max_width);
        }
    }
    buffer.previous_max_height = last_drawn_y;
}

fn draw_bar(text: &str, buffer: &mut TerminalBuffer, y: u16) -> std::io::Result<()> {
    buffer.print_str_padded(1, y, text, DEFAULT_FG, SELECTED_BG, buffer.width); 
    Ok(())
}

pub fn draw_article_view(buffer: &mut TerminalBuffer, x: u16, y: u16, y_spacing: u16, item: &RssItem) {
    let formatted =format!("{:#}", item);

    for (idx, line) in formatted.lines().enumerate() {
        buffer.print_str(x, y + y_spacing * idx as u16, line, DEFAULT_FG, DEFAULT_BG);
    }


}


// FIXME: Add instructions
pub fn draw_bottom_bar(buffer: &mut TerminalBuffer) -> std::io::Result<()> {
    let instructions = [
        "Theo luv Andrea all ze beans cheese much",
        "Q: Quit",
        "q: Back",
        "h: Left",
        "l: Right",
        "j: Down",
        "k: Up",
    ];

    let bar_text = instructions.join(" ");

    draw_bar(&bar_text, buffer, buffer.height - 2)?;


    Ok(())
}
