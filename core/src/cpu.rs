#![allow(non_upper_case_globals)]

use super::bus::AddressBus;
use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct Flag : u8 {
        const Zero      = 0b1000_0000;
        const Subtract  = 0b0100_0000;
        const HalfCarry = 0b0010_0000;
        const Carry     = 0b0001_0000;
    }
}

impl Flag {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: Flag,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

impl Registers {
    fn get_af(&self) -> u16 {
        (u16::from(self.a) << 8) | u16::from(self.f.bits)
    }

    fn get_bc(&self) -> u16 {
        (u16::from(self.b) << 8) | u16::from(self.c)
    }

    fn get_de(&self) -> u16 {
        (u16::from(self.d) << 8) | u16::from(self.e)
    }

    fn get_hl(&self) -> u16 {
        (u16::from(self.h) << 8) | u16::from(self.l)
    }

    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f.bits = (value & 0x00F0) as u8;
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
}

pub struct CPU {
    cycles: usize,
    registers: Registers,
    halt: bool,
    ime: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            cycles: 0,
            registers: Registers {
                a: 0x01,
                b: 0x00,
                c: 0x13,
                d: 0x00,
                e: 0xD8,
                f: Flag::Zero | Flag::HalfCarry | Flag::Carry,
                h: 0x01,
                l: 0x4D,
                pc: 0x0100,
                sp: 0xFFFE,
            },
            halt: false,
            ime: true,
        }
    }

    pub fn step(&mut self, memory: &mut AddressBus) -> usize {
        if self.handle_interrupts(memory) {
            return 16;
        }

        if self.halt {
            return 4;
        }

        let opcode = memory.read_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        let cycles = match opcode {
            0x00 => self.nop(),
            0x01 => self.ld_bc_nn(memory),
            0x02 => self.ld_bc_a(memory),
            0x03 => self.inc_bc(),
            0x04 => self.inc_b(),
            0x05 => self.dec_b(),
            0x06 => self.ld_b_n(memory),
            0x07 => self.rlca(),
            0x08 => self.ld_nn_sp(memory),
            0x09 => self.add_hl_bc(),
            0x0A => self.ld_a_bc(memory),
            0x0B => self.dec_bc(),
            0x0C => self.inc_c(),
            0x0D => self.dec_c(),
            0x0E => self.ld_c_n(memory),
            0x0F => self.rrca(),

            0x10 => self.stop(),
            0x11 => self.ld_de_nn(memory),
            0x12 => self.ld_de_a(memory),
            0x13 => self.inc_de(),
            0x14 => self.inc_d(),
            0x15 => self.dec_d(),
            0x16 => self.ld_d_n(memory),
            0x17 => self.rla(),
            0x18 => self.jr_n(memory),
            0x19 => self.add_hl_de(),
            0x1A => self.ld_a_de(memory),
            0x1B => self.dec_de(),
            0x1C => self.inc_e(),
            0x1D => self.dec_e(),
            0x1E => self.ld_e_n(memory),
            0x1F => self.rra(),

            0x20 => self.jr_nz_n(memory),
            0x21 => self.ld_hl_nn(memory),
            0x22 => self.ldi_hl_a(memory),
            0x23 => self.inc_hl(),
            0x24 => self.inc_h(),
            0x25 => self.dec_h(),
            0x26 => self.ld_h_n(memory),
            0x27 => self.daa(),
            0x28 => self.jr_z_n(memory),
            0x29 => self.add_hl_hl(),
            0x2A => self.ldi_a_hl(memory),
            0x2B => self.dec_hl(),
            0x2C => self.inc_l(),
            0x2D => self.dec_l(),
            0x2E => self.ld_l_n(memory),
            0x2F => self.cpl(),

            0x30 => self.jr_nc_n(memory),
            0x31 => self.ld_sp_nn(memory),
            0x32 => self.ldd_hl_a(memory),
            0x33 => self.inc_sp(),
            0x34 => self.inc_hl_ref(memory),
            0x35 => self.dec_hl_ref(memory),
            0x36 => self.ld_hl_n(memory),
            0x37 => self.scf(),
            0x38 => self.jr_c_n(memory),
            0x39 => self.add_hl_sp(),
            0x3A => self.ldd_a_hl(memory),
            0x3B => self.dec_sp(),
            0x3C => self.inc_a(),
            0x3D => self.dec_a(),
            0x3E => self.ld_a_n(memory),
            0x3F => self.ccf(),

            0x40 => self.ld_b_b(),
            0x41 => self.ld_b_c(),
            0x42 => self.ld_b_d(),
            0x43 => self.ld_b_e(),
            0x44 => self.ld_b_h(),
            0x45 => self.ld_b_l(),
            0x46 => self.ld_b_hl(memory),
            0x47 => self.ld_b_a(),
            0x48 => self.ld_c_b(),
            0x49 => self.ld_c_c(),
            0x4A => self.ld_c_d(),
            0x4B => self.ld_c_e(),
            0x4C => self.ld_c_h(),
            0x4D => self.ld_c_l(),
            0x4E => self.ld_c_hl(memory),
            0x4F => self.ld_c_a(),

            0x50 => self.ld_d_b(),
            0x51 => self.ld_d_c(),
            0x52 => self.ld_d_d(),
            0x53 => self.ld_d_e(),
            0x54 => self.ld_d_h(),
            0x55 => self.ld_d_l(),
            0x56 => self.ld_d_hl(memory),
            0x57 => self.ld_d_a(),
            0x58 => self.ld_e_b(),
            0x59 => self.ld_e_c(),
            0x5A => self.ld_e_d(),
            0x5B => self.ld_e_e(),
            0x5C => self.ld_e_h(),
            0x5D => self.ld_e_l(),
            0x5E => self.ld_e_hl(memory),
            0x5F => self.ld_e_a(),

            0x60 => self.ld_h_b(),
            0x61 => self.ld_h_c(),
            0x62 => self.ld_h_d(),
            0x63 => self.ld_h_e(),
            0x64 => self.ld_h_h(),
            0x65 => self.ld_h_l(),
            0x66 => self.ld_h_hl(memory),
            0x67 => self.ld_h_a(),
            0x68 => self.ld_l_b(),
            0x69 => self.ld_l_c(),
            0x6A => self.ld_l_d(),
            0x6B => self.ld_l_e(),
            0x6C => self.ld_l_h(),
            0x6D => self.ld_l_l(),
            0x6E => self.ld_l_hl(memory),
            0x6F => self.ld_l_a(),

            0x70 => self.ld_hl_b(memory),
            0x71 => self.ld_hl_c(memory),
            0x72 => self.ld_hl_d(memory),
            0x73 => self.ld_hl_e(memory),
            0x74 => self.ld_hl_h(memory),
            0x75 => self.ld_hl_l(memory),
            0x76 => self.halt(),
            0x77 => self.ld_hl_a(memory),
            0x78 => self.ld_a_b(),
            0x79 => self.ld_a_c(),
            0x7A => self.ld_a_d(),
            0x7B => self.ld_a_e(),
            0x7C => self.ld_a_h(),
            0x7D => self.ld_a_l(),
            0x7E => self.ld_a_hl(memory),
            0x7F => self.ld_a_a(),

            0x80 => self.add_a_b(),
            0x81 => self.add_a_c(),
            0x82 => self.add_a_d(),
            0x83 => self.add_a_e(),
            0x84 => self.add_a_h(),
            0x85 => self.add_a_l(),
            0x86 => self.add_a_hl(memory),
            0x87 => self.add_a_a(),
            0x88 => self.adc_a_b(),
            0x89 => self.adc_a_c(),
            0x8A => self.adc_a_d(),
            0x8B => self.adc_a_e(),
            0x8C => self.adc_a_h(),
            0x8D => self.adc_a_l(),
            0x8E => self.adc_a_hl(memory),
            0x8F => self.adc_a_a(),

            0x90 => self.sub_b(),
            0x91 => self.sub_c(),
            0x92 => self.sub_d(),
            0x93 => self.sub_e(),
            0x94 => self.sub_h(),
            0x95 => self.sub_l(),
            0x96 => self.sub_hl(memory),
            0x97 => self.sub_a(),
            0x98 => self.sbc_a_b(),
            0x99 => self.sbc_a_c(),
            0x9A => self.sbc_a_d(),
            0x9B => self.sbc_a_e(),
            0x9C => self.sbc_a_h(),
            0x9D => self.sbc_a_l(),
            0x9E => self.sbc_a_hl(memory),
            0x9F => self.sbc_a_a(),

            0xA0 => self.and_b(),
            0xA1 => self.and_c(),
            0xA2 => self.and_d(),
            0xA3 => self.and_e(),
            0xA4 => self.and_h(),
            0xA5 => self.and_l(),
            0xA6 => self.and_hl(memory),
            0xA7 => self.and_a(),
            0xA8 => self.xor_b(),
            0xA9 => self.xor_c(),
            0xAA => self.xor_d(),
            0xAB => self.xor_e(),
            0xAC => self.xor_h(),
            0xAD => self.xor_l(),
            0xAE => self.xor_hl(memory),
            0xAF => self.xor_a(),

            0xB0 => self.or_b(),
            0xB1 => self.or_c(),
            0xB2 => self.or_d(),
            0xB3 => self.or_e(),
            0xB4 => self.or_h(),
            0xB5 => self.or_l(),
            0xB6 => self.or_hl(memory),
            0xB7 => self.or_a(),
            0xB8 => self.cp_b(),
            0xB9 => self.cp_c(),
            0xBA => self.cp_d(),
            0xBB => self.cp_e(),
            0xBC => self.cp_h(),
            0xBD => self.cp_l(),
            0xBE => self.cp_hl(memory),
            0xBF => self.cp_a(),

            0xC0 => self.ret_nz(memory),
            0xC1 => self.pop_bc(memory),
            0xC2 => self.jp_nz_nn(memory),
            0xC3 => self.jp_nn(memory),
            0xC4 => self.call_nz_nn(memory),
            0xC5 => self.push_bc(memory),
            0xC6 => self.add_a_n(memory),
            0xC7 => self.rst_00(memory),
            0xC8 => self.ret_z(memory),
            0xC9 => self.ret(memory),
            0xCA => self.jp_z_nn(memory),
            0xCB => {
                let opcode = memory.read_byte(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);

                match opcode {
                    0x00 => self.rlc_b(),
                    0x01 => self.rlc_c(),
                    0x02 => self.rlc_d(),
                    0x03 => self.rlc_e(),
                    0x04 => self.rlc_h(),
                    0x05 => self.rlc_l(),
                    0x06 => self.rlc_hl(memory),
                    0x07 => self.rlc_a(),
                    0x08 => self.rrc_b(),
                    0x09 => self.rrc_c(),
                    0x0A => self.rrc_d(),
                    0x0B => self.rrc_e(),
                    0x0C => self.rrc_h(),
                    0x0D => self.rrc_l(),
                    0x0E => self.rrc_hl(memory),
                    0x0F => self.rrc_a(),

                    0x10 => self.rl_b(),
                    0x11 => self.rl_c(),
                    0x12 => self.rl_d(),
                    0x13 => self.rl_e(),
                    0x14 => self.rl_h(),
                    0x15 => self.rl_l(),
                    0x16 => self.rl_hl(memory),
                    0x17 => self.rl_a(),
                    0x18 => self.rr_b(),
                    0x19 => self.rr_c(),
                    0x1A => self.rr_d(),
                    0x1B => self.rr_e(),
                    0x1C => self.rr_h(),
                    0x1D => self.rr_l(),
                    0x1E => self.rr_hl(memory),
                    0x1F => self.rr_a(),

                    0x20 => self.sla_b(),
                    0x21 => self.sla_c(),
                    0x22 => self.sla_d(),
                    0x23 => self.sla_e(),
                    0x24 => self.sla_h(),
                    0x25 => self.sla_l(),
                    0x26 => self.sla_hl(memory),
                    0x27 => self.sla_a(),
                    0x28 => self.sra_b(),
                    0x29 => self.sra_c(),
                    0x2A => self.sra_d(),
                    0x2B => self.sra_e(),
                    0x2C => self.sra_h(),
                    0x2D => self.sra_l(),
                    0x2E => self.sra_hl(memory),
                    0x2F => self.sra_a(),

                    0x30 => self.swap_b(),
                    0x31 => self.swap_c(),
                    0x32 => self.swap_d(),
                    0x33 => self.swap_e(),
                    0x34 => self.swap_h(),
                    0x35 => self.swap_l(),
                    0x36 => self.swap_hl(memory),
                    0x37 => self.swap_a(),
                    0x38 => self.srl_b(),
                    0x39 => self.srl_c(),
                    0x3A => self.srl_d(),
                    0x3B => self.srl_e(),
                    0x3C => self.srl_h(),
                    0x3D => self.srl_l(),
                    0x3E => self.srl_hl(memory),
                    0x3F => self.srl_a(),

                    0x40 => self.bit_0_b(),
                    0x41 => self.bit_0_c(),
                    0x42 => self.bit_0_d(),
                    0x43 => self.bit_0_e(),
                    0x44 => self.bit_0_h(),
                    0x45 => self.bit_0_l(),
                    0x46 => self.bit_0_hl(memory),
                    0x47 => self.bit_0_a(),
                    0x48 => self.bit_1_b(),
                    0x49 => self.bit_1_c(),
                    0x4A => self.bit_1_d(),
                    0x4B => self.bit_1_e(),
                    0x4C => self.bit_1_h(),
                    0x4D => self.bit_1_l(),
                    0x4E => self.bit_1_hl(memory),
                    0x4F => self.bit_1_a(),

                    0x50 => self.bit_2_b(),
                    0x51 => self.bit_2_c(),
                    0x52 => self.bit_2_d(),
                    0x53 => self.bit_2_e(),
                    0x54 => self.bit_2_h(),
                    0x55 => self.bit_2_l(),
                    0x56 => self.bit_2_hl(memory),
                    0x57 => self.bit_2_a(),
                    0x58 => self.bit_3_b(),
                    0x59 => self.bit_3_c(),
                    0x5A => self.bit_3_d(),
                    0x5B => self.bit_3_e(),
                    0x5C => self.bit_3_h(),
                    0x5D => self.bit_3_l(),
                    0x5E => self.bit_3_hl(memory),
                    0x5F => self.bit_3_a(),

                    0x60 => self.bit_4_b(),
                    0x61 => self.bit_4_c(),
                    0x62 => self.bit_4_d(),
                    0x63 => self.bit_4_e(),
                    0x64 => self.bit_4_h(),
                    0x65 => self.bit_4_l(),
                    0x66 => self.bit_4_hl(memory),
                    0x67 => self.bit_4_a(),
                    0x68 => self.bit_5_b(),
                    0x69 => self.bit_5_c(),
                    0x6A => self.bit_5_d(),
                    0x6B => self.bit_5_e(),
                    0x6C => self.bit_5_h(),
                    0x6D => self.bit_5_l(),
                    0x6E => self.bit_5_hl(memory),
                    0x6F => self.bit_5_a(),

                    0x70 => self.bit_6_b(),
                    0x71 => self.bit_6_c(),
                    0x72 => self.bit_6_d(),
                    0x73 => self.bit_6_e(),
                    0x74 => self.bit_6_h(),
                    0x75 => self.bit_6_l(),
                    0x76 => self.bit_6_hl(memory),
                    0x77 => self.bit_6_a(),
                    0x78 => self.bit_7_b(),
                    0x79 => self.bit_7_c(),
                    0x7A => self.bit_7_d(),
                    0x7B => self.bit_7_e(),
                    0x7C => self.bit_7_h(),
                    0x7D => self.bit_7_l(),
                    0x7E => self.bit_7_hl(memory),
                    0x7F => self.bit_7_a(),

                    0x80 => self.res_0_b(),
                    0x81 => self.res_0_c(),
                    0x82 => self.res_0_d(),
                    0x83 => self.res_0_e(),
                    0x84 => self.res_0_h(),
                    0x85 => self.res_0_l(),
                    0x86 => self.res_0_hl(memory),
                    0x87 => self.res_0_a(),
                    0x88 => self.res_1_b(),
                    0x89 => self.res_1_c(),
                    0x8A => self.res_1_d(),
                    0x8B => self.res_1_e(),
                    0x8C => self.res_1_h(),
                    0x8D => self.res_1_l(),
                    0x8E => self.res_1_hl(memory),
                    0x8F => self.res_1_a(),

                    0x90 => self.res_2_b(),
                    0x91 => self.res_2_c(),
                    0x92 => self.res_2_d(),
                    0x93 => self.res_2_e(),
                    0x94 => self.res_2_h(),
                    0x95 => self.res_2_l(),
                    0x96 => self.res_2_hl(memory),
                    0x97 => self.res_2_a(),
                    0x98 => self.res_3_b(),
                    0x99 => self.res_3_c(),
                    0x9A => self.res_3_d(),
                    0x9B => self.res_3_e(),
                    0x9C => self.res_3_h(),
                    0x9D => self.res_3_l(),
                    0x9E => self.res_3_hl(memory),
                    0x9F => self.res_3_a(),

                    0xA0 => self.res_4_b(),
                    0xA1 => self.res_4_c(),
                    0xA2 => self.res_4_d(),
                    0xA3 => self.res_4_e(),
                    0xA4 => self.res_4_h(),
                    0xA5 => self.res_4_l(),
                    0xA6 => self.res_4_hl(memory),
                    0xA7 => self.res_4_a(),
                    0xA8 => self.res_5_b(),
                    0xA9 => self.res_5_c(),
                    0xAA => self.res_5_d(),
                    0xAB => self.res_5_e(),
                    0xAC => self.res_5_h(),
                    0xAD => self.res_5_l(),
                    0xAE => self.res_5_hl(memory),
                    0xAF => self.res_5_a(),

                    0xB0 => self.res_6_b(),
                    0xB1 => self.res_6_c(),
                    0xB2 => self.res_6_d(),
                    0xB3 => self.res_6_e(),
                    0xB4 => self.res_6_h(),
                    0xB5 => self.res_6_l(),
                    0xB6 => self.res_6_hl(memory),
                    0xB7 => self.res_6_a(),
                    0xB8 => self.res_7_b(),
                    0xB9 => self.res_7_c(),
                    0xBA => self.res_7_d(),
                    0xBB => self.res_7_e(),
                    0xBC => self.res_7_h(),
                    0xBD => self.res_7_l(),
                    0xBE => self.res_7_hl(memory),
                    0xBF => self.res_7_a(),

                    0xC0 => self.set_0_b(),
                    0xC1 => self.set_0_c(),
                    0xC2 => self.set_0_d(),
                    0xC3 => self.set_0_e(),
                    0xC4 => self.set_0_h(),
                    0xC5 => self.set_0_l(),
                    0xC6 => self.set_0_hl(memory),
                    0xC7 => self.set_0_a(),
                    0xC8 => self.set_1_b(),
                    0xC9 => self.set_1_c(),
                    0xCA => self.set_1_d(),
                    0xCB => self.set_1_e(),
                    0xCC => self.set_1_h(),
                    0xCD => self.set_1_l(),
                    0xCE => self.set_1_hl(memory),
                    0xCF => self.set_1_a(),

                    0xD0 => self.set_2_b(),
                    0xD1 => self.set_2_c(),
                    0xD2 => self.set_2_d(),
                    0xD3 => self.set_2_e(),
                    0xD4 => self.set_2_h(),
                    0xD5 => self.set_2_l(),
                    0xD6 => self.set_2_hl(memory),
                    0xD7 => self.set_2_a(),
                    0xD8 => self.set_3_b(),
                    0xD9 => self.set_3_c(),
                    0xDA => self.set_3_d(),
                    0xDB => self.set_3_e(),
                    0xDC => self.set_3_h(),
                    0xDD => self.set_3_l(),
                    0xDE => self.set_3_hl(memory),
                    0xDF => self.set_3_a(),

                    0xE0 => self.set_4_b(),
                    0xE1 => self.set_4_c(),
                    0xE2 => self.set_4_d(),
                    0xE3 => self.set_4_e(),
                    0xE4 => self.set_4_h(),
                    0xE5 => self.set_4_l(),
                    0xE6 => self.set_4_hl(memory),
                    0xE7 => self.set_4_a(),
                    0xE8 => self.set_5_b(),
                    0xE9 => self.set_5_c(),
                    0xEA => self.set_5_d(),
                    0xEB => self.set_5_e(),
                    0xEC => self.set_5_h(),
                    0xED => self.set_5_l(),
                    0xEE => self.set_5_hl(memory),
                    0xEF => self.set_5_a(),

                    0xF0 => self.set_6_b(),
                    0xF1 => self.set_6_c(),
                    0xF2 => self.set_6_d(),
                    0xF3 => self.set_6_e(),
                    0xF4 => self.set_6_h(),
                    0xF5 => self.set_6_l(),
                    0xF6 => self.set_6_hl(memory),
                    0xF7 => self.set_6_a(),
                    0xF8 => self.set_7_b(),
                    0xF9 => self.set_7_c(),
                    0xFA => self.set_7_d(),
                    0xFB => self.set_7_e(),
                    0xFC => self.set_7_h(),
                    0xFD => self.set_7_l(),
                    0xFE => self.set_7_hl(memory),
                    0xFF => self.set_7_a(),
                }
            }
            0xCC => self.call_z_nn(memory),
            0xCD => self.call_nn(memory),
            0xCE => self.adc_a_n(memory),
            0xCF => self.rst_08(memory),

            0xD0 => self.ret_nc(memory),
            0xD1 => self.pop_de(memory),
            0xD2 => self.jp_nc_nn(memory),
            // 0xD3
            0xD4 => self.call_nc_nn(memory),
            0xD5 => self.push_de(memory),
            0xD6 => self.sub_n(memory),
            0xD7 => self.rst_10(memory),
            0xD8 => self.ret_c(memory),
            0xD9 => self.reti(memory),
            0xDA => self.jp_c_nn(memory),
            // 0xDB
            0xDC => self.call_c_nn(memory),
            // 0xDD
            0xDE => self.sbc_a_n(memory),
            0xDF => self.rst_18(memory),

            0xE0 => self.ldh_n_a(memory),
            0xE1 => self.pop_hl(memory),
            0xE2 => self.ldh_c_a(memory),
            // 0xE3
            // 0xE4
            0xE5 => self.push_hl(memory),
            0xE6 => self.and_n(memory),
            0xE7 => self.rst_20(memory),
            0xE8 => self.add_sp_n(memory),
            0xE9 => self.jp_hl(),
            0xEA => self.ld_nn_a(memory),
            // 0xEB
            // 0xEC
            // 0xED
            0xEE => self.xor_n(memory),
            0xEF => self.rst_28(memory),

            0xF0 => self.ldh_a_n(memory),
            0xF1 => self.pop_af(memory),
            0xF2 => self.ldh_a_c(memory),
            0xF3 => self.di(),
            // 0xF4
            0xF5 => self.push_af(memory),
            0xF6 => self.or_n(memory),
            0xF7 => self.rst_30(memory),
            0xF8 => self.ldhl_sp_n(memory),
            0xF9 => self.ld_sp_hl(),
            0xFA => self.ld_a_nn(memory),
            0xFB => self.ei(),
            // 0xFC
            // 0xFD
            0xFE => self.cp_n(memory),
            0xFF => self.rst_38(memory),

            op => panic!("Op code not implemented: {:02X}", op),
        };

        self.cycles += cycles;

        cycles
    }
}

