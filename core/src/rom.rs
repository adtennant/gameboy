use std::{ops::Index, path::Path};

pub enum CartridgeType {
    ROMOnly,
    MBC1,
}

pub struct ROM(Vec<u8>);

impl Index<usize> for ROM {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl ROM {
    pub fn from_file<P>(path: P) -> Result<Self, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let bytes = std::fs::read(path)?;
        Ok(ROM(bytes))
    }
}

impl ROM {
    pub fn title(&self) -> String {
        let title = &self.0[0x134..=0x143];
        let title = if let Some(i) = title.iter().position(|&x| x == 0) {
            &title[0..i]
        } else {
            title
        };

        String::from_utf8(title.to_vec()).unwrap()
    }

    pub fn cartridge_type(&self) -> CartridgeType {
        match self.0[0x147] {
            0x00 => CartridgeType::ROMOnly,
            0x01 => CartridgeType::MBC1,
            _ => unimplemented!(),
        }
    }

    pub fn ram_size(&self) -> usize {
        match self.0[0x149] {
            0x00 => 0,
            0x01 => 2048,
            0x02 => 8192,
            0x03 => 32768,
            _ => unreachable!(),
        }
    }
}
