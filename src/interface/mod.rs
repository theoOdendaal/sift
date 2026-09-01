use std::io::Write;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Termios {
    pub c_iflag: u32,
    pub c_oflag: u32,
    pub c_cflag: u32,
    pub c_lflag: u32,
    pub c_line: u8,
    pub c_cc: [u8; 32],
    pub c_ispeed: u32,
    pub c_ospeed: u32,
}

// POSIX flag bits
const ECHO: u32 = 0x0008;
const ICANON: u32 = 0x0002;
const ISIG: u32 = 0x0001;
const IXON: u32 = 0x0400;
const ICRNL: u32 = 0x0100;
const OPOST: u32 = 0x0001;

const TCSAFLUSH: i32 = 2;
const STDIN_FILENO: i32 = 0;

unsafe extern "C" {
    fn tcgetattr(fd: i32, termios_p: *mut Termios) -> i32;
    fn tcsetattr(fd: i32, optional_actions: i32, termios_p: *const Termios) -> i32;
}

pub struct RawModeGuard {
    original: Termios,
}

impl RawModeGuard {
    pub fn enable() -> std::io::Result<Self> {
        let mut original: Termios = unsafe { std::mem::zeroed() };
        if unsafe { tcgetattr(STDIN_FILENO, &mut original) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut raw = original;
        // Disable canonical mode (line buffering), local echo, and signals (Ctrl+C)
        raw.c_lflag &= !(ECHO | ICANON | ISIG);
        // Disable software flow control (Ctrl+S, Ctrl+Q) and newline translation
        raw.c_iflag &= !(IXON | ICRNL);
        // Disable output processing (post-processing)
        raw.c_oflag &= !OPOST;

        if unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut stdout = std::io::stdout();
        write!(stdout, "\x1B[?1049h\x1B[2J\x1B[?25l")?;
        stdout.flush()?;

        Ok(RawModeGuard { original })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {

        // 1. Send ANSI cleanup sequences (Show cursor, leave alternate screen)
        let mut stdout = std::io::stdout();
        let _ = write!(stdout, "\x1B[?25h\x1B[?1049l");
        let _ = stdout.flush();

        // 2. Restore original termios settings
        unsafe {
            tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.original);
        }
    }
}

pub fn draw_box(x: u16, y: u16, width: u16, height: u16, title: &str) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();

    // Move to starting position and draw top line with title
    write!(stdout, "\x1B[{};{}H┌─ {} ", y, x, title)?;
    for _ in 0..(width.saturating_sub(title.len() as u16 + 4)) {
        write!(stdout, "─")?;
    }
    write!(stdout, "┐")?;

    // Draw sides
    for row in 1..height - 1 {
        write!(stdout, "\x1B[{};{}H│", y + row, x)?;
        write!(stdout, "\x1B[{};{}H│", y + row, x + width - 1)?;
    }

    // Draw bottom line
    write!(stdout, "\x1B[{};{}H└", y + height - 1, x)?;
    for _ in 0..(width - 2) {
        write!(stdout, "─")?;
    }
    write!(stdout, "┘")?;

    stdout.flush()?;
    Ok(())
}
