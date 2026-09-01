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