impl CPU {
    fn handle_interrupts(&mut self, memory: &mut AddressBus) -> bool {
        if !self.ime && !self.halt {
            // if interrupts are not enabled and not halted
            return false;
        }

        let inte = memory.read_byte(0xFFFF);
        let mut intf = memory.read_byte(0xFF0F);

        let triggered = inte & intf;

        if triggered == 0x00 {
            // if none of the requested interrupts are enabled
            return false;
        }

        // if any of the requested interrupts are enabled, un-halt
        self.halt = false;

        if !self.ime {
            // if interrupts are not enabled
            return false;
        }

        // disable interrupts whilst processing the interrupt
        self.ime = false;

        // find the index of the first triggered interrupt
        let n = triggered.trailing_zeros();

        // disable the interrupt that's about to be executed
        use bit_field::BitField;
        intf.set_bit(n as usize, false);

        memory.write_byte(0xFF0F, intf);

        self.push(memory, self.registers.pc);

        self.registers.pc = match n {
            0 => 0x0040, // V-Blank
            1 => 0x0048, // LCD Stat
            2 => 0x0050, // Timer
            3 => 0x0058, // Serial
            4 => 0x0060, // Joypad
            _ => unreachable!(),
        };

        true
    }

    fn get_n(&mut self, memory: &AddressBus) -> u8 {
        let n = memory.read_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        n
    }

    fn get_nn(&mut self, memory: &AddressBus) -> u16 {
        let nn = memory.read_word(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(2);

        nn
    }

    fn push(&mut self, memory: &mut AddressBus, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        memory.write_word(self.registers.sp, value);
    }

    fn pop(&mut self, memory: &AddressBus) -> u16 {
        let pop = memory.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);

        pop
    }

    fn add(&mut self, value: u8) {
        let (result, overflow) = self.registers.a.overflowing_add(value);
        let half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;

        self.registers.a = result;

        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.remove(Flag::Subtract);
        self.registers.f.set(Flag::HalfCarry, half_carry);
        self.registers.f.set(Flag::Carry, overflow);
    }

