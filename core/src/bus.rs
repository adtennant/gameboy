use crate::cartridge::Cartridge;
use crate::interrupts::Interrupts;
use crate::serial::Serial;
use crate::timer::Timer;
use crate::video::Video;
use bit_field::BitField;

pub struct AddressBus<'a> {
    cartridge: &'a mut Cartridge,
    wram: &'a mut [u8; 8192],
    serial: &'a mut Serial,
    timer: &'a mut Timer,
    video: &'a mut Video,
    interrupts: &'a mut Interrupts,
    hram: &'a mut [u8; 127],
}

impl<'a> AddressBus<'a> {
    pub fn new(
        cartridge: &'a mut Cartridge,
        wram: &'a mut [u8; 8192],
        serial: &'a mut Serial,
        timer: &'a mut Timer,
        video: &'a mut Video,
        interrupts: &'a mut Interrupts,
        hram: &'a mut [u8; 127],
    ) -> Self {
        AddressBus {
            cartridge,
            wram,
            serial,
            timer,
            video,
            interrupts,
            hram,
        }
    }
}

impl<'a> AddressBus<'a> {
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.read_byte(address),
            0x8000..=0x9FFF | 0xFE00..=0xFE9F => self.video.read_byte(address),
            0xC000..=0xDFFF => self.wram[usize::from(address) - 0xC000],
            0xE000..=0xFDFF => self.wram[usize::from(address) - 0xE000],

            0xFF01 => self.serial.sb,
            0xFF02 => self.serial.sc,

            0xFF04 => self.timer.div,
            0xFF05 => self.timer.tima,
            0xFF06 => self.timer.tma,
            0xFF07 => self.timer.tac,

            0xFF0F => self.interrupts.r#if,

            0xFF40 => self.video.lcdc,
            0xFF41 => {
                let mut stat = self.video.stat;
                stat.set_bit(2, self.video.coincidence_flag());
                stat.set_bits(0..2, self.video.mode as u8);

                stat
            }
            0xFF42 => self.video.scy,
            0xFF43 => self.video.scx,
            0xFF44 => self.video.ly,
            0xFF45 => self.video.lyc,
            // 0xFF46 => DMA,
            0xFF47 => self.video.bgp,
            0xFF48 => self.video.obp0,
            0xFF49 => self.video.obp1,
            0xFF4A => self.video.wy,
            0xFF4B => self.video.wx,

            0xFF80..=0xFFFE => self.hram[usize::from(address) - 0xFF80],
            0xFFFF => self.interrupts.ie,

            _ => 0xFF,
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(address + 1);

        u16::from_le_bytes([low, high])
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.write_byte(address, value),
            0x8000..=0x9FFF | 0xFE00..=0xFE9F => self.video.write_byte(address, value),
            0xC000..=0xDFFF => self.wram[usize::from(address) - 0xC000] = value,
            0xE000..=0xFDFF => self.wram[usize::from(address) - 0xE000] = value,

            0xFF01 => self.serial.sb = value,
            0xFF02 => self.serial.sc = value,

            0xFF04 => self.timer.div = value,
            0xFF05 => self.timer.tima = value,
            0xFF06 => self.timer.tma = value,
            0xFF07 => self.timer.tac = value,

            0xFF40 => self.video.lcdc = value,
            0xFF41 => {
                self.video.stat.set_bits(2..8, value.get_bits(2..8));
            }
            0xFF42 => self.video.scy = value,
            0xFF43 => self.video.scx = value,
            // 0xFF44 => LY,
            0xFF45 => self.video.lyc = value,
            0xFF46 => {
                let src = u16::from_le_bytes([0, value]);

                for offset in 0..160 {
                    let value = self.read_byte(src + offset);
                    self.write_byte(0xFE00 + offset, value);
                }
            }
            0xFF47 => self.video.bgp = value,
            0xFF48 => self.video.obp0 = value,
            0xFF49 => self.video.obp1 = value,
            0xFF4A => self.video.wy = value,
            0xFF4B => self.video.wx = value,
            0xFF0F => self.interrupts.r#if = value,

            0xFF80..=0xFFFE => self.hram[usize::from(address) - 0xFF80] = value,
            0xFFFF => self.interrupts.ie = value,

            _ => {}
        };
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let bytes = value.to_le_bytes();

        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }
}
