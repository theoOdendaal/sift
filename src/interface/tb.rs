use std::io::Write;

#[derive(Clone, PartialEq, Eq)]
struct Cell {
    ch: char,
    fg_colour: &'static str,
    bg_colour: &'static str,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg_colour: super::DEFAULT_FG,
            bg_colour: super::DEFAULT_BG,
        }
    }
}
/// Maintain two buffers of the terminal screen.
/// The front buffer is a representation of
/// the current terminal state.
/// Changes are pushed to the back buffer first
/// before being incrementally transferred to
/// the front buffer.
pub struct TerminalBuffer {
    pub width: u16,

    pub height: u16,

    front: Vec<Cell>,

    back: Vec<Cell>,

    pub previous_max_width: u16,

    pub previous_max_height: u16,
}

// FIXME: I need to be able to dynamically resize.
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
    
    // update a single cell of `back`
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
    
    // write string to `back` as a sequence of chars,
    // given a starting x and y coordinate
    pub fn print_str(&mut self, x: u16, y: u16, text: &str, fg: &'static str, bg: &'static str) {
        for (i, ch) in text.chars().enumerate() {
            self.set_cell(x + i as u16, y, ch, fg, bg);
        }
    }
    // write padded str to 'back' as a sequence of chars,
    // given a starting x and y coordinate. If len of `text`
    // is smaller than `max_len`, cell's to the right are 
    // set to empty.
    pub fn print_str_padded(
        &mut self,
        x: u16,
        y: u16,
        text: &str,
        fg: &'static str,
        bg: &'static str,
        max_len: u16,
    ) {
        let mut char_count = 0;

        for (i, ch) in text.chars().enumerate() {
            self.set_cell(x + i as u16, y, ch, fg, bg);
            char_count += 1;
        }

        for i in char_count..max_len {
            self.set_cell(x + i, y, ' ', fg, bg);
        }
    }

    // diff back and front buffers and output changed cells to stdout
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

    pub fn clear_back_buffer(&mut self) {
        let default_cell = Cell::default();
        for cell in self.back.iter_mut() {
            *cell = default_cell.clone();
        }
    }

}
