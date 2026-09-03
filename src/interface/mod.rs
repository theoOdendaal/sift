use std::io::Write;

use crate::interface::ffi::{
    ECHO, ICANON, ICRNL, ISIG, IXON, OPOST, STDIN_FILENO, STDOUT_FILENO, TCSAFLUSH, TIOCGWINSZ,
    Winsize, ioctl, tcsetattr,
};

mod ffi;

pub const DEFAULT_FG: &'static str = "\x1B[39m";
pub const DEFAULT_BG: &'static str = "\x1B[49m";
pub const RESET_ALL: &'static str = "\x1B[0m";

pub struct RawModeGuard {
    original: ffi::Termios,
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

        Ok(RawModeGuard { original })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let mut stdout = std::io::stdout();
        let _ = write!(stdout, "\x1B[?25h\x1B[?1049l");
        let _ = stdout.flush();

        unsafe {
            tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.original);
        }
    }
}

// https://man7.org/linux/man-pages/man2/TIOCSWINSZ.2const.html
pub fn get_terminal_size() -> std::io::Result<(u16, u16)> {
    let mut ws: Winsize = unsafe { std::mem::zeroed() };

    let result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) };

    if result == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        Ok((ws.ws_col, ws.ws_row))
    } else {
        Err(std::io::Error::last_os_error())
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
}

impl TerminalBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            front: vec![Cell::default(); size],
            back: vec![Cell::default(); size],
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

// TODO: Add numbering when printing.
// TODO: Impl Display trait for this list, rather than using draw_list
pub struct VerticalList<'a> {
    items: Vec<&'a str>,
    idx: usize,
    active: bool,
}

impl<'a> VerticalList<'a> {
    pub fn new(items: Vec<&'a str>, active: bool) -> Self {
        Self { items, idx: 0, active }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn set_active(&mut self) {
        self.active = true;
    }

    pub fn set_inactive(&mut self) {
        self.active = false;
    }

    pub fn next_item(&mut self) -> &mut Self {
        if !self.items.is_empty() {
            self.idx = (self.idx + 1) % self.items.len();
        }
        self
    }

    pub fn previous_item(&mut self) -> &mut Self {
        self.idx = if self.idx == 0 {
            self.items.len() - 1
        } else {
            self.idx - 1
        };
        self
    }
}

pub struct HorizontalList<'a> {
    items: Vec<VerticalList<'a>>,
    idx: usize,
}

impl<'a> HorizontalList<'a> {
    pub fn new(items: Vec<VerticalList<'a>>) -> Self {
        Self { items, idx: 0 } 
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn get_mut_idx(&mut self) -> &mut VerticalList<'a> {
        &mut self.items[self.idx]
    }
    
    // Unlike a vertical list, the horizontal list
    // should not wrap around.
    pub fn next_item(&mut self) -> &mut Self {
        if !self.items.is_empty() && self.idx < self.items.len() - 1 {
            self.items[self.idx].set_inactive();
            self.idx += 1;
            self.items[self.idx].set_active();
        }
        self
    }

    pub fn previous_item(&mut self) -> &mut Self {
        if self.idx > 0 {
            self.items[self.idx].set_inactive();
            self.idx -= 1;
            self.items[self.idx].set_active();
        }
        self
    }

}

// A list of lists displaying only the item
// for the current idx.
pub struct DenseList<'a> {
    items: Vec<VerticalList<'a>>,
    idx: usize,
}

pub struct Area {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

pub trait Render {
    fn render(&self, area: &Area, buffer: &mut TerminalBuffer);
}

impl<'a> Render for VerticalList<'a> {
    fn render(&self, area: &Area, buffer: &mut TerminalBuffer) {
        
    }

}


pub fn draw_horizontal_list(
    buffer: &mut TerminalBuffer,
    x: u16,
    y: u16,
    x_spacing: u16,
    y_spacing: u16,
    list: &mut HorizontalList,
) {

    for (i,l) in list.items.iter().enumerate() {
        draw_list(buffer, x + x_spacing*(i as u16) , y, y_spacing, l);
    }

}


pub fn draw_list(
    buffer: &mut TerminalBuffer,
    x: u16,
    y: u16,
    list_spacing: u16,
    list: &VerticalList,
) {
    let mut current_y = y;
    for (i, item) in list.items.iter().enumerate() {
        if i == list.idx && list.active {
            let mut prefix = String::from("> ");
            prefix.push_str(item);
            buffer.print_str(x, current_y, &prefix, "\x1B[38;5;208m", DEFAULT_BG);
        } else {
            let mut prefix = String::from("  ");
            prefix.push_str(item);
            buffer.print_str(x, current_y, &prefix, "\x1B[37m", DEFAULT_BG);
        }
        current_y += list_spacing;
    }
}

pub fn draw_bottom_bar(buffer: &mut TerminalBuffer) -> std::io::Result<()> {
    for x in 1..=buffer.width {
        buffer.set_cell(x, buffer.height - 2, ' ', DEFAULT_FG, "\x1B[48;5;208m");
    }
    Ok(())
}
