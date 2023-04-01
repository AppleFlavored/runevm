use super::{
    constants::{Constant, ConstantPool},
    error::Error,
};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Read;

#[derive(Debug)]
pub enum Attribute {
    Unhandled(String),
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

pub fn read_attributes<R: Read>(r: &mut R, pool: &ConstantPool) -> Result<Vec<Attribute>, Error> {
    let attribute_count = r.read_u16::<BigEndian>()?;
    let mut attributes: Vec<Attribute> = Vec::with_capacity(attribute_count as usize);

    for _ in 0..attribute_count {
        let name_index = r.read_u16::<BigEndian>()?;
        let length = r.read_u32::<BigEndian>()?;

        let name = match &pool[name_index as usize - 1] {
            Constant::Utf8(data) => data,
            _ => return Err(Error::InvalidIndex(name_index)),
        };

        attributes.push(match name.as_str() {
            "Code" => {
                let max_stack = r.read_u16::<BigEndian>()?;
                let max_locals = r.read_u16::<BigEndian>()?;

                let code_length = r.read_u32::<BigEndian>()?;
                let mut code = Vec::with_capacity(code_length as usize);
                r.take(code_length as u64).read_to_end(&mut code)?;

                let exception_table_length = r.read_u16::<BigEndian>()?;
                let mut exceptions = Vec::with_capacity(exception_table_length as usize);
                for _ in 0..exception_table_length {
                    let start_pc = r.read_u16::<BigEndian>()?;
                    let end_pc = r.read_u16::<BigEndian>()?;
                    let handler_pc = r.read_u16::<BigEndian>()?;
                    let catch_type = r.read_u16::<BigEndian>()?;

                    exceptions.push(ExceptionTableEntry {
                        start_pc,
                        end_pc,
                        handler_pc,
                        catch_type,
                    });
                }

                let attributes = read_attributes(r, pool)?;

                Attribute::Code { max_stack, max_locals, code, exceptions, attributes, }
            }
            "SourceFile" => {
                let sourcefile_index = r.read_u16::<BigEndian>()?;
                Attribute::SourceFile(sourcefile_index)
            }
            "LineNumberTable" => {
                let table_length = r.read_u16::<BigEndian>()?;
                let mut entries = Vec::with_capacity(table_length as usize);

                for _ in 0..table_length {
                    let start_pc = r.read_u16::<BigEndian>()?;
                    let line_number = r.read_u16::<BigEndian>()?;

                    entries.push(LineNumberTableEntry {
                        start_pc,
                        line_number,
                    });
                }

                Attribute::LineNumberTable(entries)
            }
            _ => {
                // We are not handling this attribute, so we'll just skip it.
                let mut temp = Vec::with_capacity(length as usize);
                r.take(length as u64).read_to_end(&mut temp)?;

                Attribute::Unhandled(name.clone())
            }
        })
    }

    Ok(attributes)
}
