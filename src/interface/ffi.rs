use std::io::Write as IoWrite;

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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Winsize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

pub const ECHO: u32 = 0x0008;
pub const ICANON: u32 = 0x0002;
pub const ISIG: u32 = 0x0001;
pub const IXON: u32 = 0x0400;
pub const ICRNL: u32 = 0x0100;
pub const OPOST: u32 = 0x0001;

pub const TCSAFLUSH: i32 = 2;
pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;

#[cfg(target_os = "linux")]
pub const TIOCGWINSZ: usize = 0x5413;

unsafe extern "C" {
    pub fn tcgetattr(fd: i32, termios_p: *mut Termios) -> i32;
    pub fn tcsetattr(fd: i32, optional_actions: i32, termios_p: *const Termios) -> i32;
    pub fn ioctl(fd: i32, request: usize, ws: *mut Winsize) -> i32;
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

pub struct RawModeGuard {
    original: Option<Termios>,
}

impl RawModeGuard {
    pub fn enable() -> std::io::Result<Self> {
        let mut original: Termios = unsafe { std::mem::zeroed() };

        // Retrieves and store the current control attributes
        // and parameters of the terminal, in order to
        // be able to revert back to original state.
        if unsafe { tcgetattr(STDIN_FILENO, &mut original) } != 0 {
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

        Ok(RawModeGuard {
            original: Some(original),
        })
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
