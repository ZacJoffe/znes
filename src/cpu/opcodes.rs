use crate::cpu::CPU;
use crate::cpu::StepInfo;
use crate::cpu::status::Status;
use crate::cpu::Mode;

impl CPU {
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
            self.branch(info);
        }
    }

    pub fn bcs(&mut self, info: StepInfo) {
        if self.p.carry {
            self.branch(info);
        }
    }

    pub fn beq(&mut self, info: StepInfo) {
        if self.p.zero {
            self.branch(info);
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
            self.branch(info);
        }
    }

    pub fn bne(&mut self, info: StepInfo) {
        if !self.p.zero {
            self.branch(info);
        }
    }

    pub fn bpl(&mut self, info: StepInfo) {
        if !self.p.negative {
            //self.pc = info.address as u16;
            self.branch(info);
        }
    }

    pub fn brk(&mut self, info: StepInfo) {
        self.push_u16(self.pc + 1);
        self.push(u8::from(self.p) | 0x30);
        self.p.interrupt = true;
        self.pc = self.read_u16(0xfffe);
    }

    pub fn bvc(&mut self, info: StepInfo) {
        if !self.p.overflow {
            self.branch(info);
        }
    }

    pub fn bvs(&mut self, info: StepInfo) {
        if self.p.overflow {
            self.branch(info);
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
        self.p.carry = value & 0x1 != 0;
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
        self.push(u8::from(self.p) | 0x30);
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

        let old_carry_bit = self.p.carry as u8;
        // store old bit 7 in the carry flag
        self.p.carry = (value >> 7) & 0x1 != 0;

        // value = value.rotate_left(1);
        value <<= 1;
        value |= old_carry_bit;

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

        let old_carry_bit = self.p.carry as u8;
        // store old bit 0 in the carry flag
        self.p.carry = value & 0x1 != 0;

        // value = value.rotate_right(1);
        value >>= 1;
        value |= old_carry_bit << 7;

        self.p.set_zero(value);
        self.p.set_negative(value);

        match info.mode {
            Mode::ACC => self.a = value,
            _ => self.write(info.address, value)
        };
    }

    pub fn rti(&mut self, info: StepInfo) {
        // self.p = Status::from(self.pop() & 0xef | 0x20);
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
        self.x = self.a;
        self.p.set_zero(self.x);
        self.p.set_negative(self.x);
    }

    pub fn tay(&mut self, info: StepInfo) {
        self.y = self.a;
        self.p.set_zero(self.y);
        self.p.set_negative(self.y);
    }

    pub fn tsx(&mut self, info: StepInfo) {
        self.x = self.sp;
        self.p.set_zero(self.x);
        self.p.set_negative(self.x);
    }

    pub fn txa(&mut self, info: StepInfo) {
        self.a = self.x;
        self.p.set_zero(self.a);
        self.p.set_negative(self.a);
    }

    pub fn txs(&mut self, info: StepInfo) {
        self.sp = self.x;
    }

    pub fn tya(&mut self, info: StepInfo) {
        self.a = self.y;
        self.p.set_zero(self.a);
        self.p.set_negative(self.a);
    }

    // illegal opcode
    pub fn stp(&mut self, info: StepInfo) {
        panic!("Illegal opcode!");
    }

    // interrupts
    pub fn nmi(&mut self) {
        self.push_u16(self.pc);
        self.push(u8::from(self.p) | 0x30);
        self.p.interrupt = true;
        self.pc = self.read_u16(0xfffa);
        self.cycles += 7;
    }

    pub fn irq(&mut self) {
        self.push_u16(self.pc);
        self.push(u8::from(self.p) & !0x30);
        self.p.interrupt = true;
        self.pc = self.read_u16(0xfffe);
        self.cycles += 7;
    }
}
