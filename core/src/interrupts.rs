#[repr(u8)]
pub enum Interrupt {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

impl From<Interrupt> for u8 {
    fn from(interrupt: Interrupt) -> Self {
        match interrupt {
            Interrupt::VBlank => 0b0000_0001,
            Interrupt::LCDStat => 0b0000_0010,
            Interrupt::Timer => 0b0000_0100,
            Interrupt::Serial => 0b0000_1000,
            Interrupt::Joypad => 0b0001_0000,
        }
    }
}

pub struct Interrupts {
    pub r#if: u8,
    pub ie: u8,
}

impl Interrupts {
    pub fn new() -> Self {
        Interrupts { r#if: 0, ie: 0 }
    }
}

impl Interrupts {
    pub fn request(&mut self, interrupt: Interrupt) {
        self.r#if |= u8::from(interrupt);
    }
}
