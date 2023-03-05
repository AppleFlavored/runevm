#[derive(Debug)]
pub enum Attribute {
    LineNumberTable(Vec<LineNumberTableEntry>),
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exceptions: Vec<ExceptionTableEntry>,
        attributes: Vec<Attribute>,
    },
    SourceFile(u16),
}

#[derive(Debug)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}
