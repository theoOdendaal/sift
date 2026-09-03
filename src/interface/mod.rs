use std::io::Write;

use crate::interface::ffi::{ECHO, ICANON, ICRNL, ISIG, IXON, OPOST, STDIN_FILENO, TCSAFLUSH, tcsetattr};

mod ffi;

// FIXME: Have more strict defined areas for all sections.
// And ensure there isn't overlap between them,

// FIXME: List to SIGWINCH to dynamically detect terminal size changes.

// FIXME: All the unsafe code should be moved to the ffi file.

pub const DEFAULT_FG: &'static str = "\x1B[39m";
pub const DEFAULT_BG: &'static str = "\x1B[49m";
pub const RESET_ALL: &'static str = "\x1B[0m";

pub const SELECTED_FG: &'static str = "\x1B[38;5;208m";
pub const SELECTED_BG: &'static str = "\x1B[48;5;208m";

pub use ffi::get_terminal_size;


pub struct RawModeGuard {
    original: Option<ffi::Termios>,
}

impl RawModeGuard {
    pub fn enable() -> std::io::Result<Self> {
        let mut original: ffi::Termios = unsafe { std::mem::zeroed() };
        // Retrieves and store the current control attributes
        // and parameters of the terminal, in order to
        // be able to revert back to original state.
        if unsafe { ffi::tcgetattr(ffi::STDIN_FILENO, &mut original) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut raw = original;
        raw.c_lflag &= !(ECHO | ICANON | ISIG);
        raw.c_iflag &= !(IXON | ICRNL);
        raw.c_oflag &= !OPOST;
        if unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        // Switch to alternate screen.
        let mut stdout = std::io::stdout();
        write!(stdout, "\x1B[?1049h\x1B[2J\x1B[?25l")?;
        stdout.flush()?;

        Ok(RawModeGuard { original: Some(original) })
    }

    pub fn disable(&mut self) {
        if let Some(original) = self.original.take() {
            let mut stdout = std::io::stdout();
            let _ = write!(stdout, "\x1B[?25h\x1B[?1049l");
            let _ = stdout.flush();

            unsafe {
                tcsetattr(STDIN_FILENO, TCSAFLUSH, &original);
            }
        }
    }

}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        self.disable(); 
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg_colour: &'static str,
    pub bg_colour: &'static str,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg_colour: DEFAULT_FG,
            bg_colour: DEFAULT_BG,
        }
    }
}

pub struct TerminalBuffer {
    width: u16,
    height: u16,
    front: Vec<Cell>,
    back: Vec<Cell>,
    previous_max_width: u16,
    previous_max_height: u16,
}

impl TerminalBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            front: vec![Cell::default(); size],
            back: vec![Cell::default(); size],
            previous_max_width: 0,
            previous_max_height: 0,
        }
    }

    pub fn set_cell(&mut self, x: u16, y: u16, ch: char, fg: &'static str, bg: &'static str) {
        if x == 0 || y == 0 || x > self.width || y > self.height {
            return;
        }
        let index = ((y - 1) as usize) * (self.width as usize) + ((x - 1) as usize);
        self.back[index] = Cell {
            ch,
            fg_colour: fg,
            bg_colour: bg,
        };
    }

    pub fn print_str(&mut self, x: u16, y: u16, text: &str, fg: &'static str, bg: &'static str) {
        for (i, ch) in text.chars().enumerate() {
            self.set_cell(x + i as u16, y, ch, fg, bg);
        }
    }
    
    pub fn print_str_padded(&mut self, x: u16, y: u16, text: &str, fg: &'static str, bg: &'static str, max_len: u16) {
        
        let mut char_count = 0;

        for (i, ch) in text.chars().enumerate() {
            self.set_cell(x + i as u16, y, ch, fg, bg);
            char_count += 1;
        }

        for i in char_count..max_len {
            self.set_cell(x + i as u16, y, ' ', fg, bg);
        }
    }

    /// Diff back and front buffers and output changed cells to stdout
    pub fn flush_to_screen(&mut self) -> std::io::Result<()> {
        let stdout = std::io::stdout();
        let mut handle = std::io::BufWriter::new(stdout.lock());

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                if self.back[idx] != self.front[idx] {
                    write!(
                        handle,
                        "\x1B[{};{}H{}{}{}",
                        y + 1,
                        x + 1,
                        self.back[idx].fg_colour,
                        self.back[idx].bg_colour,
                        self.back[idx].ch
                    )?;
                    self.front[idx] = self.back[idx].clone();
                }
            }
        }
        handle.flush()

    }
}

