mod opcodes;
mod status;
mod unofficial_opcodes;

use crate::cpu::status::Status;

#[derive(Copy, Clone)]
pub enum Mode {
    ABS, // Absolute
    ABX, // AbsoluteX
    ABY, // AbsoluteY
    ACC, // Accumulator
    IMM, // Immediate
    IMP, // Implied
    IDX, // IndexedIndirect
    IND, // Indirect
    INX, // IndirectIndexed
    REL, // Relative
    ZPG, // ZeroPage
    ZPX, // ZeroPageX
    ZPY // ZeroPageY
}

pub struct StepInfo {
    address: usize,
    mode: Mode
}

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    p: Status,

    memory: [u8; 0x2000],

    cycles: u64,

    opcode_table: [fn(&mut Self, StepInfo); 256],
    mode_table: [Mode; 256],
    cycle_table: [u8; 256],
    cycle_pages_table: [u8; 256],
    opcode_size_table: [u8; 256]
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,
            p: Status::from(0x24),

            memory: [0; 0x2000],

            cycles: 0,

            opcode_table: [
                CPU::brk, CPU::ora, CPU::stp, CPU::slo, CPU::nop, CPU::ora, CPU::asl, CPU::slo,
                CPU::php, CPU::ora, CPU::asl, CPU::anc, CPU::nop, CPU::ora, CPU::asl, CPU::slo,
                CPU::bpl, CPU::ora, CPU::stp, CPU::slo, CPU::nop, CPU::ora, CPU::asl, CPU::slo,
                CPU::clc, CPU::ora, CPU::nop, CPU::slo, CPU::nop, CPU::ora, CPU::asl, CPU::slo,
                CPU::jsr, CPU::and, CPU::stp, CPU::rla, CPU::bit, CPU::and, CPU::rol, CPU::rla,
                CPU::plp, CPU::and, CPU::rol, CPU::anc, CPU::bit, CPU::and, CPU::rol, CPU::rla,
                CPU::bmi, CPU::and, CPU::stp, CPU::rla, CPU::nop, CPU::and, CPU::rol, CPU::rla,
                CPU::sec, CPU::and, CPU::nop, CPU::rla, CPU::nop, CPU::and, CPU::rol, CPU::rla,
                CPU::rti, CPU::eor, CPU::stp, CPU::sre, CPU::nop, CPU::eor, CPU::lsr, CPU::sre,
                CPU::pha, CPU::eor, CPU::lsr, CPU::alr, CPU::jmp, CPU::eor, CPU::lsr, CPU::sre,
                CPU::bvc, CPU::eor, CPU::stp, CPU::sre, CPU::nop, CPU::eor, CPU::lsr, CPU::sre,
                CPU::cli, CPU::eor, CPU::nop, CPU::sre, CPU::nop, CPU::eor, CPU::lsr, CPU::sre,
                CPU::rts, CPU::adc, CPU::stp, CPU::rra, CPU::nop, CPU::adc, CPU::ror, CPU::rra,
                CPU::pla, CPU::adc, CPU::ror, CPU::arr, CPU::jmp, CPU::adc, CPU::ror, CPU::rra,
                CPU::bvs, CPU::adc, CPU::stp, CPU::rra, CPU::nop, CPU::adc, CPU::ror, CPU::rra,
                CPU::sei, CPU::adc, CPU::nop, CPU::rra, CPU::nop, CPU::adc, CPU::ror, CPU::rra,
                CPU::nop, CPU::sta, CPU::nop, CPU::sax, CPU::sty, CPU::sta, CPU::stx, CPU::sax,
                CPU::dey, CPU::nop, CPU::txa, CPU::xaa, CPU::sty, CPU::sta, CPU::stx, CPU::sax,
                CPU::bcc, CPU::sta, CPU::stp, CPU::ahx, CPU::sty, CPU::sta, CPU::stx, CPU::sax,
                CPU::tya, CPU::sta, CPU::txs, CPU::tas, CPU::shy, CPU::sta, CPU::shx, CPU::ahx,
                CPU::ldy, CPU::lda, CPU::ldx, CPU::lax, CPU::ldy, CPU::lda, CPU::ldx, CPU::lax,
                CPU::tay, CPU::lda, CPU::tax, CPU::lax, CPU::ldy, CPU::lda, CPU::ldx, CPU::lax,
                CPU::bcs, CPU::lda, CPU::stp, CPU::lax, CPU::ldy, CPU::lda, CPU::ldx, CPU::lax,
                CPU::clv, CPU::lda, CPU::tsx, CPU::las, CPU::ldy, CPU::lda, CPU::ldx, CPU::lax,
                CPU::cpy, CPU::cmp, CPU::nop, CPU::dcp, CPU::cpy, CPU::cmp, CPU::dec, CPU::dcp,
                CPU::iny, CPU::cmp, CPU::dex, CPU::axs, CPU::cpy, CPU::cmp, CPU::dec, CPU::dcp,
                CPU::bne, CPU::cmp, CPU::stp, CPU::dcp, CPU::nop, CPU::cmp, CPU::dec, CPU::dcp,
                CPU::cld, CPU::cmp, CPU::nop, CPU::dcp, CPU::nop, CPU::cmp, CPU::dec, CPU::dcp,
                CPU::cpx, CPU::sbc, CPU::nop, CPU::isc, CPU::cpx, CPU::sbc, CPU::inc, CPU::isc,
                CPU::inx, CPU::sbc, CPU::nop, CPU::sbc, CPU::cpx, CPU::sbc, CPU::inc, CPU::isc,
                CPU::beq, CPU::sbc, CPU::stp, CPU::isc, CPU::nop, CPU::sbc, CPU::inc, CPU::isc,
                CPU::sed, CPU::sbc, CPU::nop, CPU::isc, CPU::nop, CPU::sbc, CPU::inc, CPU::isc
            ],

            mode_table: [
                Mode::IMP, Mode::IDX, Mode::IMP, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG,
                Mode::IMP, Mode::IMM, Mode::ACC, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,
                Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX,
                Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,
                Mode::ABS, Mode::IDX, Mode::IMP, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG,
                Mode::IMP, Mode::IMM, Mode::ACC, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,
                Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX,
                Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,
                Mode::IMP, Mode::IDX, Mode::IMP, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG,
                Mode::IMP, Mode::IMM, Mode::ACC, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,
                Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX,
                Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,
                Mode::IMP, Mode::IDX, Mode::IMP, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG,
                Mode::IMP, Mode::IMM, Mode::ACC, Mode::IMM, Mode::IND, Mode::ABS, Mode::ABS, Mode::ABS,
                Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX,
                Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,
                Mode::IMM, Mode::IDX, Mode::IMM, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG,
                Mode::IMP, Mode::IMM, Mode::IMP, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,
                Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPY, Mode::ZPY,
                Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABY, Mode::ABY,
                Mode::IMM, Mode::IDX, Mode::IMM, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG,
                Mode::IMP, Mode::IMM, Mode::IMP, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,
                Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPY, Mode::ZPY,
                Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABY, Mode::ABY,
                Mode::IMM, Mode::IDX, Mode::IMM, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG,
                Mode::IMP, Mode::IMM, Mode::IMP, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,
                Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX,
                Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX,
                Mode::IMM, Mode::IDX, Mode::IMM, Mode::IDX, Mode::ZPG, Mode::ZPG, Mode::ZPG, Mode::ZPG,
                Mode::IMP, Mode::IMM, Mode::IMP, Mode::IMM, Mode::ABS, Mode::ABS, Mode::ABS, Mode::ABS,
                Mode::REL, Mode::INX, Mode::IMP, Mode::INX, Mode::ZPX, Mode::ZPX, Mode::ZPX, Mode::ZPX,
                Mode::IMP, Mode::ABY, Mode::IMP, Mode::ABY, Mode::ABX, Mode::ABX, Mode::ABX, Mode::ABX
            ],

            cycle_table: [
                7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
                2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
                2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
                2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
                2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7
            ],

            cycle_pages_table: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0
            ],

            opcode_size_table: [
                2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
                3, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
                1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
                1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 0, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 0, 3, 0, 0,
                2, 2, 2, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0
            ]
        };

        cpu.reset();
        cpu
    }

    fn reset(&mut self) {
        self.pc = self.read_u16(0xfffc);
        self.sp = 0xfd;
        self.p = Status::from(0x24);
    }

    fn step(&mut self) -> u64 {
        let cycles = self.cycles;
        let opcode = self.read(self.pc as usize);
        let mode = self.mode_table[opcode as usize];

        let address: (u16, bool) = match mode {
            Mode::ABS => (self.read_u16(self.pc as usize + 1), false),
            Mode::ABX => {
                let address = self.read_u16(self.pc as usize + 1) + self.x as u16;
                (address, page_crossed(address.wrapping_sub(self.x as u16) as usize, address as usize))
            },
            Mode::ABY => {
                let address = self.read_u16(self.pc as usize + 1) + self.y as u16;
                (address, page_crossed(address.wrapping_sub(self.y as u16) as usize, address as usize))
            },
            Mode::ACC => (0, false),
            Mode::IMM => (self.pc + 1, false),
            Mode::IMP => (0, false),
            Mode::IDX => {
                let address = self.read(self.pc as usize + 1);

                let zp_low = address.wrapping_add(self.x);
                let zp_high = zp_low.wrapping_add(1);
                let zp_low_value = self.read(zp_low as usize) as u16;
                let zp_high_value = self.read(zp_high as usize) as u16;

                ((zp_high_value << 8) | zp_low_value, false)
            },
            Mode::IND => {
                let address = self.read_u16(self.pc as usize + 1);

                let low = self.read(address as usize) as u16;
                let high = if address & 0xff == 0xff {
                    self.read(address as usize - 0xff) as u16
                } else {
                    self.read(address as usize + 1) as u16
                };

                ((high << 8) | low, false)
            },
            Mode::INX => {
                let address = self.read(self.pc as usize + 1);

                let zp_low = address;
                let zp_high = zp_low.wrapping_add(1);
                let zp_low_value = self.read(zp_low as usize) as u16;
                let zp_high_value = self.read(zp_high as usize) as u16;

                let old_address = (zp_high_value << 8) | zp_low_value;
                let new_address = old_address.wrapping_add(self.y as u16);

                (new_address, page_crossed(old_address as usize, new_address as usize))
            },
            Mode::REL => {
                let offset = self.read(self.pc as usize + 1) as u16;

                let address = if offset < 0x80 {
                    self.pc + 2 + offset
                } else {
                    self.pc + 2 + offset - 0x100
                };

                (address, false)
            },
            Mode::ZPG => (self.read(self.pc as usize + 1) as u16, false),
            Mode::ZPX => (self.read(self.pc as usize + 1).wrapping_add(self.x) as u16, false),
            Mode::ZPY => (self.read(self.pc as usize + 1).wrapping_add(self.y) as u16, false)
        };

        self.pc += self.opcode_size_table[opcode as usize] as u16;
        self.cycles += self.cycle_table[opcode as usize] as u64;

        if address.1 {
            self.cycles += self.cycle_pages_table[opcode as usize] as u64;
        }

        let info = StepInfo {
            address: address.0 as usize,
            mode: mode
        };

        self.opcode_table[opcode as usize](self, info);

        self.cycles - cycles
    }

    fn read(&self, address: usize) -> u8 {
        self.memory[address % 0x0800]
    }

    fn read_u16(&self, address: usize) -> u16 {
        (self.read(address.wrapping_add(1)) as u16) << 8 | (self.read(address) as u16)
    }

    fn write(&mut self, address: usize, value: u8) {
        self.memory[address % 0x0800] = value
    }

    fn branch(&mut self, info: StepInfo) {
        self.cycles += 1;

        let offset = self.read(info.address) as i8;
        let old_pc = self.pc;

        if offset >= 0 {
            self.pc += offset as u16;
        } else {
            self.pc -= (-offset) as u16;
        }

        /*
        if old_pc / 0xff != self.pc / 0xff {
            self.cycles += 2;
        }
        */
        if page_crossed(old_pc as usize, self.pc as usize) {
            self.cycles += 2;
        }
    }

    fn push(&mut self, value: u8) {
        self.write(0x100 + self.sp as usize, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_u16(&mut self, value: u16) {
        self.push((value >> 8) as u8);
        self.push((value & 0xff) as u8);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(0x100 + self.sp as usize)
    }

    fn pop_u16(&mut self) -> u16 {
        let low = self.pop() as u16;
        let high = self.pop() as u16;
        (high << 8) | low
    }
}

fn page_crossed(address1: usize, address2: usize) -> bool {
    return address1 / 0xff != address2 / 0xff
}
