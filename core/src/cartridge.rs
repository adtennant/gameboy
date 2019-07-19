use super::rom::{CartridgeType, ROM};

trait MemoryBankController {
    fn read_byte(&self, rom: &ROM, address: u16) -> u8 {
        let address = usize::from(address);

        match address {
            0x0000..=0x7FFF => rom[address],
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, _address: u16, _value: u8) {}
}

pub struct Cartridge {
    rom: ROM,
    mbc: Box<MemoryBankController>,
}

impl Cartridge {
    pub fn read_byte(&self, address: u16) -> u8 {
        self.mbc.read_byte(&self.rom, address)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.mbc.write_byte(address, value);
    }
}

impl From<ROM> for Cartridge {
    fn from(rom: ROM) -> Self {
        let mbc: Box<MemoryBankController> = match rom.cartridge_type() {
            CartridgeType::ROMOnly => Box::new(MBC0 {}),
            CartridgeType::MBC1 => Box::new(MBC1::new(rom.ram_size())),
        };

        Cartridge { rom, mbc }
    }
}

pub struct MBC0;

impl MemoryBankController for MBC0 {}

use bit_field::BitField;

enum BankMode {
    ROM,
    RAM,
}

pub struct MBC1 {
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    bank_mode: BankMode,
}

impl MBC1 {
    fn new(ram_size: usize) -> Self {
        MBC1 {
            ram: vec![0; ram_size],
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            bank_mode: BankMode::ROM,
        }
    }
}

impl MemoryBankController for MBC1 {
    fn read_byte(&self, rom: &ROM, address: u16) -> u8 {
        let address = usize::from(address);

        match address {
            // ROM Bank 00 (Read Only)
            0x0000..=0x3FFF => rom[address],
            // ROM Bank 01-7F (Read Only)
            0x4000..=0x7FFF => {
                let offset = self.rom_bank as usize * 0x4000;
                rom[offset + address - 0x4000]
            }
            // RAM Bank 00-03, if any (Read/Write)
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                let offset = self.ram_bank as usize * 0x2000;
                self.ram[offset + address - 0xA000]
            }
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        let address = usize::from(address);

        match address {
            // RAM Enable (Write Only)
            0x0000..=0x1FFF => {
                self.ram_enabled = value.get_bits(0..4) == 0x0A;
            }
            // ROM Bank Number (Write Only)
            0x2000..=0x3FFF => {
                self.rom_bank.set_bits(0..5, value);
            }
            // RAM Bank Number - or - Upper Bits of ROM Bank Number (Write Only)
            0x4000..=0x5FFF => {
                match self.bank_mode {
                    BankMode::ROM => {
                        self.rom_bank.set_bits(5..6, value);
                    }
                    BankMode::RAM => match value {
                        0x00..=0x03 => self.ram_bank = value,
                        _ => unreachable!(),
                    },
                };
            }
            // ROM/RAM Mode Select (Write Only)
            0x6000..=0x7FFF => {
                self.bank_mode = match value {
                    0x00 => BankMode::ROM,
                    0x01 => BankMode::RAM,
                    _ => unreachable!(),
                };
            }
            // RAM Bank 00-03, if any (Read/Write)
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }

                let offset = self.ram_bank as usize * 0x2000;
                self.ram[offset + address - 0xA000] = value;
            }
            _ => unreachable!(),
        }
    }
}
