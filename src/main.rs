use std::convert::From;

fn main() {
    println!("Hello, world!");
}

enum Mode {
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

struct Status {
    negative: bool,
    overflow: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool
}

impl Status {
    fn new() -> Status {
        Status {
            negative: false,
            overflow: false,
            decimal: false,
            interrupt: false,
            zero: false,
            carry: false
        }
    }
}

impl From<u8> for Status {
    fn from(byte: u8) -> Self {
        let negative = ((byte >> 7) & 0x1) != 0;
        let overflow = ((byte >> 6) & 0x1) != 0;
        let decimal = ((byte >> 3) & 0x1) != 0;
        let interrupt = ((byte >> 2) & 0x1) != 0;
        let zero = ((byte >> 1) & 0x1) != 0;
        let carry = (byte & 0x1) != 0;

        Status {
            negative,
            overflow,
            decimal,
            interrupt,
            zero,
            carry
        }
    }
}

struct StepInfo {

}

struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    p: Status,

    opcode_table: [fn(&mut Self, StepInfo); 256],
    mode_table: [Mode; 256]
}

impl CPU {
    fn new() -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,
            p: Status::from(0x24),

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
            ]
        }
    }

    pub fn adc(&mut self, info: StepInfo) {
        
    }

    pub fn and(&mut self, info: StepInfo) {

    }

    pub fn asl(&mut self, info: StepInfo) {

    }

    pub fn bcc(&mut self, info: StepInfo) {

    }

    pub fn bcs(&mut self, info: StepInfo) {

    }

    pub fn beq(&mut self, info: StepInfo) {

    }

    pub fn bit(&mut self, info: StepInfo) {

    }

    pub fn bmi(&mut self, info: StepInfo) {

    }

    pub fn bne(&mut self, info: StepInfo) {

    }

    pub fn bpl(&mut self, info: StepInfo) {

    }

    pub fn brk(&mut self, info: StepInfo) {

    }

    pub fn bvc(&mut self, info: StepInfo) {

    }

    pub fn bvs(&mut self, info: StepInfo) {

    }

    pub fn clc(&mut self, info: StepInfo) {

    }
    pub fn cld(&mut self, info: StepInfo) {

    }
    pub fn cli(&mut self, info: StepInfo) {

    }
    pub fn clv(&mut self, info: StepInfo) {

    }
    pub fn cmp(&mut self, info: StepInfo) {

    }
    pub fn cpx(&mut self, info: StepInfo) {

    }
    pub fn cpy(&mut self, info: StepInfo) {

    }
    pub fn dec(&mut self, info: StepInfo) {

    }
    pub fn dex(&mut self, info: StepInfo) {

    }
    pub fn dey(&mut self, info: StepInfo) {

    }
    pub fn eor(&mut self, info: StepInfo) {
        
    }
    pub fn inc(&mut self, info: StepInfo) {

    }

    pub fn inx(&mut self, info: StepInfo) {

    }
    pub fn iny(&mut self, info: StepInfo) {

    }
    pub fn jmp(&mut self, info: StepInfo) {

    }
    pub fn jsr(&mut self, info: StepInfo) {

    }

    pub fn lda(&mut self, info: StepInfo) {

    }
    pub fn ldx(&mut self, info: StepInfo) {

    }
    pub fn ldy(&mut self, info: StepInfo) {

    }
    pub fn lsr(&mut self, info: StepInfo) {

    }
    pub fn nop(&mut self, info: StepInfo) {

    }
    pub fn ora(&mut self, info: StepInfo) {

    }
    pub fn pha(&mut self, info: StepInfo) {
        
    }
    pub fn php(&mut self, info: StepInfo) {

    }
    pub fn pla(&mut self, info: StepInfo) {

    }
    pub fn plp(&mut self, info: StepInfo) {

    }
    pub fn rol(&mut self, info: StepInfo) {

    }
    pub fn ror(&mut self, info: StepInfo) {

    }
    pub fn rti(&mut self, info: StepInfo) {

    }
    pub fn rts(&mut self, info: StepInfo) {

    }
    pub fn sbc(&mut self, info: StepInfo) {
        
    }
    pub fn sec(&mut self, info: StepInfo) {

    }
    pub fn sed(&mut self, info: StepInfo) {

    }
    pub fn sei(&mut self, info: StepInfo) {

    }
    pub fn sta(&mut self, info: StepInfo) {

    }
    pub fn stx(&mut self, info: StepInfo) {

    }
    pub fn sty(&mut self, info: StepInfo) {

    }
    pub fn tax(&mut self, info: StepInfo) {

    }
    pub fn tay(&mut self, info: StepInfo) {

    }
    pub fn tsx(&mut self, info: StepInfo) {

    }
    pub fn txa(&mut self, info: StepInfo) {

    }
    pub fn txs(&mut self, info: StepInfo) {

    }
    pub fn tya(&mut self, info: StepInfo) {

    }

    // illegal opcode
    pub fn stp(&mut self, info: StepInfo) {

    }


    // unofficial opcodes
    pub fn ahx(&mut self, info: StepInfo) {

    }
    pub fn alr(&mut self, info: StepInfo) {

    }
    pub fn anc(&mut self, info: StepInfo) {

    }
    pub fn arr(&mut self, info: StepInfo) {

    }
    pub fn axs(&mut self, info: StepInfo) {

    }
    pub fn dcp(&mut self, info: StepInfo) {

    }
    pub fn isc(&mut self, info: StepInfo) {

    }
    pub fn las(&mut self, info: StepInfo) {

    }
    pub fn lax(&mut self, info: StepInfo) {

    }
    pub fn rla(&mut self, info: StepInfo) {

    }
    pub fn rra(&mut self, info: StepInfo) {

    }
    pub fn sax(&mut self, info: StepInfo) {

    }
    pub fn shx(&mut self, info: StepInfo) {

    }
    pub fn shy(&mut self, info: StepInfo) {

    }
    pub fn slo(&mut self, info: StepInfo) {

    }
    pub fn sre(&mut self, info: StepInfo) {

    }
    pub fn tas(&mut self, info: StepInfo) {

    }
    pub fn xaa(&mut self, info: StepInfo) {

    }
}
