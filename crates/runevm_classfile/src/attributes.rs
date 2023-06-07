use crate::{ConstantPool, ParsingError, Stream};

#[derive(Debug)]
pub enum Attribute {
    Unhandled(String),
    // LineNumberTable(Vec<LineNumberTableEntry>),
    ConstantValue(u16),
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exceptions: Vec<ExceptionTableEntry>,
        attributes: Vec<Attribute>,
    },
    SourceFile(u16),
}

pub fn read_attributes<'a>(
    stream: &'a mut Stream,
    constant_pool: &ConstantPool,
) -> Result<Vec<Attribute>, ParsingError> {
    let attributes_count = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
    let mut attributes = Vec::with_capacity(attributes_count as _);

    for _ in 0..attributes_count {
        let name_index = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let length = stream.read::<u32>().ok_or(ParsingError::MissingField)?;

        let attribute_name = match constant_pool.resolve_name(name_index) {
            Some(name) => name,
            None => return Err(ParsingError::InvalidIndex),
        };

        let attribute = match attribute_name.as_str() {
            "ConstantValue" => {
                let constant_index = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
                Some(Attribute::ConstantValue(constant_index))
            }
            "Code" => {
                let max_stack = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
                let max_locals = stream.read::<u16>().ok_or(ParsingError::MissingField)?;

                let code_length = stream.read::<u32>().ok_or(ParsingError::MissingField)?;
                let code_bytes = stream
                    .read_bytes(code_length as _)
                    .ok_or(ParsingError::MissingField)?;

                let exceptions = read_exception_table(stream)?;
                let attributes = read_attributes(stream, constant_pool)?;

                Some(Attribute::Code {
                    max_stack,
                    max_locals,
                    code: code_bytes.to_vec(),
                    exceptions,
                    attributes,
                })
            }
            "SourceFile" => {
                let name_index = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
                Some(Attribute::SourceFile(name_index))
            }
            _ => None,
        };

        match attribute {
            Some(attr) => attributes.push(attr),
            None => {
                attributes.push(Attribute::Unhandled(attribute_name));
                stream.advance(length as _); // Skip remaining bytes
                continue;
            }
        }
    }

    Ok(attributes)
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

fn read_exception_table<'a>(
    stream: &'a mut Stream,
) -> Result<Vec<ExceptionTableEntry>, ParsingError> {
    let exception_table_length = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
    let mut exceptions = Vec::with_capacity(exception_table_length as _);

    for _ in 0..exception_table_length {
        let start_pc = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let end_pc = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let handler_pc = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let catch_type = stream.read::<u16>().ok_or(ParsingError::MissingField)?;

        exceptions.push(ExceptionTableEntry {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        });
    }

    Ok(exceptions)
}
