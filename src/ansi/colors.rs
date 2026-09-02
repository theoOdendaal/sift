pub enum FourBit {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Reset,
}

pub struct EightBit {
    code: u8,
}

pub struct TwentyFourBit {
    r: u8,
    g: u8,
    b: u8,
}