// TODO: Add numbering to feeds.
// TODO: I want to also be able to navigate using indexes.
pub struct Feed<'a> {
    display_name: &'a str,
    articles: Vec<String>,
    idx: usize,
    active: bool,
}

pub struct Subscriptions<'a> {
    feeds: Vec<Feed<'a>>,
    idx: usize,
    in_articles: bool,
}

impl<'a> Feed<'a> {
    pub fn new(display_name: &'a str, articles: Vec<String>) -> Self {
        Self { display_name, articles, idx: 0, active:false }
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
        Self { feeds, idx: 0, in_articles: false }
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

    pub fn next(&mut self) -> &mut Self {
        if !self.feeds.is_empty() && !self.in_articles {
            self.idx = (self.idx + 1) % self.feeds.len();
        } else if self.in_articles {
            self.get_idx_mut().next_article();
        }
        self
    }

    pub fn previous(&mut self) -> &mut Self {
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

pub fn draw_subscriptions(buffer: &mut TerminalBuffer, x: u16, y: u16, y_spacing: u16, subscriptions: &Subscriptions) {
    let mut current_y = y;
    for (i, item) in subscriptions.feeds.iter().enumerate() {
        if i == subscriptions.idx { 
            
            let mut prefix = if subscriptions.in_articles {
                String::from("  ")
            } else {
                String::from("> ")
            };
            prefix.push_str(&i.to_string());
            prefix.push_str(&" - ".to_string());
            prefix.push_str(item.display_name);
            buffer.print_str(x, current_y, &prefix, SELECTED_FG, DEFAULT_BG);
        } else {
            let mut prefix = String::from("  ");
            prefix.push_str(&i.to_string());
            prefix.push_str(&" - ".to_string());
            prefix.push_str(item.display_name);
            buffer.print_str(x, current_y, &prefix, "\x1B[37m", DEFAULT_BG);
        }
        current_y += y_spacing;
    }
}

pub fn draw_feed_articles(buffer: &mut TerminalBuffer, x: u16, y: u16, y_spacing: u16, feed: &mut Feed) {
    let mut current_y = y;

    let mut max_width = 0;
    
    // buffer height - starting y - bottom bar offset - height - padding
    let allowed_height = (buffer.height - y - 3) as usize;

    let starting_idx = feed.idx.saturating_sub(allowed_height);
    
    let eligible_item = feed.articles.len().min(allowed_height);

    for (i, item) in feed.articles.iter().skip(starting_idx).take(eligible_item).enumerate() {

        let idx = i + starting_idx;

        if idx == feed.idx {
            let mut prefix = if !feed.active {
                String::from("  ")
            } else {
                String::from("> ")
            };
            prefix.push_str(&idx.to_string());
            prefix.push_str(&" - ".to_string());
            prefix.push_str(item);
            let width = x + prefix.len() as u16;
            if width > max_width {max_width = width };
            buffer.print_str_padded(x, current_y, &prefix, SELECTED_FG, DEFAULT_BG, width.max(buffer.previous_max_width));
        } else {
            let mut prefix = String::from("  ");
            prefix.push_str(&idx.to_string());
            prefix.push_str(&" - ".to_string());
            prefix.push_str(item);
            let width = x + prefix.len() as u16;
            if width > max_width {max_width = width };
            buffer.print_str_padded(x, current_y, &prefix, "\x1B[37m", DEFAULT_BG, width.max(buffer.previous_max_width));
        }
        current_y += y_spacing;
    }

    if max_width > buffer.previous_max_width { buffer.previous_max_width = max_width };
    
    if current_y - y_spacing < buffer.previous_max_height {
        for padded_y in current_y..=buffer.previous_max_height {
            buffer.print_str_padded(x, padded_y, " ", DEFAULT_FG, DEFAULT_BG, max_width);
        }
    }
    buffer.previous_max_height = current_y - y_spacing;      

}

pub fn draw_bottom_bar(buffer: &mut TerminalBuffer) -> std::io::Result<()> {
    for x in 1..=buffer.width {
        buffer.set_cell(x, buffer.height - 2, ' ', DEFAULT_FG, SELECTED_BG);
    }
    Ok(())
}
