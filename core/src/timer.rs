//use crate::bus::{Interrupt, InterruptHandler};

use crate::interrupts::Interrupt;
use bit_field::BitField;

pub struct Timer {
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,

    divider_cycles: usize,
    timer_cycles: usize,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,

            divider_cycles: 0,
            timer_cycles: 0,
        }
    }
}

impl Timer {
    pub fn step(&mut self, cycles: usize) -> Vec<Interrupt> {
        let mut interrupts = vec![];

        self.step_divider(cycles);

        if self.timer_enabled() {
            let overflow = self.step_timer(cycles);

            if overflow {
                interrupts.push(Interrupt::Timer);
            }
        }

        interrupts
    }
}

impl Timer {
    fn step_divider(&mut self, cycles: usize) {
        self.divider_cycles += cycles;

        while self.divider_cycles >= 256 {
            // step div at 16384Hz, the CPU clock rate is 4194304Hz, so div is steps every 256 cycles
            self.div = self.div.wrapping_add(1);
            self.divider_cycles -= 256;
        }
    }

    fn timer_enabled(&self) -> bool {
        self.tac.get_bit(2)
    }

    fn get_freq(&self) -> usize {
        match self.tac.get_bits(0..1) {
            0b00 => 1024, // 4096Hz
            0b01 => 16,   // 262144Hz
            0b10 => 64,   // 65536Hz
            0b11 => 256,  // 16384Hz
            _ => unreachable!(),
        }
    }

    fn step_timer(&mut self, cycles: usize) -> bool {
        let mut has_overflown = false;

        // increment tima at a rate of cycles / freq
        self.timer_cycles += cycles;

        while self.timer_cycles >= self.get_freq() {
            let (tima, overflow) = self.tima.overflowing_add(1);

            if overflow {
                self.tima = self.tma;
                has_overflown = true;
            } else {
                self.tima = tima;
            }

            self.timer_cycles -= self.get_freq();
        }

        has_overflown
    }
}
