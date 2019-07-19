mod bus;
mod cartridge;
mod cpu;
mod ffi;
mod interrupts;
mod rom;
mod serial;
mod timer;
mod video;

use bus::AddressBus;
use cartridge::Cartridge;
use cpu::CPU;
use interrupts::Interrupts;
use serial::Serial;
use timer::Timer;
use video::Video;

const CPU_CYCLES_PER_FRAME: usize = 70_224;

pub struct Console {
    cpu: CPU,
    cartridge: Option<Cartridge>,
    wram: [u8; 8192],
    serial: Serial,
    timer: Timer,
    video: Video,
    interrupts: Interrupts,
    hram: [u8; 127],
}

impl Console {
    fn new() -> Self {
        Console {
            cpu: CPU::new(),
            cartridge: None,
            wram: [0; 8192],
            serial: Serial::new(),
            timer: Timer::new(),
            video: Video::new(),
            interrupts: Interrupts::new(),
            hram: [0; 127],
        }
    }
}

impl Console {
    fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);

        let mut bus = AddressBus::new(
            self.cartridge.as_mut().unwrap(),
            &mut self.wram,
            &mut self.serial,
            &mut self.timer,
            &mut self.video,
            &mut self.interrupts,
            &mut self.hram,
        );

        bus.write_byte(0xFF05, 0x00);
        bus.write_byte(0xFF06, 0x00);
        bus.write_byte(0xFF07, 0x00);
        bus.write_byte(0xFF10, 0x80);
        bus.write_byte(0xFF11, 0xBF);
        bus.write_byte(0xFF12, 0xF3);
        bus.write_byte(0xFF14, 0xBF);
        bus.write_byte(0xFF16, 0x3F);
        bus.write_byte(0xFF17, 0x00);
        bus.write_byte(0xFF19, 0xBF);
        bus.write_byte(0xFF1A, 0x7F);
        bus.write_byte(0xFF1B, 0xFF);
        bus.write_byte(0xFF1C, 0x9F);
        bus.write_byte(0xFF1E, 0xBF);
        bus.write_byte(0xFF20, 0xFF);
        bus.write_byte(0xFF21, 0x00);
        bus.write_byte(0xFF22, 0x00);
        bus.write_byte(0xFF23, 0xBF);
        bus.write_byte(0xFF24, 0x77);
        bus.write_byte(0xFF25, 0xF3);
        bus.write_byte(0xFF26, 0xF1);
        bus.write_byte(0xFF40, 0x91);
        bus.write_byte(0xFF42, 0x00);
        bus.write_byte(0xFF43, 0x00);
        bus.write_byte(0xFF45, 0x00);
        bus.write_byte(0xFF47, 0xFC);
        bus.write_byte(0xFF48, 0xFF);
        bus.write_byte(0xFF49, 0xFF);
        bus.write_byte(0xFF4A, 0x00);
        bus.write_byte(0xFF4B, 0x00);
        bus.write_byte(0xFFFF, 0x00);
    }

    fn run_frame(&mut self) {
        let mut elapsed_cycles = 0;

        if let Some(cartridge) = &mut self.cartridge {
            while elapsed_cycles <= CPU_CYCLES_PER_FRAME {
                let mut bus = AddressBus::new(
                    cartridge,
                    &mut self.wram,
                    &mut self.serial,
                    &mut self.timer,
                    &mut self.video,
                    &mut self.interrupts,
                    &mut self.hram,
                );

                let cycles = self.cpu.step(&mut bus);

                let interrupts: Vec<_> = vec![
                    self.serial.step(cycles),
                    self.timer.step(cycles),
                    self.video.step(cycles),
                ]
                .into_iter()
                .flatten()
                .collect();

                for interrupt in interrupts {
                    self.interrupts.request(interrupt);
                }

                elapsed_cycles += cycles;
            }
        }
    }
}