    fn adc(&mut self, value: u8) {
        let carry = self.registers.f.contains(Flag::Carry) as u8;

        let (result, overflow) = self.registers.a.overflowing_add(value);
        let (result, carry_overflow) = result.overflowing_add(carry);
        let half_carry = (self.registers.a & 0xF) + (value & 0xF) + carry > 0xF;

        self.registers.a = result;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.set(Flag::HalfCarry, half_carry);
        self.registers
            .f
            .set(Flag::Carry, overflow || carry_overflow);
    }

    fn sub(&mut self, value: u8) {
        let (result, underflow) = self.registers.a.overflowing_sub(value);
        let half_carry = (value & 0x0F) > (self.registers.a & 0x0F);

        self.registers.a = result;

        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.insert(Flag::Subtract);
        self.registers.f.set(Flag::HalfCarry, half_carry);
        self.registers.f.set(Flag::Carry, underflow);
    }

    fn sbc(&mut self, value: u8) {
        let carry = self.registers.f.contains(Flag::Carry) as u8;

        let (result, underflow) = self.registers.a.overflowing_sub(value);
        let (result, carry_underflow) = result.overflowing_sub(carry);
        let half_carry = (value & 0x0F) + carry > (self.registers.a & 0x0F);

        self.registers.a = result;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.insert(Flag::Subtract);
        self.registers.f.set(Flag::HalfCarry, half_carry);
        self.registers
            .f
            .set(Flag::Carry, underflow || carry_underflow);
    }

