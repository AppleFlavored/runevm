use self::{
    attribute::{Attribute, ExceptionTableEntry, LineNumberTableEntry},
    constant::Constant,
    field::{AccessFlags as FieldFlags, Field},
    method::{AccessFlags as MethodFlags, Method},
};
use crate::errors::ClassFileError;
use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt};
use std::{fs::File, io::Read};

mod attribute;
mod constant;
mod field;
mod method;

type Result<T> = std::result::Result<T, ClassFileError>;

bitflags! {
    pub struct AccessFlags: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
        const MODULE = 0x8000;
    }
}

#[derive(Debug)]
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<Constant>,
    pub access_flags: AccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}

impl ClassFile {
    pub fn new(file: &mut File) -> Result<ClassFile> {
        let magic = file.read_u32::<BigEndian>()?;
        if magic != 0xCAFEBABE {
            return Err(ClassFileError::InvalidMagic(magic));
        }

        let minor_version = file.read_u16::<BigEndian>()?;
        let major_version = file.read_u16::<BigEndian>()?;
        let constant_pool = read_constant_pool(file)?;
        let access_flags = AccessFlags {
            bits: file.read_u16::<BigEndian>()?,
        };
        let this_class = file.read_u16::<BigEndian>()?;
        let super_class = file.read_u16::<BigEndian>()?;
        let interfaces = read_interfaces(file)?;
        let fields = read_fields(file, &constant_pool)?;
        let methods = read_methods(file, &constant_pool)?;
        let attributes = read_attributes(file, &constant_pool)?;

        Ok(ClassFile {
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }
}

fn read_constant_pool(file: &mut File) -> Result<Vec<Constant>> {
    let count = file.read_u16::<BigEndian>()?;
    let mut constants = Vec::<Constant>::with_capacity(count as usize);

    for _ in 1..count {
        let tag = file.read_u8()?;
        constants.push(match tag {
            1 => {
                let length = file.read_u16::<BigEndian>()?;
                let mut buf = String::with_capacity(length as usize);
                file.take(length as u64).read_to_string(&mut buf)?;
                Constant::Utf8(buf)
            }
            7 => {
                let name_index = file.read_u16::<BigEndian>()?;
                Constant::Class { name_index }
            }
            8 => {
                let string_index = file.read_u16::<BigEndian>()?;
                Constant::String { string_index }
            }
            9 => {
                let class_index = file.read_u16::<BigEndian>()?;
                let name_and_type_index = file.read_u16::<BigEndian>()?;
                Constant::FieldRef {
                    class_index,
                    name_and_type_index,
                }
            }
            10 => {
                let class_index = file.read_u16::<BigEndian>()?;
                let name_and_type_index = file.read_u16::<BigEndian>()?;
                Constant::MethodRef {
                    class_index,
                    name_and_type_index,
                }
            }
            11 => {
                let class_index = file.read_u16::<BigEndian>()?;
                let name_and_type_index = file.read_u16::<BigEndian>()?;
                Constant::InterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                }
            }
            12 => {
                let name_index = file.read_u16::<BigEndian>()?;
                let descriptor_index = file.read_u16::<BigEndian>()?;
                Constant::NameAndType {
                    name_index,
                    descriptor_index,
                }
            }
            _ => return Err(ClassFileError::InvalidTag(tag)),
        });
    }

    Ok(constants)
}

fn read_interfaces(file: &mut File) -> Result<Vec<u16>> {
    let count = file.read_u16::<BigEndian>()?;
    let mut interfaces = Vec::with_capacity(count as usize);

    for _ in 0..count {
        interfaces.push(file.read_u16::<BigEndian>()?);
    }

    Ok(interfaces)
}

fn read_fields(file: &mut File, constants: &Vec<Constant>) -> Result<Vec<Field>> {
    let count = file.read_u16::<BigEndian>()?;
    let mut fields = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let access_flags: FieldFlags = file.read_u16::<BigEndian>()?.into();
        let name_index = file.read_u16::<BigEndian>()?;
        let descriptor_index = file.read_u16::<BigEndian>()?;
        let attributes = read_attributes(file, constants)?;

        fields.push(Field {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }

    Ok(fields)
}

fn read_methods(file: &mut File, constants: &Vec<Constant>) -> Result<Vec<Method>> {
    let count = file.read_u16::<BigEndian>()?;
    let mut methods = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let access_flags: MethodFlags = file.read_u16::<BigEndian>()?.into();
        let name_index = file.read_u16::<BigEndian>()?;
        let descriptor_index = file.read_u16::<BigEndian>()?;
        let attributes = read_attributes(file, constants)?;

        methods.push(Method {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }

    Ok(methods)
}

fn read_attributes(file: &mut File, constants: &Vec<Constant>) -> Result<Vec<Attribute>> {
    let count = file.read_u16::<BigEndian>()?;
    let mut attributes = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let name_index = file.read_u16::<BigEndian>()? - 1;
        let _ = file.read_u32::<BigEndian>()?;

        let attrib_name = if let Constant::Utf8(data) = &constants[name_index as usize] {
            data
        } else {
            return Err(ClassFileError::InvalidConstant(name_index));
        };

        attributes.push(match attrib_name.as_str() {
            "LineNumberTable" => {
                let table_length = file.read_u16::<BigEndian>()?;
                let mut entries = Vec::with_capacity(table_length as usize);

                for _ in 0..table_length {
                    let start_pc = file.read_u16::<BigEndian>()?;
                    let line_number = file.read_u16::<BigEndian>()?;

                    entries.push(LineNumberTableEntry {
                        start_pc,
                        line_number,
                    });
                }

                Attribute::LineNumberTable(entries)
            }
            "Code" => {
                let max_stack = file.read_u16::<BigEndian>()?;
                let max_locals = file.read_u16::<BigEndian>()?;

                let code_length = file.read_u32::<BigEndian>()?;
                let mut code = Vec::with_capacity(code_length as usize);
                file.take(code_length as u64).read_to_end(&mut code)?;

                let exception_table_length = file.read_u16::<BigEndian>()?;
                let mut exceptions = Vec::with_capacity(exception_table_length as usize);
                for _ in 0..exception_table_length {
                    let start_pc = file.read_u16::<BigEndian>()?;
                    let end_pc = file.read_u16::<BigEndian>()?;
                    let handler_pc = file.read_u16::<BigEndian>()?;
                    let catch_type = file.read_u16::<BigEndian>()?;

                    exceptions.push(ExceptionTableEntry {
                        start_pc,
                        end_pc,
                        handler_pc,
                        catch_type,
                    });
                }

                let attributes = read_attributes(file, constants)?;

                Attribute::Code {
                    max_stack,
                    max_locals,
                    code,
                    exceptions,
                    attributes,
                }
            }
            "SourceFile" => {
                let source_file_index = file.read_u16::<BigEndian>()?;
                Attribute::SourceFile(source_file_index)
            }
            _ => return Err(ClassFileError::InvalidAttribute(attrib_name.to_string())),
        });
    }

    Ok(attributes)
}
