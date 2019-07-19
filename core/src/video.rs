// TODO: 8x16 Sprites
// TODO: Scrolling
// TODO: Window
use crate::interrupts::Interrupt;
use bit_field::BitField;

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAMRead = 2,
    VRAMRead = 3,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Shade {
    White = 0,
    LightGrey = 1,
    DarkGrey = 2,
    Black = 3,
}

impl Default for Shade {
    fn default() -> Self {
        Shade::White
    }
}

pub struct Palettes {
    pub bgp: Vec<Shade>,
    pub obp0: Vec<Shade>,
    pub obp1: Vec<Shade>,
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub pixels: [usize; 64],
}

impl Default for Tile {
    fn default() -> Self {
        Tile { pixels: [0; 64] }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Priority {
    Above,
    Behind,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Above
    }
}

#[derive(Clone, Copy, Default)]
pub struct Sprite {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub priority: Priority,
    pub y_flip: bool,
    pub x_flip: bool,
    pub palette: u8,
}

pub struct Video {
    vram: [u8; 8192],
    oam: [u8; 160],

    pub lcdc: u8,
    pub stat: u8,
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
    pub wy: u8,
    pub wx: u8,

    mode_cycles: usize,
    pub mode: Mode,

    framebuffer: [Shade; 160 * 144],

    tiles: [Tile; 384],
    sprites: [Sprite; 40],
}

impl Video {
    pub fn new() -> Self {
        Video {
            vram: [0; 8192],
            oam: [0; 160],

            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,

            mode_cycles: 0,
            mode: Mode::OAMRead,

            framebuffer: [Shade::White; 160 * 144],

            tiles: [Tile::default(); 384],
            sprites: [Sprite::default(); 40],
        }
    }
}

impl Video {
    pub fn step(&mut self, cycles: usize) -> Vec<Interrupt> {
        let mut interrupts = vec![];

        if !self.display_enabled() {
            self.mode = Mode::HBlank;
            self.ly = 0;
            return interrupts;
        }

        self.mode_cycles += cycles;

        match self.mode {
            Mode::OAMRead => {
                if self.mode_cycles >= 80 {
                    self.mode_cycles -= 80;
                    self.mode = Mode::VRAMRead;
                }
            }
            Mode::VRAMRead => {
                if self.mode_cycles >= 172 {
                    self.mode_cycles -= 172;
                    self.mode = Mode::HBlank;

                    if self.hblank_interrupt_enabled() {
                        interrupts.push(Interrupt::LCDStat);
                    }

                    // draw line
                    self.render_scanline();
                }
            }
            Mode::HBlank => {
                if self.mode_cycles >= 204 {
                    self.mode_cycles -= 204;
                    self.ly += 1;

                    if self.coincidence_flag() && self.coincidence_interrupt_enabled() {
                        interrupts.push(Interrupt::LCDStat);
                    }

                    if self.ly == 143 {
                        self.mode = Mode::VBlank;
                        interrupts.push(Interrupt::VBlank);

                        if self.vblank_interrupt_enabled() {
                            interrupts.push(Interrupt::LCDStat);
                        }
                    } else {
                        self.mode = Mode::OAMRead;

                        if self.oam_interrupt_enabled() {
                            interrupts.push(Interrupt::LCDStat);
                        }
                    }
                }
            }
            Mode::VBlank => {
                if self.mode_cycles >= 456 {
                    self.mode_cycles -= 456;
                    self.ly += 1;

                    if self.coincidence_flag() && self.coincidence_interrupt_enabled() {
                        interrupts.push(Interrupt::LCDStat);
                    }

                    if self.ly > 153 {
                        self.mode = Mode::OAMRead;

                        if self.oam_interrupt_enabled() {
                            interrupts.push(Interrupt::LCDStat);
                        }

                        self.ly = 0;
                    }
                }
            }
        }

        interrupts
    }

    fn render_scanline(&mut self) {
        let palettes = self.palettes();
        let background_tile_map = self.background_tile_map();

        let line = self.ly;
        let framebuffer_offset = usize::from(line) * 160;

        let mut scanline = vec![std::usize::MAX; 160];

        for x in 0..160usize {
            let framebuffer_index = framebuffer_offset + x;

            if self.background_enabled() {
                let background_map_index = (usize::from(line) * 256) + x;
                let pixel = background_tile_map[background_map_index];

                scanline[x] = pixel;
                self.framebuffer[framebuffer_index] = palettes.bgp[pixel];
            } else {
                self.framebuffer[framebuffer_index] = Shade::White;
            }
        }

        if self.sprites_enabled() {
            for sprite in self
                .sprites
                .as_ref()
                .iter()
                .filter(|s| s.y > 0 && s.y < 160)
                .filter(|s| (s.y as i16 - 16) <= line as i16 && (s.y as i16 - 16) + 8 > line as i16)
            {
                let tile = &self.tiles[usize::from(sprite.tile)];
                let palette = if sprite.palette == 0 {
                    &palettes.obp0
                } else {
                    &palettes.obp1
                };

                let pixel_y_offset = usize::from(line - (sprite.y - 16));
                let pixel_y_offset = if sprite.y_flip {
                    7 - pixel_y_offset
                } else {
                    pixel_y_offset
                };

                for x in 0..8usize {
                    let pixel_x_offset = if sprite.x_flip { 7 - x } else { x };
                    let pixel_index = pixel_y_offset * 8 + pixel_x_offset;
                    let pixel = tile.pixels[pixel_index];

                    if pixel == 0 {
                        continue;
                    }

                    let framebuffer_x = usize::from(sprite.x - 8) + x;
                    let framebuffer_index = framebuffer_offset + framebuffer_x;

                    if sprite.priority == Priority::Behind && scanline[framebuffer_x] != 0 {
                        continue;
                    }

                    self.framebuffer[framebuffer_index] = palette[pixel];
                }
            }
        }
    }
}

impl Video {
    pub fn read_byte(&self, address: u16) -> u8 {
        let address = usize::from(address);

        match address {
            0x8000..=0x9FFF => {
                if let Mode::VRAMRead = self.mode {
                    return 0xFF;
                }

                self.vram[address - 0x8000]
            }
            0xFE00..=0xFE9F => {
                if let Mode::OAMRead | Mode::VRAMRead = self.mode {
                    return 0xFF;
                }

                self.oam[address - 0xFE00]
            }
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                if let Mode::VRAMRead = self.mode {
                    return;
                }

                self.write_vram(address, value)
            }
            0xFE00..=0xFE9F => {
                if let Mode::OAMRead | Mode::VRAMRead = self.mode {
                    return;
                }

                self.write_oam(address, value)
            }
            _ => unreachable!(),
        }
    }

