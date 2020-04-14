use crate::cpu::CPU;

impl super::CPU {
    fn debug_print(&mut self) {
        println!("{:X}  {}    A:{:X} X:{:X} Y:{:X} P:{:X} SP{:X} CYC:{}", self.pc, self.read(self.pc as usize), self.a, self.x, self.y, u8::from(self.p), self.sp, self.cycles);
    }
}
