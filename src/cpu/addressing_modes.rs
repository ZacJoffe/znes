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
