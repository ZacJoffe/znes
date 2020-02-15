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

#[derive(Copy, Clone)]
struct Status {
    negative: bool,
    overflow: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool
}

impl Status {
    pub fn new() -> Status {
        Status {
            negative: false,
            overflow: false,
            decimal: false,
            interrupt: false,
            zero: false,
            carry: false
        }
    }

    pub fn set_negative(&mut self, num: u8) {
        self.negative = (num & 0x80) == 0x80;
    }

    pub fn set_zero(&mut self, num: u8) {
        self.zero = num == 0;
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

impl From<Status> for u8 {
    fn from(status: Status) -> u8 {
        let negative = if status.negative { 1 } else { 0 };
        let overflow = if status.overflow { 1 } else { 0 };
        let decimal = if status.decimal { 1 } else { 0 };
        let interrupt = if status.interrupt { 1 } else { 0 };
        let zero = if status.zero { 1 } else { 0 };
        let carry = if status.carry { 1 } else { 0 };

        (negative << 7) | (overflow << 6) | (decimal << 3) | (interrupt << 2) | (zero << 1) | carry
    }
}

struct StepInfo {
    address: usize,
    mode: Mode
}

struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    p: Status,

    memory: [u8; 0x2000],

    cycles: u64,

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

    pub fn adc(&mut self, info: StepInfo) {
        let value = self.read(info.address);

        let result = self.a.wrapping_add(value);
        let result = result.wrapping_add(self.p.carry as u8);

        self.p.carry = result <= self.a && (value != 0 || self.p.carry);
        self.p.set_zero(result);
        self.p.set_negative(result);

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        self.p.overflow = (value ^ result) & (self.a ^ result) & 0x80 != 0;

        self.a = result;
    }

    pub fn and(&mut self, info: StepInfo) {
        self.a &= self.read(info.address);
        self.p.set_zero(self.a);
        self.p.set_negative(self.a);
    }

    pub fn asl(&mut self, info: StepInfo) {
        let mut value = match info.mode {
            Mode::ACC => self.a,
            _ => self.read(info.address)
        };

        self.p.carry = (value >> 7) & 0x1 != 0;
        value <<= 1;
        self.p.set_zero(value);
        self.p.set_negative(value);

        match info.mode {
            Mode::ACC => self.a = value,
            _ => self.write(info.address, value)
        };
    }

    pub fn bcc(&mut self, info: StepInfo) {
        if !self.p.carry {
            self.branch(self.read(info.address))
        }
    }

    pub fn bcs(&mut self, info: StepInfo) {
        if self.p.carry {
            self.branch(self.read(info.address))
        }
    }

    pub fn beq(&mut self, info: StepInfo) {
        if self.p.zero {
            self.branch(self.read(info.address))
        }
    }

    pub fn bit(&mut self, info: StepInfo) {
        let value = self.read(info.address);
        self.p.overflow = (value >> 6) & 0x1 != 0;
        self.p.set_zero(value & self.a);
        self.p.set_negative(value);
    }

    pub fn bmi(&mut self, info: StepInfo) {
        if self.p.negative {
            self.branch(self.read(info.address))
        }
    }

    pub fn bne(&mut self, info: StepInfo) {
        if !self.p.zero {
            self.branch(self.read(info.address))
        }
    }

    pub fn bpl(&mut self, info: StepInfo) {
        if !self.p.negative {
            self.branch(self.read(info.address))
        }
    }

    pub fn brk(&mut self, info: StepInfo) {
        // todo - implement interrupts
    }

    pub fn bvc(&mut self, info: StepInfo) {
        if !self.p.overflow {
            self.branch(self.read(info.address))
        }
    }

    pub fn bvs(&mut self, info: StepInfo) {
        if self.p.overflow {
            self.branch(self.read(info.address))
        }
    }

    pub fn clc(&mut self, info: StepInfo) {
        self.p.carry = false;
    }

    pub fn cld(&mut self, info: StepInfo) {
        self.p.decimal = false;
    }

    pub fn cli(&mut self, info: StepInfo) {
        self.p.interrupt = false;
    }

    pub fn clv(&mut self, info: StepInfo) {
        self.p.overflow = false;
    }

    pub fn cmp(&mut self, info: StepInfo) {
        let value = self.read(info.address);
        self.p.carry = self.a >= value;
        self.p.zero = self.a == value;
        self.p.set_negative(self.a.wrapping_sub(value));
    }

    pub fn cpx(&mut self, info: StepInfo) {
        let value = self.read(info.address);
        self.p.carry = self.x >= value;
        self.p.zero = self.x == value;
        self.p.set_negative(self.x.wrapping_sub(value));
    }

    pub fn cpy(&mut self, info: StepInfo) {
        let value = self.read(info.address);
        self.p.carry = self.y >= value;
        self.p.zero = self.y == value;
        self.p.set_negative(self.y.wrapping_sub(value));
    }

    pub fn dec(&mut self, info: StepInfo) {
        let value = self.read(info.address).wrapping_sub(1);
        self.write(info.address, value);

        self.p.set_zero(value);
        self.p.set_negative(value);
    }

    pub fn dex(&mut self, info: StepInfo) {
        self.x = self.x.wrapping_sub(1);
        self.p.set_zero(self.x);
        self.p.set_negative(self.x);
    }

    pub fn dey(&mut self, info: StepInfo) {
        self.y = self.y.wrapping_sub(1);
        self.p.set_zero(self.y);
        self.p.set_negative(self.y);
    }

    pub fn eor(&mut self, info: StepInfo) {
        self.a ^= self.read(info.address);
        self.p.set_zero(self.a);
        self.p.set_negative(self.a);
    }

    pub fn inc(&mut self, info: StepInfo) {
        let value = self.read(info.address).wrapping_add(1);
        self.write(info.address, value);

        self.p.set_zero(value);
        self.p.set_negative(value);
    }

    pub fn inx(&mut self, info: StepInfo) {
        self.x = self.x.wrapping_add(1);
        self.p.set_zero(self.x);
        self.p.set_negative(self.x);
    }

    pub fn iny(&mut self, info: StepInfo) {
        self.y = self.y.wrapping_add(1);
        self.p.set_zero(self.y);
        self.p.set_negative(self.y);
    }

    pub fn jmp(&mut self, info: StepInfo) {
        self.pc = info.address as u16;
    }

    pub fn jsr(&mut self, info: StepInfo) {
        self.push_u16(self.pc - 1);
        self.pc = info.address as u16;
    }

    pub fn lda(&mut self, info: StepInfo) {
        self.a = self.read(info.address);
        self.p.set_zero(self.a);
        self.p.set_negative(self.a);
    }

    pub fn ldx(&mut self, info: StepInfo) {
        self.x = self.read(info.address);
        self.p.set_zero(self.x);
        self.p.set_negative(self.x);
    }

    pub fn ldy(&mut self, info: StepInfo) {
        self.y = self.read(info.address);
        self.p.set_zero(self.y);
        self.p.set_negative(self.y);
    }

    pub fn lsr(&mut self, info: StepInfo) {
        let mut value = match info.mode {
            Mode::ACC => self.a,
            _ => self.read(info.address)
        };

        self.p.carry = (value >> 7) & 0x1 != 0;
        value >>= 1;
        self.p.set_zero(value);
        self.p.set_negative(value);

        match info.mode {
            Mode::ACC => self.a = value,
            _ => self.write(info.address, value)
        };
    }

    pub fn nop(&mut self, info: StepInfo) {

    }

    pub fn ora(&mut self, info: StepInfo) {
        self.a |= self.read(info.address);
        self.p.set_zero(self.a);
        self.p.set_negative(self.a);
    }

    pub fn pha(&mut self, info: StepInfo) {
        self.push(self.a);
    }

    pub fn php(&mut self, info: StepInfo) {
        self.push(u8::from(self.p) | 0x10);
    }

    pub fn pla(&mut self, info: StepInfo) {
        self.a = self.pop();
        self.p.set_zero(self.a);
        self.p.set_negative(self.a);
    }

    pub fn plp(&mut self, info: StepInfo) {
        self.p = Status::from(self.pop());
    }

    pub fn rol(&mut self, info: StepInfo) {
        let mut value = match info.mode {
            Mode::ACC => self.a,
            _ => self.read(info.address)
        };

        // store bit 0 in the carry flag
        self.p.carry = value & 0x1 != 0;
        value = value.rotate_left(1);
        self.p.set_zero(value);
        self.p.set_negative(value);

        match info.mode {
            Mode::ACC => self.a = value,
            _ => self.write(info.address, value)
        };
    }

    pub fn ror(&mut self, info: StepInfo) {
        let mut value = match info.mode {
            Mode::ACC => self.a,
            _ => self.read(info.address)
        };

        // store bit 7 in the carry flag
        self.p.carry = (value >> 7) & 0x1 != 0;
        value = value.rotate_right(1);
        self.p.set_zero(value);
        self.p.set_negative(value);

        match info.mode {
            Mode::ACC => self.a = value,
            _ => self.write(info.address, value)
        };
    }
    pub fn rti(&mut self, info: StepInfo) {
        self.p = Status::from(self.pop());
        self.pc = self.pop_u16();
    }

    pub fn rts(&mut self, info: StepInfo) {
        self.pc = self.pop_u16() + 1;
    }

    pub fn sbc(&mut self, info: StepInfo) {
        let value = self.read(info.address);
        let result = self.a.wrapping_sub(value).wrapping_sub(!self.p.carry as u8);

        self.p.carry = !(result >= self.a && (value != 0 || self.p.carry));
        self.p.set_zero(self.a);
        self.p.set_negative(self.a);

        let acc = self.a & 0x80 == 0;
        let mem = value & 0x80 == 0;
        let res = result & 0x80 == 0;

        self.p.overflow = (acc && !mem && !res) || (!acc && mem && res);

        self.a = result;
    }

    pub fn sec(&mut self, info: StepInfo) {
        self.p.carry = true;
    }

    pub fn sed(&mut self, info: StepInfo) {
        self.p.decimal = true;
    }

    pub fn sei(&mut self, info: StepInfo) {
        self.p.interrupt = true;
    }

    pub fn sta(&mut self, info: StepInfo) {
        self.write(info.address, self.a);
    }

    pub fn stx(&mut self, info: StepInfo) {
        self.write(info.address, self.x);
    }

    pub fn sty(&mut self, info: StepInfo) {
        self.write(info.address, self.y);
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
