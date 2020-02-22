mod addressing_modes;
mod opcodes;
mod status;
mod unofficial_opcodes;

use crate::cpu::status::Status;
use crate::cpu::addressing_modes::Mode;

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
    opcode_size_table: [u8; 256]
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xfd,
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
                2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
            ]
        }
    }

    fn read(&self, address: usize) -> u8 {
        self.memory[address % 0x0800]
    }

    fn write(&mut self, address: usize, value: u8) {
        self.memory[address % 0x0800] = value
    }

    fn branch(&mut self, offset: u8) {
        // todo
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