    fn write_vram(&mut self, address: u16, value: u8) {
        let address = usize::from(address);

        let index = address - 0x8000;
        self.vram[index] = value;

        if address > 0x97FF {
            return; // background tile map addresses
        }

        let tile_index = index / 16;
        let tile = &mut self.tiles[tile_index];

        let byte = index % 16;
        let row = byte / 2;

        let bit_to_set = if byte % 2 == 0 { 0 } else { 1 };

        for x in 0..8 {
            let i = 7 - x;
            let bit = value.get_bit(i);

            tile.pixels[row * 8 + x].set_bit(bit_to_set, bit);
        }
    }

    fn write_oam(&mut self, address: u16, value: u8) {
        let address = usize::from(address);

        let index = address - 0xFE00;
        self.oam[index] = value;

        let sprite_index = index / 4;
        let sprite = &mut self.sprites[sprite_index];

        let byte = index % 4;

        match byte {
            0 => sprite.y = value, // - 16,
            1 => sprite.x = value, // - 8,
            2 => sprite.tile = value,
            3 => {
                sprite.priority = if value.get_bit(7) {
                    Priority::Behind
                } else {
                    Priority::Above
                };
                sprite.y_flip = value.get_bit(6);
                sprite.x_flip = value.get_bit(5);
                sprite.palette = if value.get_bit(4) { 1 } else { 0 };
            }
            _ => unreachable!(),
        }
    }
}

#[allow(non_camel_case_types)]
pub enum BackgroundAddressMode {
    x8000,
    x8800,
}

#[allow(non_camel_case_types)]
pub enum BackgroundTileMap {
    x9800,
    x9C00,
}

impl Video {
    fn display_enabled(&self) -> bool {
        self.lcdc.get_bit(7)
    }

    fn background_address_mode(&self) -> BackgroundAddressMode {
        if self.lcdc.get_bit(4) {
            BackgroundAddressMode::x8000
        } else {
            BackgroundAddressMode::x8800
        }
    }

    fn background_tile_map_display(&self) -> BackgroundTileMap {
        if self.lcdc.get_bit(3) {
            BackgroundTileMap::x9C00
        } else {
            BackgroundTileMap::x9800
        }
    }

    fn sprites_enabled(&self) -> bool {
        self.lcdc.get_bit(1)
    }

    fn background_enabled(&self) -> bool {
        self.lcdc.get_bit(0)
    }

    fn coincidence_interrupt_enabled(&self) -> bool {
        self.stat.get_bit(6)
    }

    fn oam_interrupt_enabled(&self) -> bool {
        self.stat.get_bit(5)
    }

    fn vblank_interrupt_enabled(&self) -> bool {
        self.stat.get_bit(4)
    }

    fn hblank_interrupt_enabled(&self) -> bool {
        self.stat.get_bit(3)
    }

    pub fn coincidence_flag(&self) -> bool {
        self.lyc == self.ly
    }
}

impl Video {
    fn palette(&self, reg: u8) -> Vec<Shade> {
        [0..=1, 2..=3, 4..=5, 6..=7]
            .into_iter()
            .map(|range| match reg.get_bits(range.clone()) {
                0 => Shade::White,
                1 => Shade::LightGrey,
                2 => Shade::DarkGrey,
                3 => Shade::Black,
                _ => unreachable!("Invalid shade"),
            })
            .collect()
    }

    fn palettes(&self) -> Palettes {
        Palettes {
            bgp: self.palette(self.bgp),
            obp0: self.palette(self.obp0),
            obp1: self.palette(self.obp1),
        }
    }

    fn background_tile_map(&self) -> Vec<usize> {
        let mut result = vec![0; 32 * 32 * 8 * 8];

        let tile_map_address = match self.background_tile_map_display() {
            BackgroundTileMap::x9800 => 0x9800,
            BackgroundTileMap::x9C00 => 0x9C00,
        };

        for i in 0..(32 * 32) {
            let tile_index = self.vram[tile_map_address + i - 0x8000]; // don't use read_byte as this can happen during VRAM/OAMRead

            let tile = match self.background_address_mode() {
                BackgroundAddressMode::x8000 => &self.tiles[usize::from(tile_index)],
                BackgroundAddressMode::x8800 => {
                    let tile_index = i16::from(tile_index as i8);
                    let tile_index = 256 + tile_index;
                    &self.tiles[tile_index as usize]
                }
            };

            let x_offset = (i % 32) * 8;
            let y_offset = (i / 32) * 8;

            for x in 0..8 {
                for y in 0..8 {
                    let i = (x_offset + x) + ((y_offset + y) * 8 * 32);
                    result[usize::from(i)] = tile.pixels[usize::from(y * 8 + x)];
                }
            }
        }

        result
    }

    pub fn framebuffer(&self) -> &[Shade] {
        &self.framebuffer
    }
}
