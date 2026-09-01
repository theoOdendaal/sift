use std::io::Write;

mod ffi;

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
        raw.c_lflag &= !(ffi::ECHO | ffi::ICANON | ffi::ISIG);
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
            fg_colour: "\x1B[39m",
            bg_colour: "\x1B[49m",
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
        self.back[index] = Cell { ch, fg_colour: fg, bg_colour: bg };
    }

    pub fn print_str(&mut self, x: u16, y: u16, text: &str, fg: &'static str, bg: &'static str) {
        for (i, ch) in text.chars().enumerate() {
            self.set_cell(x + i as u16, y, ch, fg, bg);
        }
    }

    /// Diff back and front buffers and output changed cells to stdout
    pub fn flush_to_screen(&mut self) -> std::io::Result<()> {
        let mut stdout = std::io::stdout();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                if self.back[idx] != self.front[idx] {
                    write!(
                        stdout,
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
        stdout.flush()
    }
}

pub fn draw_list(
    buffer: &mut TerminalBuffer,
    x: u16,
    y: u16,
    list_spacing: u16,
    list: &[String],
    fg: &'static str,
    bg: &'static str,
) {
    let mut current_y = y;
    for item in list {
        buffer.print_str(x, current_y, item, fg, bg);
        current_y += list_spacing;
    }
}