    fn and(&mut self, value: u8) {
        self.registers.a &= value;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, self.registers.a == 0);
        self.registers.f.insert(Flag::HalfCarry);
    }

    fn xor(&mut self, value: u8) {
        self.registers.a ^= value;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, self.registers.a == 0);
    }

    fn or(&mut self, value: u8) {
        self.registers.a |= value;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, self.registers.a == 0);
    }

    fn cp(&mut self, value: u8) {
        let (result, underflow) = self.registers.a.overflowing_sub(value);
        let half_carry = (value & 0x0F) > (self.registers.a & 0x0F);

        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.insert(Flag::Subtract);
        self.registers.f.set(Flag::HalfCarry, half_carry);
        self.registers.f.set(Flag::Carry, underflow);
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        // Half Carry is set if the lower nibble of the value is equal to 0xF.
        // If the nibble is equal to 0xF (0b1111) that means incrementing the value
        // by 1 would cause a carry from the lower nibble to the upper nibble.
        let half_carry = (value & 0xF) == 0xF;

        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.remove(Flag::Subtract);
        self.registers.f.set(Flag::HalfCarry, half_carry);

        result
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        // Half Carry is set if the lower nibble of the value is equal to 0x0.
        // If the nibble is equal to 0x0 (0b0000) that means decrementing the value
        // by 1 would cause a carry from the upper nibble to the lower nibble.
        let half_carry = value.trailing_zeros() >= 4; // (value & 0xF) == 0x0;

        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.insert(Flag::Subtract);
        self.registers.f.set(Flag::HalfCarry, half_carry);

        result
    }

    fn add_hl(&mut self, value: u16) {
        let hl = self.registers.get_hl();

        let (result, overflow) = hl.overflowing_add(value);
        let half_carry = (hl & 0xFFF) + (value & 0xFFF) > 0xFFF;

        self.registers.set_hl(result);

        self.registers.f.remove(Flag::Subtract);
        self.registers.f.set(Flag::HalfCarry, half_carry);
        self.registers.f.set(Flag::Carry, overflow);
    }

    fn rlc(&mut self, value: u8) -> u8 {
        use bit_field::BitField;
        let bit7 = value.get_bit(7);

        self.registers.f.set(Flag::Carry, bit7);

        let result = value.rotate_left(1);

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.set(Flag::Carry, bit7);

        result
    }

    fn rl(&mut self, value: u8) -> u8 {
        use bit_field::BitField;

        let carry = self.registers.f.contains(Flag::Carry);
        let bit7 = value.get_bit(7);

        let mut result = value << 1;
        result |= carry as u8;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.set(Flag::Carry, bit7);

        result
    }

    fn rrc(&mut self, value: u8) -> u8 {
        use bit_field::BitField;
        let bit0 = value.get_bit(0);

        self.registers.f.set(Flag::Carry, bit0);

        let result = value.rotate_right(1);

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.set(Flag::Carry, bit0);

        result
    }

    fn rr(&mut self, value: u8) -> u8 {
        use bit_field::BitField;

        let carry = self.registers.f.contains(Flag::Carry);
        let bit0 = value.get_bit(0);

        let mut result = value >> 1;
        result |= (carry as u8) << 7;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.set(Flag::Carry, bit0);

        result
    }

    fn sla(&mut self, value: u8) -> u8 {
        use bit_field::BitField;
        let bit7 = value.get_bit(7);

        let result = value << 1;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.set(Flag::Carry, bit7);

        result
    }

    fn swap(&mut self, value: u8) -> u8 {
        use bit_field::BitField;
        let high = value.get_bits(4..8);
        let low = value.get_bits(0..4);

        let mut result = 0u8;
        result.set_bits(0..4, high);
        result.set_bits(4..8, low);

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);

        result
    }

    fn sra(&mut self, value: u8) -> u8 {
        use bit_field::BitField;
        let bit7 = value.get_bit(7);
        let bit0 = value.get_bit(0);

        let mut result = value >> 1;
        result.set_bit(7, bit7);

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.set(Flag::Carry, bit0);

        result
    }

    fn srl(&mut self, value: u8) -> u8 {
        use bit_field::BitField;
        let bit0 = value.get_bit(0);

        let result = value >> 1;

        self.registers.f.clear();
        self.registers.f.set(Flag::Zero, result == 0);
        self.registers.f.set(Flag::Carry, bit0);

        result
    }

    fn bit(&mut self, bit: usize, value: u8) {
        use bit_field::BitField;
        let bit = value.get_bit(bit);

        self.registers.f.set(Flag::Zero, !bit);
        self.registers.f.remove(Flag::Subtract);
        self.registers.f.insert(Flag::HalfCarry);
    }

    fn set(&mut self, bit: usize, value: u8) -> u8 {
        let mut value = value;

        use bit_field::BitField;
        *value.set_bit(bit, true)
    }

    fn res(&mut self, bit: usize, value: u8) -> u8 {
        let mut value = value;

        use bit_field::BitField;
        *value.set_bit(bit, false)
    }

    fn jr(&mut self, offset: u8) {
        let offset = offset as i8;

        self.registers.pc = if offset >= 0 {
            self.registers.pc.wrapping_add(offset as u16)
        } else {
            self.registers.pc.wrapping_sub(offset.abs() as u16)
        };
    }

    fn call(&mut self, memory: &mut AddressBus, value: u16) {
        self.push(memory, self.registers.pc);
        self.registers.pc = value;
    }

    // 0x00 - 0x0F

    // NOP
    fn nop(&mut self) -> usize {
        4
    }

    // LD BC,nn
    fn ld_bc_nn(&mut self, memory: &AddressBus) -> usize {
        let nn = self.get_nn(memory);
        self.registers.set_bc(nn);

        12
    }

    // LD (BC),A
    fn ld_bc_a(&mut self, memory: &mut AddressBus) -> usize {
        let bc = self.registers.get_bc();
        memory.write_byte(bc, self.registers.a);

        8
    }

    // INC BC
    fn inc_bc(&mut self) -> usize {
        let result = self.registers.get_bc().wrapping_add(1);
        self.registers.set_bc(result);

        8
    }

    // INC B
    fn inc_b(&mut self) -> usize {
        self.registers.b = self.inc(self.registers.b);

        4
    }

    // DEC B
    fn dec_b(&mut self) -> usize {
        self.registers.b = self.dec(self.registers.b);

        4
    }

    // LD B,n
    fn ld_b_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.registers.b = n;

        8
    }

    // RLCA
    fn rlca(&mut self) -> usize {
        use bit_field::BitField;
        let bit7 = self.registers.a.get_bit(7);

        self.registers.a = self.registers.a.rotate_left(1);

        self.registers.f.clear();
        self.registers.f.set(Flag::Carry, bit7);

        4
    }

    // LD (nn),SP
    fn ld_nn_sp(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);
        memory.write_word(nn, self.registers.sp);

        20
    }

    // ADD HL,BC
    fn add_hl_bc(&mut self) -> usize {
        self.add_hl(self.registers.get_bc());

        8
    }

    // LD A,(BC)
    fn ld_a_bc(&mut self, memory: &AddressBus) -> usize {
        let bc = self.registers.get_bc();
        self.registers.a = memory.read_byte(bc);

        8
    }

    // DEC BC
    fn dec_bc(&mut self) -> usize {
        let bc = self.registers.get_bc();
        self.registers.set_bc(bc.wrapping_sub(1));

        8
    }

    // INC C
    fn inc_c(&mut self) -> usize {
        self.registers.c = self.inc(self.registers.c);

        4
    }

    // DEC C
    fn dec_c(&mut self) -> usize {
        self.registers.c = self.dec(self.registers.c);

        4
    }

    // LD C,n
    fn ld_c_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.registers.c = n;

        8
    }

    // RRCA
    fn rrca(&mut self) -> usize {
        use bit_field::BitField;
        let bit0 = self.registers.a.get_bit(0);

        self.registers.a = self.registers.a.rotate_right(1);

        self.registers.f.clear();
        self.registers.f.set(Flag::Carry, bit0);

        4
    }

    // 0x10 - 0x1F

    // STOP
    fn stop(&mut self) -> usize {
        unimplemented!();
    }

    // LD DE,nn
    fn ld_de_nn(&mut self, memory: &AddressBus) -> usize {
        let nn = self.get_nn(memory);
        self.registers.set_de(nn);

        12
    }

    // LD (DE),A
    fn ld_de_a(&mut self, memory: &mut AddressBus) -> usize {
        let de = self.registers.get_de();
        memory.write_byte(de, self.registers.a);

        8
    }

    // INC DE
    fn inc_de(&mut self) -> usize {
        let de = self.registers.get_de();
        let result = de.wrapping_add(1);

        self.registers.set_de(result);

        8
    }

    // INC D
    fn inc_d(&mut self) -> usize {
        self.registers.d = self.inc(self.registers.d);

        4
    }

    // DEC D
    fn dec_d(&mut self) -> usize {
        self.registers.d = self.dec(self.registers.d);

        4
    }

    // LD D,n
    fn ld_d_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        self.registers.d = n;

        8
    }

    // RLA
    fn rla(&mut self) -> usize {
        use bit_field::BitField;

        let carry = self.registers.f.contains(Flag::Carry);
        let bit7 = self.registers.a.get_bit(7);

        let mut result = self.registers.a << 1;
        result |= carry as u8;

        self.registers.a = result;

        self.registers.f.clear();
        self.registers.f.set(Flag::Carry, bit7);

        4
    }

    // JR n
    fn jr_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        //self.registers.pc = self.registers.pc.wrapping_add(i16::from(n as i8) as u16);
        self.jr(n);

        12
    }

    // ADD HL,DE
    fn add_hl_de(&mut self) -> usize {
        self.add_hl(self.registers.get_de());

        8
    }

    // LD A,(DE)
    fn ld_a_de(&mut self, memory: &AddressBus) -> usize {
        let de = self.registers.get_de();
        self.registers.a = memory.read_byte(de);

        8
    }

    // DEC DE
    fn dec_de(&mut self) -> usize {
        let de = self.registers.get_de();
        self.registers.set_de(de.wrapping_sub(1));

        8
    }

    // INC E
    fn inc_e(&mut self) -> usize {
        self.registers.e = self.inc(self.registers.e);

        4
    }

    // DEC E
    fn dec_e(&mut self) -> usize {
        self.registers.e = self.dec(self.registers.e);

        4
    }

    // LD E,n
    fn ld_e_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.registers.e = n;

        8
    }

    // RRA
    fn rra(&mut self) -> usize {
        use bit_field::BitField;

        let carry = self.registers.f.contains(Flag::Carry);
        let bit0 = self.registers.a.get_bit(0);

        let mut result = self.registers.a >> 1;
        result |= (carry as u8) << 7;

        self.registers.a = result;

        self.registers.f.clear();
        self.registers.f.set(Flag::Carry, bit0);

        4
    }

    // 0x20 - 0x2F

    // JR NZ,n
    fn jr_nz_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);

        if !self.registers.f.contains(Flag::Zero) {
            // Can jump a max of 128 bytes in either direction, hence the weird chain of casts
            //self.registers.pc = self.registers.pc.wrapping_add(i16::from(n as i8) as u16);
            self.jr(n);

            12
        } else {
            8
        }
    }

    // LD HL,nn
    fn ld_hl_nn(&mut self, memory: &AddressBus) -> usize {
        let nn = self.get_nn(memory);
        self.registers.set_hl(nn);

        12
    }

    // LD (HL+),A
    fn ldi_hl_a(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.a);

        let hl = hl.wrapping_add(1);
        self.registers.set_hl(hl);

        8
    }

    // INC HL
    fn inc_hl(&mut self) -> usize {
        let result = self.registers.get_hl().wrapping_add(1);
        self.registers.set_hl(result);

        8
    }

    // INC H
    fn inc_h(&mut self) -> usize {
        self.registers.h = self.inc(self.registers.h);

        4
    }

    // DEC H
    fn dec_h(&mut self) -> usize {
        self.registers.h = self.dec(self.registers.h);

        4
    }

    // LD H,n
    fn ld_h_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.registers.h = n;

        8
    }

    // DAA
    fn daa(&mut self) -> usize {
        let mut a = self.registers.a;
        let mut adjust = if self.registers.f.contains(Flag::Carry) {
            0x60
        } else {
            0x00
        };

        if self.registers.f.contains(Flag::HalfCarry) {
            adjust |= 0x06;
        };

        if !self.registers.f.contains(Flag::Subtract) {
            if a & 0x0F > 0x09 {
                adjust |= 0x06;
            };
            if a > 0x99 {
                adjust |= 0x60;
            };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.registers.f.set(Flag::Zero, a == 0);
        self.registers.f.remove(Flag::HalfCarry);
        self.registers.f.set(Flag::Carry, adjust >= 0x60);

        self.registers.a = a;

        4
    }

    // JR Z,n
    fn jr_z_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);

        if self.registers.f.contains(Flag::Zero) {
            // Can jump a max of 128 bytes in either direction, hence the weird chain of casts
            //self.registers.pc = self.registers.pc.wrapping_add(i16::from(n as i8) as u16);
            self.jr(n);

            12
        } else {
            8
        }
    }

    // ADD HL,Hl
    fn add_hl_hl(&mut self) -> usize {
        self.add_hl(self.registers.get_hl());

        8
    }

    // LD A,(HL+)
    fn ldi_a_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.a = memory.read_byte(hl);

        let hl = hl.wrapping_add(1);
        self.registers.set_hl(hl);

        8
    }

    // DEC HL
    fn dec_hl(&mut self) -> usize {
        let hl = self.registers.get_hl();
        self.registers.set_hl(hl.wrapping_sub(1));

        8
    }

    // INC L
    fn inc_l(&mut self) -> usize {
        self.registers.l = self.inc(self.registers.l);

        4
    }

    // DEC L
    fn dec_l(&mut self) -> usize {
        self.registers.l = self.dec(self.registers.l);

        4
    }

    // LD L,n
    fn ld_l_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.registers.l = n;

        8
    }

    // CPL
    fn cpl(&mut self) -> usize {
        self.registers.a = !self.registers.a;
        self.registers.f.insert(Flag::Subtract | Flag::HalfCarry);

        4
    }

    // 0x30 - 0x3F

    // JR NC,n
    fn jr_nc_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);

        if !self.registers.f.contains(Flag::Carry) {
            // Can jump a max of 128 bytes in either direction, hence the weird chain of casts
            //self.registers.pc = self.registers.pc.wrapping_add(i16::from(n as i8) as u16);
            self.jr(n);

            12
        } else {
            8
        }
    }

    // LD SP,nn
    fn ld_sp_nn(&mut self, memory: &AddressBus) -> usize {
        let nn = self.get_nn(memory);
        self.registers.sp = nn;

        12
    }

    // INC SP
    fn inc_sp(&mut self) -> usize {
        self.registers.sp = self.registers.sp.wrapping_add(1);

        8
    }

    // INC (HL)
    fn inc_hl_ref(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.inc(n);
        memory.write_byte(hl, result);

        12
    }

    // DEC (HL)
    fn dec_hl_ref(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.dec(n);
        memory.write_byte(hl, result);

        12
    }

    // LD (HL),n
    fn ld_hl_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);

        let hl = self.registers.get_hl();
        memory.write_byte(hl, n);

        12
    }

    // SCF
    fn scf(&mut self) -> usize {
        self.registers.f.remove(Flag::Subtract);
        self.registers.f.remove(Flag::HalfCarry);
        self.registers.f.insert(Flag::Carry);

        4
    }

    // JR C,n
    fn jr_c_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);

        if self.registers.f.contains(Flag::Carry) {
            // Can jump a max of 128 bytes in either direction, hence the weird chain of casts
            //self.registers.pc = self.registers.pc.wrapping_add(i16::from(n as i8) as u16);
            self.jr(n);

            12
        } else {
            8
        }
    }

    // ADD HL,SP
    fn add_hl_sp(&mut self) -> usize {
        self.add_hl(self.registers.sp);

        8
    }

    // LD A,(HL-)
    fn ldd_a_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.a = memory.read_byte(hl);

        let hl = hl.wrapping_sub(1);
        self.registers.set_hl(hl);

        8
    }

    // DEC SP
    fn dec_sp(&mut self) -> usize {
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        8
    }

    // INC A
    fn inc_a(&mut self) -> usize {
        self.registers.a = self.inc(self.registers.a);

        4
    }

    // DEC A
    fn dec_a(&mut self) -> usize {
        self.registers.a = self.dec(self.registers.a);

        4
    }

    // LD A,n
    fn ld_a_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.registers.a = n;

        8
    }

    // CCF
    fn ccf(&mut self) -> usize {
        self.registers.f.remove(Flag::Subtract);
        self.registers.f.remove(Flag::HalfCarry);
        self.registers
            .f
            .set(Flag::Carry, !self.registers.f.contains(Flag::Carry));

        4
    }

    // 0x40 - 0x4F

    // LD B,B
    fn ld_b_b(&mut self) -> usize {
        self.registers.b = self.registers.b;

        4
    }

    // LD B,C
    fn ld_b_c(&mut self) -> usize {
        self.registers.b = self.registers.c;

        4
    }

    // LD B,D
    fn ld_b_d(&mut self) -> usize {
        self.registers.b = self.registers.d;

        4
    }

    // LD B,E
    fn ld_b_e(&mut self) -> usize {
        self.registers.b = self.registers.e;

        4
    }

    // LD B,H
    fn ld_b_h(&mut self) -> usize {
        self.registers.b = self.registers.h;

        4
    }

    // LD B,L
    fn ld_b_l(&mut self) -> usize {
        self.registers.b = self.registers.l;

        4
    }

    // LD B,(HL)
    fn ld_b_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.b = memory.read_byte(hl);

        8
    }

    // LD B,A
    fn ld_b_a(&mut self) -> usize {
        self.registers.b = self.registers.a;

        4
    }

    // LD C,B
    fn ld_c_b(&mut self) -> usize {
        self.registers.c = self.registers.b;

        4
    }

    // LD C,C
    fn ld_c_c(&mut self) -> usize {
        self.registers.c = self.registers.c;

        4
    }

    // LD C,D
    fn ld_c_d(&mut self) -> usize {
        self.registers.c = self.registers.d;

        4
    }

    // LD C,E
    fn ld_c_e(&mut self) -> usize {
        self.registers.c = self.registers.e;

        4
    }

    // LD C,H
    fn ld_c_h(&mut self) -> usize {
        self.registers.c = self.registers.h;

        4
    }

    // LD C,L
    fn ld_c_l(&mut self) -> usize {
        self.registers.c = self.registers.l;

        4
    }

    // LD C,(HL)
    fn ld_c_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.c = memory.read_byte(hl);

        8
    }

    // LD C,A
    fn ld_c_a(&mut self) -> usize {
        self.registers.c = self.registers.a;

        4
    }

    // LD D,B
    fn ld_d_b(&mut self) -> usize {
        self.registers.d = self.registers.b;

        4
    }

    // LD D,C
    fn ld_d_c(&mut self) -> usize {
        self.registers.d = self.registers.c;

        4
    }

    // LD D,D
    fn ld_d_d(&mut self) -> usize {
        self.registers.d = self.registers.d;

        4
    }

    // LD D,E
    fn ld_d_e(&mut self) -> usize {
        self.registers.d = self.registers.e;

        4
    }

    // LD D,H
    fn ld_d_h(&mut self) -> usize {
        self.registers.d = self.registers.h;

        4
    }

    // LD D,L
    fn ld_d_l(&mut self) -> usize {
        self.registers.d = self.registers.l;

        4
    }

    // LD D,(HL)
    fn ld_d_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.d = memory.read_byte(hl);

        8
    }

    // LD D,A
    fn ld_d_a(&mut self) -> usize {
        self.registers.d = self.registers.a;

        4
    }

    // LD E,B
    fn ld_e_b(&mut self) -> usize {
        self.registers.e = self.registers.b;

        4
    }

    // LD E,C
    fn ld_e_c(&mut self) -> usize {
        self.registers.e = self.registers.c;

        4
    }

    // LD E,D
    fn ld_e_d(&mut self) -> usize {
        self.registers.e = self.registers.d;

        4
    }

    // LD E,E
    fn ld_e_e(&mut self) -> usize {
        self.registers.e = self.registers.e;

        4
    }

    // LD E,H
    fn ld_e_h(&mut self) -> usize {
        self.registers.e = self.registers.h;

        4
    }

    // LD E,L
    fn ld_e_l(&mut self) -> usize {
        self.registers.e = self.registers.l;

        4
    }

    // LD E,(HL)
    fn ld_e_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.e = memory.read_byte(hl);

        8
    }

    // LD E,A
    fn ld_e_a(&mut self) -> usize {
        self.registers.e = self.registers.a;

        4
    }

    // LD H,B
    fn ld_h_b(&mut self) -> usize {
        self.registers.h = self.registers.b;

        4
    }

    // LD H,C
    fn ld_h_c(&mut self) -> usize {
        self.registers.h = self.registers.c;

        4
    }

    // LD H,D
    fn ld_h_d(&mut self) -> usize {
        self.registers.h = self.registers.d;

        4
    }

    // LD H,E
    fn ld_h_e(&mut self) -> usize {
        self.registers.h = self.registers.e;

        4
    }

    // LD H,H
    fn ld_h_h(&mut self) -> usize {
        self.registers.h = self.registers.h;

        4
    }

    // LD H,L
    fn ld_h_l(&mut self) -> usize {
        self.registers.h = self.registers.l;

        4
    }

    // LD H,(HL)
    fn ld_h_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.h = memory.read_byte(hl);

        8
    }

    // LD H,A
    fn ld_h_a(&mut self) -> usize {
        self.registers.h = self.registers.a;

        4
    }

    // LD L,B
    fn ld_l_b(&mut self) -> usize {
        self.registers.l = self.registers.b;

        4
    }

    // LD L,C
    fn ld_l_c(&mut self) -> usize {
        self.registers.l = self.registers.c;

        4
    }

    // LD L,D
    fn ld_l_d(&mut self) -> usize {
        self.registers.l = self.registers.d;

        4
    }

    // LD L,E
    fn ld_l_e(&mut self) -> usize {
        self.registers.l = self.registers.e;

        4
    }

    // LD L,H
    fn ld_l_h(&mut self) -> usize {
        self.registers.l = self.registers.h;

        4
    }

    // LD L,L
    fn ld_l_l(&mut self) -> usize {
        self.registers.l = self.registers.l;

        4
    }

    // LD L,(HL)
    fn ld_l_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.l = memory.read_byte(hl);

        8
    }

    // LD L,A
    fn ld_l_a(&mut self) -> usize {
        self.registers.l = self.registers.a;

        4
    }

    // LD (HL),B
    fn ld_hl_b(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.b);

        8
    }

    // LD (HL),C
    fn ld_hl_c(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.c);

        8
    }

    // LD (HL),D
    fn ld_hl_d(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.d);

        8
    }

    // LD (HL),E
    fn ld_hl_e(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.e);

        8
    }

    // LD (HL),H
    fn ld_hl_h(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.h);

        8
    }

    // LD (HL),L
    fn ld_hl_l(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.l);

        8
    }

    // HALT
    fn halt(&mut self) -> usize {
        self.halt = true;

        4
    }

    // LD (HL),A
    fn ld_hl_a(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.a);

        8
    }

    // LD (HL-),A
    fn ldd_hl_a(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        memory.write_byte(hl, self.registers.a);

        let hl = hl.wrapping_sub(1);
        self.registers.set_hl(hl);

        8
    }

    // LD A,B
    fn ld_a_b(&mut self) -> usize {
        self.registers.a = self.registers.b;

        4
    }

    // LD A,C
    fn ld_a_c(&mut self) -> usize {
        self.registers.a = self.registers.c;

        4
    }

    // LD A,D
    fn ld_a_d(&mut self) -> usize {
        self.registers.a = self.registers.d;

        4
    }

    // LD A,E
    fn ld_a_e(&mut self) -> usize {
        self.registers.a = self.registers.e;

        4
    }

    // LD A,H
    fn ld_a_h(&mut self) -> usize {
        self.registers.a = self.registers.h;

        4
    }

    // LD A,L
    fn ld_a_l(&mut self) -> usize {
        self.registers.a = self.registers.l;

        4
    }

    // LD A,(HL)
    fn ld_a_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.registers.a = memory.read_byte(hl);

        8
    }

    // LD A,A
    fn ld_a_a(&mut self) -> usize {
        self.registers.a = self.registers.a;

        4
    }

    // ADD A,B
    fn add_a_b(&mut self) -> usize {
        self.add(self.registers.b);

        4
    }

    // ADD A,C
    fn add_a_c(&mut self) -> usize {
        self.add(self.registers.c);

        4
    }

    // ADD A,D
    fn add_a_d(&mut self) -> usize {
        self.add(self.registers.d);

        4
    }

    // ADD A,E
    fn add_a_e(&mut self) -> usize {
        self.add(self.registers.e);

        4
    }

    // ADD A,H
    fn add_a_h(&mut self) -> usize {
        self.add(self.registers.h);

        4
    }

    // ADD A,L
    fn add_a_l(&mut self) -> usize {
        self.add(self.registers.l);

        4
    }

    // ADD A,(Hl)
    fn add_a_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.add(n);

        8
    }

    // ADD A,A
    fn add_a_a(&mut self) -> usize {
        self.add(self.registers.a);

        4
    }

    // ADC A,B
    fn adc_a_b(&mut self) -> usize {
        self.adc(self.registers.b);

        4
    }

    // ADC A,C
    fn adc_a_c(&mut self) -> usize {
        self.adc(self.registers.c);

        4
    }

    // ADC A,D
    fn adc_a_d(&mut self) -> usize {
        self.adc(self.registers.d);

        4
    }

    // ADC A,E
    fn adc_a_e(&mut self) -> usize {
        self.adc(self.registers.e);

        4
    }

    // ADC A,H
    fn adc_a_h(&mut self) -> usize {
        self.adc(self.registers.h);

        4
    }

    // ADC A,L
    fn adc_a_l(&mut self) -> usize {
        self.adc(self.registers.l);

        4
    }

    // ADC A,(HL)
    fn adc_a_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.adc(n);

        8
    }

    // ADC A,A
    fn adc_a_a(&mut self) -> usize {
        self.adc(self.registers.a);

        4
    }

    // SUB B
    fn sub_b(&mut self) -> usize {
        self.sub(self.registers.b);

        4
    }

    // SUB C
    fn sub_c(&mut self) -> usize {
        self.sub(self.registers.c);

        4
    }

    // SUB D
    fn sub_d(&mut self) -> usize {
        self.sub(self.registers.d);

        4
    }

    // SUB E
    fn sub_e(&mut self) -> usize {
        self.sub(self.registers.e);

        4
    }

    // SUB H
    fn sub_h(&mut self) -> usize {
        self.sub(self.registers.h);

        4
    }

    // SUB L
    fn sub_l(&mut self) -> usize {
        self.sub(self.registers.l);

        4
    }

    // SUB (HL)
    fn sub_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.sub(n);

        8
    }

    // SUB A
    fn sub_a(&mut self) -> usize {
        self.sub(self.registers.a);

        4
    }

    // SBC A,B
    fn sbc_a_b(&mut self) -> usize {
        self.sbc(self.registers.b);

        4
    }

    // SBC A,C
    fn sbc_a_c(&mut self) -> usize {
        self.sbc(self.registers.c);

        4
    }

    // SBC A,D
    fn sbc_a_d(&mut self) -> usize {
        self.sbc(self.registers.d);

        4
    }

    // SBC A,E
    fn sbc_a_e(&mut self) -> usize {
        self.sbc(self.registers.e);

        4
    }

    // SBC A,H
    fn sbc_a_h(&mut self) -> usize {
        self.sbc(self.registers.h);

        4
    }

    // SBC A,L
    fn sbc_a_l(&mut self) -> usize {
        self.sbc(self.registers.l);

        4
    }

    // SBC A,(HL)
    fn sbc_a_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.sbc(n);

        8
    }

    // SBC A,A
    fn sbc_a_a(&mut self) -> usize {
        self.sbc(self.registers.a);

        4
    }

    // AND B
    fn and_b(&mut self) -> usize {
        self.and(self.registers.b);

        4
    }

    // AND C
    fn and_c(&mut self) -> usize {
        self.and(self.registers.c);

        4
    }

    // AND D
    fn and_d(&mut self) -> usize {
        self.and(self.registers.d);

        4
    }

    // AND E
    fn and_e(&mut self) -> usize {
        self.and(self.registers.e);

        4
    }

    // AND H
    fn and_h(&mut self) -> usize {
        self.and(self.registers.h);

        4
    }

    // AND L
    fn and_l(&mut self) -> usize {
        self.and(self.registers.l);

        4
    }

    // AND (HL)
    fn and_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.and(n);

        8
    }

    // AND A
    fn and_a(&mut self) -> usize {
        self.and(self.registers.a);

        4
    }

    // XOR B
    fn xor_b(&mut self) -> usize {
        self.xor(self.registers.b);

        4
    }

    // XOR C
    fn xor_c(&mut self) -> usize {
        self.xor(self.registers.c);

        4
    }

    // XOR D
    fn xor_d(&mut self) -> usize {
        self.xor(self.registers.d);

        4
    }

    // XOR E
    fn xor_e(&mut self) -> usize {
        self.xor(self.registers.e);

        4
    }

    // XOR H
    fn xor_h(&mut self) -> usize {
        self.xor(self.registers.h);

        4
    }

    // XOR L
    fn xor_l(&mut self) -> usize {
        self.xor(self.registers.l);

        4
    }

    // XOR (HL)
    fn xor_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.xor(n);

        8
    }

    // XOR A
    fn xor_a(&mut self) -> usize {
        self.xor(self.registers.a);

        4
    }

    // OR B
    fn or_b(&mut self) -> usize {
        self.or(self.registers.b);

        4
    }

    // OR C
    fn or_c(&mut self) -> usize {
        self.or(self.registers.c);

        4
    }

    // OR D
    fn or_d(&mut self) -> usize {
        self.or(self.registers.d);

        4
    }

    // OR E
    fn or_e(&mut self) -> usize {
        self.or(self.registers.e);

        4
    }

    // OR H
    fn or_h(&mut self) -> usize {
        self.or(self.registers.h);

        4
    }

    // OR L
    fn or_l(&mut self) -> usize {
        self.or(self.registers.l);

        4
    }

    // OR (HL)
    fn or_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.or(n);

        8
    }

    // OR A
    fn or_a(&mut self) -> usize {
        self.or(self.registers.a);

        4
    }

    // CP B
    fn cp_b(&mut self) -> usize {
        self.cp(self.registers.b);

        4
    }

    // CP C
    fn cp_c(&mut self) -> usize {
        self.cp(self.registers.c);

        4
    }

    // CP D
    fn cp_d(&mut self) -> usize {
        self.cp(self.registers.d);

        4
    }

    // CP E
    fn cp_e(&mut self) -> usize {
        self.cp(self.registers.e);

        4
    }

    // CP H
    fn cp_h(&mut self) -> usize {
        self.cp(self.registers.h);

        4
    }

    // CP L
    fn cp_l(&mut self) -> usize {
        self.cp(self.registers.l);

        4
    }

    // CP (HL)
    fn cp_hl(&mut self, memory: &AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.cp(n);

        8
    }

    // CP A
    fn cp_a(&mut self) -> usize {
        self.cp(self.registers.a);

        4
    }

    // 0xC0 - 0xCF

    // RET NZ
    fn ret_nz(&mut self, memory: &AddressBus) -> usize {
        if !self.registers.f.contains(Flag::Zero) {
            self.registers.pc = self.pop(memory);

            20
        } else {
            8
        }
    }

    // POP BC
    fn pop_bc(&mut self, memory: &AddressBus) -> usize {
        let pop = self.pop(memory);
        self.registers.set_bc(pop);

        12
    }

    // JP NZ,nn
    fn jp_nz_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);

        if !self.registers.f.contains(Flag::Zero) {
            self.registers.pc = nn;

            16
        } else {
            12
        }
    }

    // JP nn
    fn jp_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);
        self.registers.pc = nn;

        16
    }

    // CALL NZ,nn
    fn call_nz_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);

        if !self.registers.f.contains(Flag::Zero) {
            self.call(memory, nn);

            24
        } else {
            12
        }
    }

    // PUSH BC
    fn push_bc(&mut self, memory: &mut AddressBus) -> usize {
        let bc = self.registers.get_bc();
        self.push(memory, bc);

        16
    }

    // ADD A,n
    fn add_a_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.add(n);

        8
    }

    // RST 00H
    fn rst_00(&mut self, memory: &mut AddressBus) -> usize {
        self.call(memory, 0x00);

        16
    }

    // RET Z
    fn ret_z(&mut self, memory: &AddressBus) -> usize {
        if self.registers.f.contains(Flag::Zero) {
            let pop = self.pop(memory);
            self.registers.pc = pop;

            20
        } else {
            8
        }
    }

    // RET
    fn ret(&mut self, memory: &AddressBus) -> usize {
        let pop = self.pop(memory);
        self.registers.pc = pop;

        16
    }

    // JP Z,nn
    fn jp_z_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);

        if self.registers.f.contains(Flag::Zero) {
            self.registers.pc = nn;

            16
        } else {
            12
        }
    }

    // RLC B
    fn rlc_b(&mut self) -> usize {
        self.registers.b = self.rlc(self.registers.b);

        8
    }

    // RLC C
    fn rlc_c(&mut self) -> usize {
        self.registers.c = self.rlc(self.registers.c);

        8
    }

    // RLC D
    fn rlc_d(&mut self) -> usize {
        self.registers.d = self.rlc(self.registers.d);

        8
    }

    // RLC E
    fn rlc_e(&mut self) -> usize {
        self.registers.e = self.rlc(self.registers.e);

        8
    }

    // RLC H
    fn rlc_h(&mut self) -> usize {
        self.registers.h = self.rlc(self.registers.h);

        8
    }

    // RLC L
    fn rlc_l(&mut self) -> usize {
        self.registers.l = self.rlc(self.registers.l);

        8
    }

    // RLC (HL)
    fn rlc_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.rlc(n);

        memory.write_byte(hl, result);

        16
    }

    // RLC A
    fn rlc_a(&mut self) -> usize {
        self.registers.a = self.rlc(self.registers.a);

        8
    }

    // RRC B
    fn rrc_b(&mut self) -> usize {
        self.registers.b = self.rrc(self.registers.b);

        8
    }

    // RRC C
    fn rrc_c(&mut self) -> usize {
        self.registers.c = self.rrc(self.registers.c);

        8
    }

    // RRC D
    fn rrc_d(&mut self) -> usize {
        self.registers.d = self.rrc(self.registers.d);

        8
    }

    // RRC E
    fn rrc_e(&mut self) -> usize {
        self.registers.e = self.rrc(self.registers.e);

        8
    }

    // RRC H
    fn rrc_h(&mut self) -> usize {
        self.registers.h = self.rrc(self.registers.h);

        8
    }

    // RRC L
    fn rrc_l(&mut self) -> usize {
        self.registers.l = self.rrc(self.registers.l);

        8
    }

    // RRC (HL)
    fn rrc_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.rrc(n);
        memory.write_byte(hl, result);

        16
    }

    // RRC A
    fn rrc_a(&mut self) -> usize {
        self.registers.a = self.rrc(self.registers.a);

        8
    }

    // RL B
    fn rl_b(&mut self) -> usize {
        self.registers.b = self.rl(self.registers.b);

        8
    }

    // RL C
    fn rl_c(&mut self) -> usize {
        self.registers.c = self.rl(self.registers.c);

        8
    }

    // RL D
    fn rl_d(&mut self) -> usize {
        self.registers.d = self.rl(self.registers.d);

        8
    }

    // RL E
    fn rl_e(&mut self) -> usize {
        self.registers.e = self.rl(self.registers.e);

        8
    }

    // RL H
    fn rl_h(&mut self) -> usize {
        self.registers.h = self.rl(self.registers.h);

        8
    }

    // RL L
    fn rl_l(&mut self) -> usize {
        self.registers.l = self.rl(self.registers.l);

        8
    }

    // RL (HL)
    fn rl_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.rl(n);
        memory.write_byte(hl, result);

        16
    }

    // RL A
    fn rl_a(&mut self) -> usize {
        self.registers.a = self.rl(self.registers.a);

        8
    }

    // RR B
    fn rr_b(&mut self) -> usize {
        self.registers.b = self.rr(self.registers.b);

        8
    }

    // RR C
    fn rr_c(&mut self) -> usize {
        self.registers.c = self.rr(self.registers.c);

        8
    }

    // RR D
    fn rr_d(&mut self) -> usize {
        self.registers.d = self.rr(self.registers.d);

        8
    }

    // RR E
    fn rr_e(&mut self) -> usize {
        self.registers.e = self.rr(self.registers.e);

        8
    }

    // RR H
    fn rr_h(&mut self) -> usize {
        self.registers.h = self.rr(self.registers.h);

        8
    }

    // RR L
    fn rr_l(&mut self) -> usize {
        self.registers.l = self.rr(self.registers.l);

        8
    }

    // RR (HL)
    fn rr_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.rr(n);
        memory.write_byte(hl, result);

        16
    }

    // RR A
    fn rr_a(&mut self) -> usize {
        self.registers.a = self.rr(self.registers.a);

        8
    }

    // SLA B
    fn sla_b(&mut self) -> usize {
        self.registers.b = self.sla(self.registers.b);

        8
    }

    // SLA C
    fn sla_c(&mut self) -> usize {
        self.registers.c = self.sla(self.registers.c);

        8
    }

    // SLA D
    fn sla_d(&mut self) -> usize {
        self.registers.d = self.sla(self.registers.d);

        8
    }

    // SLA E
    fn sla_e(&mut self) -> usize {
        self.registers.e = self.sla(self.registers.e);

        8
    }

    // SLA H
    fn sla_h(&mut self) -> usize {
        self.registers.h = self.sla(self.registers.h);

        8
    }

    // SLA L
    fn sla_l(&mut self) -> usize {
        self.registers.l = self.sla(self.registers.l);

        8
    }

    // SLA (HL)
    fn sla_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.sla(n);
        memory.write_byte(hl, result);

        16
    }

    // SLA A
    fn sla_a(&mut self) -> usize {
        self.registers.a = self.sla(self.registers.a);

        8
    }

    // SRA B
    fn sra_b(&mut self) -> usize {
        self.registers.b = self.sra(self.registers.b);

        8
    }

    // SRA C
    fn sra_c(&mut self) -> usize {
        self.registers.c = self.sra(self.registers.c);

        8
    }

    // SRA D
    fn sra_d(&mut self) -> usize {
        self.registers.d = self.sra(self.registers.d);

        8
    }

    // SRA E
    fn sra_e(&mut self) -> usize {
        self.registers.e = self.sra(self.registers.e);

        8
    }

    // SRA H
    fn sra_h(&mut self) -> usize {
        self.registers.h = self.sra(self.registers.h);

        8
    }

    // SRA L
    fn sra_l(&mut self) -> usize {
        self.registers.l = self.sra(self.registers.l);

        8
    }

    // SRA (HL)
    fn sra_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.sra(n);
        memory.write_byte(hl, result);

        16
    }

    // SRA A
    fn sra_a(&mut self) -> usize {
        self.registers.a = self.sra(self.registers.a);

        8
    }

    // SWAP B
    fn swap_b(&mut self) -> usize {
        self.registers.b = self.swap(self.registers.b);

        8
    }

    // SWAP C
    fn swap_c(&mut self) -> usize {
        self.registers.c = self.swap(self.registers.c);

        8
    }

    // SWAP D
    fn swap_d(&mut self) -> usize {
        self.registers.d = self.swap(self.registers.d);

        8
    }

    // SWAP E
    fn swap_e(&mut self) -> usize {
        self.registers.e = self.swap(self.registers.e);

        8
    }

    // SWAP H
    fn swap_h(&mut self) -> usize {
        self.registers.h = self.swap(self.registers.h);

        8
    }

    // SWAP L
    fn swap_l(&mut self) -> usize {
        self.registers.l = self.swap(self.registers.l);

        8
    }

    // SWAP (HL)
    fn swap_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.swap(n);
        memory.write_byte(hl, result);

        16
    }

    // SWAP A
    fn swap_a(&mut self) -> usize {
        self.registers.a = self.swap(self.registers.a);

        8
    }

    // SRL B
    fn srl_b(&mut self) -> usize {
        self.registers.b = self.srl(self.registers.b);

        8
    }

    // SRL C
    fn srl_c(&mut self) -> usize {
        self.registers.c = self.srl(self.registers.c);

        8
    }

    // SRL D
    fn srl_d(&mut self) -> usize {
        self.registers.d = self.srl(self.registers.d);

        8
    }

    // SRL E
    fn srl_e(&mut self) -> usize {
        self.registers.e = self.srl(self.registers.e);

        8
    }

    // SRL H
    fn srl_h(&mut self) -> usize {
        self.registers.h = self.srl(self.registers.h);

        8
    }

    // SRL L
    fn srl_l(&mut self) -> usize {
        self.registers.l = self.srl(self.registers.l);

        8
    }

    // SRL (HL)
    fn srl_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.srl(n);
        memory.write_byte(hl, result);

        16
    }

    // SRL A
    fn srl_a(&mut self) -> usize {
        self.registers.a = self.srl(self.registers.a);

        8
    }

    // BIT 0,B
    fn bit_0_b(&mut self) -> usize {
        self.bit(0, self.registers.b);

        8
    }

    // BIT 0,C
    fn bit_0_c(&mut self) -> usize {
        self.bit(0, self.registers.c);

        8
    }

    // BIT 0,D
    fn bit_0_d(&mut self) -> usize {
        self.bit(0, self.registers.d);

        8
    }

    // BIT 0,E
    fn bit_0_e(&mut self) -> usize {
        self.bit(0, self.registers.e);

        8
    }

    // BIT 0,H
    fn bit_0_h(&mut self) -> usize {
        self.bit(0, self.registers.h);

        8
    }

    // BIT 0,L
    fn bit_0_l(&mut self) -> usize {
        self.bit(0, self.registers.l);

        8
    }

    // BIT 0,(HL)
    fn bit_0_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.bit(0, n);

        12
    }

    // BIT 0,A
    fn bit_0_a(&mut self) -> usize {
        self.bit(0, self.registers.a);

        8
    }

    // BIT 1,B
    fn bit_1_b(&mut self) -> usize {
        self.bit(1, self.registers.b);

        8
    }

    // BIT 1,C
    fn bit_1_c(&mut self) -> usize {
        self.bit(1, self.registers.c);

        8
    }

    // BIT 1,D
    fn bit_1_d(&mut self) -> usize {
        self.bit(1, self.registers.d);

        8
    }

    // BIT 1,E
    fn bit_1_e(&mut self) -> usize {
        self.bit(1, self.registers.e);

        8
    }

    // BIT 1,H
    fn bit_1_h(&mut self) -> usize {
        self.bit(1, self.registers.h);

        8
    }

    // BIT 1,L
    fn bit_1_l(&mut self) -> usize {
        self.bit(1, self.registers.l);

        8
    }

    // BIT 1,(HL)
    fn bit_1_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.bit(1, n);

        12
    }

    // BIT 1,A
    fn bit_1_a(&mut self) -> usize {
        self.bit(1, self.registers.a);

        8
    }

    // BIT 2,B
    fn bit_2_b(&mut self) -> usize {
        self.bit(2, self.registers.b);

        8
    }

    // BIT 2,C
    fn bit_2_c(&mut self) -> usize {
        self.bit(2, self.registers.c);

        8
    }

    // BIT 2,D
    fn bit_2_d(&mut self) -> usize {
        self.bit(2, self.registers.d);

        8
    }

    // BIT 2,E
    fn bit_2_e(&mut self) -> usize {
        self.bit(2, self.registers.e);

        8
    }

    // BIT 2,H
    fn bit_2_h(&mut self) -> usize {
        self.bit(2, self.registers.h);

        8
    }

    // BIT 2,L
    fn bit_2_l(&mut self) -> usize {
        self.bit(2, self.registers.l);

        8
    }

    // BIT 2,(HL)
    fn bit_2_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.bit(2, n);

        12
    }

    // BIT 2,A
    fn bit_2_a(&mut self) -> usize {
        self.bit(2, self.registers.a);

        8
    }

    // BIT 3,B
    fn bit_3_b(&mut self) -> usize {
        self.bit(3, self.registers.b);

        8
    }

    // BIT 3,C
    fn bit_3_c(&mut self) -> usize {
        self.bit(3, self.registers.c);

        8
    }

    // BIT 3,D
    fn bit_3_d(&mut self) -> usize {
        self.bit(3, self.registers.d);

        8
    }

    // BIT 3,E
    fn bit_3_e(&mut self) -> usize {
        self.bit(3, self.registers.e);

        8
    }

    // BIT 3,H
    fn bit_3_h(&mut self) -> usize {
        self.bit(3, self.registers.h);

        8
    }

    // BIT 3,L
    fn bit_3_l(&mut self) -> usize {
        self.bit(3, self.registers.l);

        8
    }

    // BIT 3,(HL)
    fn bit_3_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.bit(3, n);

        12
    }

    // BIT 3,A
    fn bit_3_a(&mut self) -> usize {
        self.bit(3, self.registers.a);

        8
    }

    // BIT 4,B
    fn bit_4_b(&mut self) -> usize {
        self.bit(4, self.registers.b);

        8
    }

    // BIT 4,C
    fn bit_4_c(&mut self) -> usize {
        self.bit(4, self.registers.c);

        8
    }

    // BIT 4,D
    fn bit_4_d(&mut self) -> usize {
        self.bit(4, self.registers.d);

        8
    }

    // BIT 4,E
    fn bit_4_e(&mut self) -> usize {
        self.bit(4, self.registers.e);

        8
    }

    // BIT 4,H
    fn bit_4_h(&mut self) -> usize {
        self.bit(4, self.registers.h);

        8
    }

    // BIT 4,L
    fn bit_4_l(&mut self) -> usize {
        self.bit(4, self.registers.l);

        8
    }

    // BIT 4,(HL)
    fn bit_4_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.bit(4, n);

        12
    }

    // BIT 4,A
    fn bit_4_a(&mut self) -> usize {
        self.bit(4, self.registers.a);

        8
    }

    // BIT 5,B
    fn bit_5_b(&mut self) -> usize {
        self.bit(5, self.registers.b);

        8
    }

    // BIT 5,C
    fn bit_5_c(&mut self) -> usize {
        self.bit(5, self.registers.c);

        8
    }

    // BIT 5,D
    fn bit_5_d(&mut self) -> usize {
        self.bit(5, self.registers.d);

        8
    }

    // BIT 5,E
    fn bit_5_e(&mut self) -> usize {
        self.bit(5, self.registers.e);

        8
    }

    // BIT 5,H
    fn bit_5_h(&mut self) -> usize {
        self.bit(5, self.registers.h);

        8
    }

    // BIT 5,L
    fn bit_5_l(&mut self) -> usize {
        self.bit(5, self.registers.l);

        8
    }

    // BIT 5,(HL)
    fn bit_5_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.bit(5, n);

        12
    }

    // BIT 5,A
    fn bit_5_a(&mut self) -> usize {
        self.bit(5, self.registers.a);

        8
    }

    // BIT 6,B
    fn bit_6_b(&mut self) -> usize {
        self.bit(6, self.registers.b);

        8
    }

    // BIT 6,C
    fn bit_6_c(&mut self) -> usize {
        self.bit(6, self.registers.c);

        8
    }

    // BIT 6,D
    fn bit_6_d(&mut self) -> usize {
        self.bit(6, self.registers.d);

        8
    }

    // BIT 6,E
    fn bit_6_e(&mut self) -> usize {
        self.bit(6, self.registers.e);

        8
    }

    // BIT 6,H
    fn bit_6_h(&mut self) -> usize {
        self.bit(6, self.registers.h);

        8
    }

    // BIT 6,L
    fn bit_6_l(&mut self) -> usize {
        self.bit(6, self.registers.l);

        8
    }

    // BIT 6,(HL)
    fn bit_6_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.bit(6, n);

        12
    }

    // BIT 6,A
    fn bit_6_a(&mut self) -> usize {
        self.bit(6, self.registers.a);

        8
    }

    // BIT 7,B
    fn bit_7_b(&mut self) -> usize {
        self.bit(7, self.registers.b);

        8
    }

    // BIT 7,C
    fn bit_7_c(&mut self) -> usize {
        self.bit(7, self.registers.c);

        8
    }

    // BIT 7,D
    fn bit_7_d(&mut self) -> usize {
        self.bit(7, self.registers.d);

        8
    }

    // BIT 7,E
    fn bit_7_e(&mut self) -> usize {
        self.bit(7, self.registers.e);

        8
    }

    // BIT 7,H
    fn bit_7_h(&mut self) -> usize {
        self.bit(7, self.registers.h);

        8
    }

    // BIT 7,L
    fn bit_7_l(&mut self) -> usize {
        self.bit(7, self.registers.l);

        8
    }

    // BIT 7,(HL)
    fn bit_7_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        self.bit(7, n);

        12
    }

    // BIT 7,A
    fn bit_7_a(&mut self) -> usize {
        self.bit(7, self.registers.a);

        8
    }

    // RES 0,B
    fn res_0_b(&mut self) -> usize {
        self.registers.b = self.res(0, self.registers.b);

        8
    }

    // RES 0,C
    fn res_0_c(&mut self) -> usize {
        self.registers.c = self.res(0, self.registers.c);

        8
    }

    // RES 0,D
    fn res_0_d(&mut self) -> usize {
        self.registers.d = self.res(0, self.registers.d);

        8
    }

    // RES 0,E
    fn res_0_e(&mut self) -> usize {
        self.registers.e = self.res(0, self.registers.e);

        8
    }

    // RES 0,H
    fn res_0_h(&mut self) -> usize {
        self.registers.h = self.res(0, self.registers.h);

        8
    }

    // RES 0,L
    fn res_0_l(&mut self) -> usize {
        self.registers.l = self.res(0, self.registers.l);

        8
    }

    // RES 0,(HL)
    fn res_0_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.res(0, n);
        memory.write_byte(hl, result);

        16
    }

    // RES 0,A
    fn res_0_a(&mut self) -> usize {
        self.registers.a = self.res(0, self.registers.a);

        8
    }

    // RES 1,B
    fn res_1_b(&mut self) -> usize {
        self.registers.b = self.res(1, self.registers.b);

        8
    }

    // RES 1,C
    fn res_1_c(&mut self) -> usize {
        self.registers.c = self.res(1, self.registers.c);

        8
    }

    // RES 1,D
    fn res_1_d(&mut self) -> usize {
        self.registers.d = self.res(1, self.registers.d);

        8
    }

    // RES 1,E
    fn res_1_e(&mut self) -> usize {
        self.registers.e = self.res(1, self.registers.e);

        8
    }

    // RES 1,H
    fn res_1_h(&mut self) -> usize {
        self.registers.h = self.res(1, self.registers.h);

        8
    }

    // RES 1,L
    fn res_1_l(&mut self) -> usize {
        self.registers.l = self.res(1, self.registers.l);

        8
    }

    // RES 1,(HL)
    fn res_1_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.res(1, n);
        memory.write_byte(hl, result);

        16
    }

    // RES 1,A
    fn res_1_a(&mut self) -> usize {
        self.registers.a = self.res(1, self.registers.a);

        8
    }

    // RES 2,B
    fn res_2_b(&mut self) -> usize {
        self.registers.b = self.res(2, self.registers.b);

        8
    }

    // RES 2,C
    fn res_2_c(&mut self) -> usize {
        self.registers.c = self.res(2, self.registers.c);

        8
    }

    // RES 2,D
    fn res_2_d(&mut self) -> usize {
        self.registers.d = self.res(2, self.registers.d);

        8
    }

    // RES 2,E
    fn res_2_e(&mut self) -> usize {
        self.registers.e = self.res(2, self.registers.e);

        8
    }

    // RES 2,H
    fn res_2_h(&mut self) -> usize {
        self.registers.h = self.res(2, self.registers.h);

        8
    }

    // RES 2,L
    fn res_2_l(&mut self) -> usize {
        self.registers.l = self.res(2, self.registers.l);

        8
    }

    // RES 2,(HL)
    fn res_2_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.res(2, n);
        memory.write_byte(hl, result);

        16
    }

    // RES 2,A
    fn res_2_a(&mut self) -> usize {
        self.registers.a = self.res(2, self.registers.a);

        8
    }

    // RES 3,B
    fn res_3_b(&mut self) -> usize {
        self.registers.b = self.res(3, self.registers.b);

        8
    }

    // RES 3,C
    fn res_3_c(&mut self) -> usize {
        self.registers.c = self.res(3, self.registers.c);

        8
    }

    // RES 3,D
    fn res_3_d(&mut self) -> usize {
        self.registers.d = self.res(3, self.registers.d);

        8
    }

    // RES 3,E
    fn res_3_e(&mut self) -> usize {
        self.registers.e = self.res(3, self.registers.e);

        8
    }

    // RES 3,H
    fn res_3_h(&mut self) -> usize {
        self.registers.h = self.res(3, self.registers.h);

        8
    }

    // RES 3,L
    fn res_3_l(&mut self) -> usize {
        self.registers.l = self.res(3, self.registers.l);

        8
    }

    // RES 3,(HL)
    fn res_3_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.res(3, n);
        memory.write_byte(hl, result);

        16
    }

    // RES 3,A
    fn res_3_a(&mut self) -> usize {
        self.registers.a = self.res(3, self.registers.a);

        8
    }

    // RES 4,B
    fn res_4_b(&mut self) -> usize {
        self.registers.b = self.res(4, self.registers.b);

        8
    }

    // RES 4,C
    fn res_4_c(&mut self) -> usize {
        self.registers.c = self.res(4, self.registers.c);

        8
    }

    // RES 4,D
    fn res_4_d(&mut self) -> usize {
        self.registers.d = self.res(4, self.registers.d);

        8
    }

    // RES 4,E
    fn res_4_e(&mut self) -> usize {
        self.registers.e = self.res(4, self.registers.e);

        8
    }

    // RES 4,H
    fn res_4_h(&mut self) -> usize {
        self.registers.h = self.res(4, self.registers.h);

        8
    }

    // RES 4,L
    fn res_4_l(&mut self) -> usize {
        self.registers.l = self.res(4, self.registers.l);

        8
    }

    // RES 4,(HL)
    fn res_4_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.res(4, n);
        memory.write_byte(hl, result);

        16
    }

    // RES 4,A
    fn res_4_a(&mut self) -> usize {
        self.registers.a = self.res(4, self.registers.a);

        8
    }

    // RES 5,B
    fn res_5_b(&mut self) -> usize {
        self.registers.b = self.res(5, self.registers.b);

        8
    }

    // RES 5,C
    fn res_5_c(&mut self) -> usize {
        self.registers.c = self.res(5, self.registers.c);

        8
    }

    // RES 5,D
    fn res_5_d(&mut self) -> usize {
        self.registers.d = self.res(5, self.registers.d);

        8
    }

    // RES 5,E
    fn res_5_e(&mut self) -> usize {
        self.registers.e = self.res(5, self.registers.e);

        8
    }

    // RES 5,H
    fn res_5_h(&mut self) -> usize {
        self.registers.h = self.res(5, self.registers.h);

        8
    }

    // RES 5,L
    fn res_5_l(&mut self) -> usize {
        self.registers.l = self.res(5, self.registers.l);

        8
    }

    // RES 5,(HL)
    fn res_5_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.res(5, n);
        memory.write_byte(hl, result);

        16
    }

    // RES 5,A
    fn res_5_a(&mut self) -> usize {
        self.registers.a = self.res(5, self.registers.a);

        8
    }

    // RES 6,B
    fn res_6_b(&mut self) -> usize {
        self.registers.b = self.res(6, self.registers.b);

        8
    }

    // RES 6,C
    fn res_6_c(&mut self) -> usize {
        self.registers.c = self.res(6, self.registers.c);

        8
    }

    // RES 6,D
    fn res_6_d(&mut self) -> usize {
        self.registers.d = self.res(6, self.registers.d);

        8
    }

    // RES 6,E
    fn res_6_e(&mut self) -> usize {
        self.registers.e = self.res(6, self.registers.e);

        8
    }

    // RES 6,H
    fn res_6_h(&mut self) -> usize {
        self.registers.h = self.res(6, self.registers.h);

        8
    }

    // RES 6,L
    fn res_6_l(&mut self) -> usize {
        self.registers.l = self.res(6, self.registers.l);

        8
    }

    // RES 6,(HL)
    fn res_6_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.res(6, n);
        memory.write_byte(hl, result);

        16
    }

    // RES 6,A
    fn res_6_a(&mut self) -> usize {
        self.registers.a = self.res(6, self.registers.a);

        8
    }

    // RES 7,B
    fn res_7_b(&mut self) -> usize {
        self.registers.b = self.res(7, self.registers.b);

        8
    }

    // RES 7,C
    fn res_7_c(&mut self) -> usize {
        self.registers.c = self.res(7, self.registers.c);

        8
    }

    // RES 7,D
    fn res_7_d(&mut self) -> usize {
        self.registers.d = self.res(7, self.registers.d);

        8
    }

    // RES 7,E
    fn res_7_e(&mut self) -> usize {
        self.registers.e = self.res(7, self.registers.e);

        8
    }

    // RES 7,H
    fn res_7_h(&mut self) -> usize {
        self.registers.h = self.res(7, self.registers.h);

        8
    }

    // RES 7,L
    fn res_7_l(&mut self) -> usize {
        self.registers.l = self.res(7, self.registers.l);

        8
    }

    // RES 7,(HL)
    fn res_7_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.res(7, n);
        memory.write_byte(hl, result);

        16
    }

    // RES 7,A
    fn res_7_a(&mut self) -> usize {
        self.registers.a = self.res(7, self.registers.a);

        8
    }

    // SET 0,B
    fn set_0_b(&mut self) -> usize {
        self.registers.b = self.set(0, self.registers.b);

        8
    }

    // SET 0,C
    fn set_0_c(&mut self) -> usize {
        self.registers.c = self.set(0, self.registers.c);

        8
    }

    // SET 0,D
    fn set_0_d(&mut self) -> usize {
        self.registers.d = self.set(0, self.registers.d);

        8
    }

    // SET 0,E
    fn set_0_e(&mut self) -> usize {
        self.registers.e = self.set(0, self.registers.e);

        8
    }

    // SET 0,H
    fn set_0_h(&mut self) -> usize {
        self.registers.h = self.set(0, self.registers.h);

        8
    }

    // SET 0,L
    fn set_0_l(&mut self) -> usize {
        self.registers.l = self.set(0, self.registers.l);

        8
    }

    // SET 0,(HL)
    fn set_0_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.set(0, n);
        memory.write_byte(hl, result);

        16
    }

    // SET 0,A
    fn set_0_a(&mut self) -> usize {
        self.registers.a = self.set(0, self.registers.a);

        8
    }

    // SET 1,B
    fn set_1_b(&mut self) -> usize {
        self.registers.b = self.set(1, self.registers.b);

        8
    }

    // SET 1,C
    fn set_1_c(&mut self) -> usize {
        self.registers.c = self.set(1, self.registers.c);

        8
    }

    // SET 1,D
    fn set_1_d(&mut self) -> usize {
        self.registers.d = self.set(1, self.registers.d);

        8
    }

    // SET 1,E
    fn set_1_e(&mut self) -> usize {
        self.registers.e = self.set(1, self.registers.e);

        8
    }

    // SET 1,H
    fn set_1_h(&mut self) -> usize {
        self.registers.h = self.set(1, self.registers.h);

        8
    }

    // SET 1,L
    fn set_1_l(&mut self) -> usize {
        self.registers.l = self.set(1, self.registers.l);

        8
    }

    // SET 1,(HL)
    fn set_1_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.set(1, n);
        memory.write_byte(hl, result);

        16
    }

    // SET 1,A
    fn set_1_a(&mut self) -> usize {
        self.registers.a = self.set(1, self.registers.a);

        8
    }

    // SET 2,B
    fn set_2_b(&mut self) -> usize {
        self.registers.b = self.set(2, self.registers.b);

        8
    }

    // SET 2,C
    fn set_2_c(&mut self) -> usize {
        self.registers.c = self.set(2, self.registers.c);

        8
    }

    // SET 2,D
    fn set_2_d(&mut self) -> usize {
        self.registers.d = self.set(2, self.registers.d);

        8
    }

    // SET 2,E
    fn set_2_e(&mut self) -> usize {
        self.registers.e = self.set(2, self.registers.e);

        8
    }

    // SET 2,H
    fn set_2_h(&mut self) -> usize {
        self.registers.h = self.set(2, self.registers.h);

        8
    }

    // SET 2,L
    fn set_2_l(&mut self) -> usize {
        self.registers.l = self.set(2, self.registers.l);

        8
    }

    // SET 2,(HL)
    fn set_2_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.set(2, n);
        memory.write_byte(hl, result);

        16
    }

    // SET 2,A
    fn set_2_a(&mut self) -> usize {
        self.registers.a = self.set(2, self.registers.a);

        8
    }

    // SET 3,B
    fn set_3_b(&mut self) -> usize {
        self.registers.b = self.set(3, self.registers.b);

        8
    }

    // SET 3,C
    fn set_3_c(&mut self) -> usize {
        self.registers.c = self.set(3, self.registers.c);

        8
    }

    // SET 3,D
    fn set_3_d(&mut self) -> usize {
        self.registers.d = self.set(3, self.registers.d);

        8
    }

    // SET 3,E
    fn set_3_e(&mut self) -> usize {
        self.registers.e = self.set(3, self.registers.e);

        8
    }

    // SET 3,H
    fn set_3_h(&mut self) -> usize {
        self.registers.h = self.set(3, self.registers.h);

        8
    }

    // SET 3,L
    fn set_3_l(&mut self) -> usize {
        self.registers.l = self.set(3, self.registers.l);

        8
    }

    // SET 3,(HL)
    fn set_3_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.set(3, n);
        memory.write_byte(hl, result);

        16
    }

    // SET 3,A
    fn set_3_a(&mut self) -> usize {
        self.registers.a = self.set(3, self.registers.a);

        8
    }

    // SET 4,B
    fn set_4_b(&mut self) -> usize {
        self.registers.b = self.set(4, self.registers.b);

        8
    }

    // SET 4,C
    fn set_4_c(&mut self) -> usize {
        self.registers.c = self.set(4, self.registers.c);

        8
    }

    // SET 4,D
    fn set_4_d(&mut self) -> usize {
        self.registers.d = self.set(4, self.registers.d);

        8
    }

    // SET 4,E
    fn set_4_e(&mut self) -> usize {
        self.registers.e = self.set(4, self.registers.e);

        8
    }

    // SET 4,H
    fn set_4_h(&mut self) -> usize {
        self.registers.h = self.set(4, self.registers.h);

        8
    }

    // SET 4,L
    fn set_4_l(&mut self) -> usize {
        self.registers.l = self.set(4, self.registers.l);

        8
    }

    // SET 4,(HL)
    fn set_4_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.set(4, n);
        memory.write_byte(hl, result);

        16
    }

    // SET 4,A
    fn set_4_a(&mut self) -> usize {
        self.registers.a = self.set(4, self.registers.a);

        8
    }

    // SET 5,B
    fn set_5_b(&mut self) -> usize {
        self.registers.b = self.set(5, self.registers.b);

        8
    }

    // SET 5,C
    fn set_5_c(&mut self) -> usize {
        self.registers.c = self.set(5, self.registers.c);

        8
    }

    // SET 5,D
    fn set_5_d(&mut self) -> usize {
        self.registers.d = self.set(5, self.registers.d);

        8
    }

    // SET 5,E
    fn set_5_e(&mut self) -> usize {
        self.registers.e = self.set(5, self.registers.e);

        8
    }

    // SET 5,H
    fn set_5_h(&mut self) -> usize {
        self.registers.h = self.set(5, self.registers.h);

        8
    }

    // SET 5,L
    fn set_5_l(&mut self) -> usize {
        self.registers.l = self.set(5, self.registers.l);

        8
    }

    // SET 5,(HL)
    fn set_5_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.set(5, n);
        memory.write_byte(hl, result);

        16
    }

    // SET 5,A
    fn set_5_a(&mut self) -> usize {
        self.registers.a = self.set(5, self.registers.a);

        8
    }

    // SET 6,B
    fn set_6_b(&mut self) -> usize {
        self.registers.b = self.set(6, self.registers.b);

        8
    }

    // SET 6,C
    fn set_6_c(&mut self) -> usize {
        self.registers.c = self.set(6, self.registers.c);

        8
    }

    // SET 6,D
    fn set_6_d(&mut self) -> usize {
        self.registers.d = self.set(6, self.registers.d);

        8
    }

    // SET 6,E
    fn set_6_e(&mut self) -> usize {
        self.registers.e = self.set(6, self.registers.e);

        8
    }

    // SET 6,H
    fn set_6_h(&mut self) -> usize {
        self.registers.h = self.set(6, self.registers.h);

        8
    }

    // SET 6,L
    fn set_6_l(&mut self) -> usize {
        self.registers.l = self.set(6, self.registers.l);

        8
    }

    // SET 6,(HL)
    fn set_6_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.set(6, n);
        memory.write_byte(hl, result);

        16
    }

    // SET 6,A
    fn set_6_a(&mut self) -> usize {
        self.registers.a = self.set(6, self.registers.a);

        8
    }

    // SET 7,B
    fn set_7_b(&mut self) -> usize {
        self.registers.b = self.set(7, self.registers.b);

        8
    }

    // SET 7,C
    fn set_7_c(&mut self) -> usize {
        self.registers.c = self.set(7, self.registers.c);

        8
    }

    // SET 7,D
    fn set_7_d(&mut self) -> usize {
        self.registers.d = self.set(7, self.registers.d);

        8
    }

    // SET 7,E
    fn set_7_e(&mut self) -> usize {
        self.registers.e = self.set(7, self.registers.e);

        8
    }

    // SET 7,H
    fn set_7_h(&mut self) -> usize {
        self.registers.h = self.set(7, self.registers.h);

        8
    }

    // SET 7,L
    fn set_7_l(&mut self) -> usize {
        self.registers.l = self.set(7, self.registers.l);

        8
    }

    // SET 7,(HL)
    fn set_7_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        let n = memory.read_byte(hl);

        let result = self.set(7, n);
        memory.write_byte(hl, result);

        16
    }

    // SET 7,A
    fn set_7_a(&mut self) -> usize {
        self.registers.a = self.set(7, self.registers.a);

        8
    }

    // CALL Z,nn
    fn call_z_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);

        if self.registers.f.contains(Flag::Zero) {
            self.call(memory, nn);

            24
        } else {
            12
        }
    }

    // CALL nn
    fn call_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);
        self.call(memory, nn);

        24
    }

    // ADC A,n
    fn adc_a_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        self.adc(n);

        8
    }

    // RST 08H
    fn rst_08(&mut self, memory: &mut AddressBus) -> usize {
        self.call(memory, 0x08);

        16
    }

    // 0xD0 - 0xDF

    // RET NC
    fn ret_nc(&mut self, memory: &AddressBus) -> usize {
        if !self.registers.f.contains(Flag::Carry) {
            let pop = self.pop(memory);

            self.registers.pc = pop;

            20
        } else {
            8
        }
    }

    // POP DE
    fn pop_de(&mut self, memory: &mut AddressBus) -> usize {
        let pop = self.pop(memory);
        self.registers.set_de(pop);

        12
    }

    // JP NC,nn
    fn jp_nc_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);

        if !self.registers.f.contains(Flag::Carry) {
            self.registers.pc = nn;

            16
        } else {
            12
        }
    }

    // CALL NC,nn
    fn call_nc_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);

        if !self.registers.f.contains(Flag::Carry) {
            self.call(memory, nn);

            24
        } else {
            12
        }
    }

    // PUSH DE
    fn push_de(&mut self, memory: &mut AddressBus) -> usize {
        let de = self.registers.get_de();
        self.push(memory, de);

        16
    }

    // SUB n
    fn sub_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.sub(n);

        8
    }

    // RST 10H
    fn rst_10(&mut self, memory: &mut AddressBus) -> usize {
        self.call(memory, 0x10);

        16
    }

    // RET C
    fn ret_c(&mut self, memory: &AddressBus) -> usize {
        if self.registers.f.contains(Flag::Carry) {
            let pop = self.pop(memory);

            self.registers.pc = pop;

            20
        } else {
            8
        }
    }

    // RETI
    fn reti(&mut self, memory: &AddressBus) -> usize {
        let pop = self.pop(memory);

        self.registers.pc = pop;

        self.ime = true;

        16
    }

    // JP C,nn
    fn jp_c_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);

        if self.registers.f.contains(Flag::Carry) {
            self.registers.pc = nn;

            16
        } else {
            12
        }
    }

    // CALL C,nn
    fn call_c_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);

        if self.registers.f.contains(Flag::Carry) {
            self.call(memory, nn);

            24
        } else {
            12
        }
    }

    // SBC A,n
    fn sbc_a_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.sbc(n);

        8
    }

    // LD (nn),A
    fn ld_nn_a(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);
        memory.write_byte(nn, self.registers.a);

        16
    }

    // RST 18H
    fn rst_18(&mut self, memory: &mut AddressBus) -> usize {
        self.call(memory, 0x18);

        16
    }

    // 0xE0 - 0xEF

    // LDH ($FF00+n),A
    fn ldh_n_a(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        memory.write_byte(0xFF00 + u16::from(n), self.registers.a);

        12
    }

    // POP HL
    fn pop_hl(&mut self, memory: &mut AddressBus) -> usize {
        let pop = self.pop(memory);
        self.registers.set_hl(pop);

        12
    }

    // LD (C),A
    fn ldh_c_a(&mut self, memory: &mut AddressBus) -> usize {
        memory.write_byte(0xFF00 + u16::from(self.registers.c), self.registers.a);

        8
    }

    // PUSH HL
    fn push_hl(&mut self, memory: &mut AddressBus) -> usize {
        let hl = self.registers.get_hl();
        self.push(memory, hl);

        16
    }

    // AND n
    fn and_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        self.and(n);

        8
    }

    // RST 20H
    fn rst_20(&mut self, memory: &mut AddressBus) -> usize {
        self.call(memory, 0x20);

        16
    }

    // ADD SP,n
    fn add_sp_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        let n = i16::from(n as i8) as u16;

        let half_carry = (self.registers.sp & 0xF) + (n & 0xF) > 0xF;
        let carry = (self.registers.sp & 0xFF) + (n & 0xFF) > 0xFF;

        self.registers.sp = self.registers.sp.wrapping_add(n);

        self.registers.f.clear();
        self.registers.f.set(Flag::HalfCarry, half_carry);
        self.registers.f.set(Flag::Carry, carry);

        16
    }

    // JP HL
    fn jp_hl(&mut self) -> usize {
        let hl = self.registers.get_hl();
        self.registers.pc = hl;

        4
    }

    // XOR n
    fn xor_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        self.xor(n);

        8
    }

    // RST 28H
    fn rst_28(&mut self, memory: &mut AddressBus) -> usize {
        self.call(memory, 0x28);

        16
    }

    // 0xF0 - 0xFF

    // LDH A,($FF00+n)
    fn ldh_a_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        self.registers.a = memory.read_byte(0xFF00 + u16::from(n));

        12
    }

    // POP AF
    fn pop_af(&mut self, memory: &AddressBus) -> usize {
        let pop = self.pop(memory);
        self.registers.set_af(pop);

        12
    }

    // LD A,(C)
    fn ldh_a_c(&mut self, memory: &mut AddressBus) -> usize {
        self.registers.a = memory.read_byte(0xFF00 + u16::from(self.registers.c));

        8
    }

    // DI
    fn di(&mut self) -> usize {
        self.ime = false;

        4
    }

    // PUSH AF
    fn push_af(&mut self, memory: &mut AddressBus) -> usize {
        let af = self.registers.get_af();
        self.push(memory, af);

        16
    }

    // OR n
    fn or_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        self.or(n);

        8
    }

    // RST 30H
    fn rst_30(&mut self, memory: &mut AddressBus) -> usize {
        self.call(memory, 0x30);

        16
    }

    // LDHL SP,n
    fn ldhl_sp_n(&mut self, memory: &mut AddressBus) -> usize {
        let n = self.get_n(memory);
        let n = i16::from(n as i8) as u16;

        let half_carry = (self.registers.sp & 0xF) + (n & 0xF) > 0xF;
        let carry = (self.registers.sp & 0xFF) + (n & 0xFF) > 0xFF;

        let hl = self.registers.sp.wrapping_add(n);
        self.registers.set_hl(hl);

        self.registers.f.clear();
        self.registers.f.set(Flag::HalfCarry, half_carry);
        self.registers.f.set(Flag::Carry, carry);

        12
    }

    // LD SP,HL
    fn ld_sp_hl(&mut self) -> usize {
        self.registers.sp = self.registers.get_hl();

        8
    }

    // LD A,(nn)
    fn ld_a_nn(&mut self, memory: &mut AddressBus) -> usize {
        let nn = self.get_nn(memory);
        self.registers.a = memory.read_byte(nn);

        16
    }

    // EI
    fn ei(&mut self) -> usize {
        self.ime = true;

        4
    }

    // CP n
    fn cp_n(&mut self, memory: &AddressBus) -> usize {
        let n = self.get_n(memory);
        self.cp(n);

        8
    }

    // RST 38H
    fn rst_38(&mut self, memory: &mut AddressBus) -> usize {
        self.call(memory, 0x38);

        16
    }
}
