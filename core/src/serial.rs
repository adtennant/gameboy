// TODO: Shifting bits in/out during transfer
use crate::interrupts::Interrupt;
use std::io;

pub struct Serial {
    pub sb: u8,
    pub sc: u8,

    transfer_cycles: usize,
}

impl Serial {
    pub fn new() -> Self {
        Serial {
            sb: 0,
            sc: 0,

            transfer_cycles: 0,
        }
    }
}

impl Serial {
    pub fn step(&mut self, cycles: usize) -> Vec<Interrupt> {
        let mut interrupts = vec![];

        if self.sc == 0x81 {
            self.transfer_cycles += cycles;

            if self.transfer_cycles >= 8 {
                print!("{}", self.sb as char);

                use io::Write;
                io::stdout().flush().unwrap();

                self.sb = 0xFF;
                self.sc = 0x01;

                self.transfer_cycles = 0;

                interrupts.push(Interrupt::Serial);
            }
        }

        interrupts
    }
}
