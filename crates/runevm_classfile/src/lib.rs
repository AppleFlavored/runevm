pub mod attributes;
pub mod constant_pool;
mod stream;

pub use crate::constant_pool::{Constant, ConstantPool};
pub use crate::stream::{FromData, Stream};
use attributes::{read_attributes, Attribute};
use bitflags::bitflags;

#[derive(Debug)]
pub enum ParsingError {
    InvalidMagic,
    InvalidIndex,
    MissingField,
    UnhandledConstant(u8),
}

impl std::error::Error for ParsingError {}

impl core::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::InvalidMagic => write!(f, "invalid magic"),
            ParsingError::InvalidIndex => write!(f, "invalid index into constant pool"),
            ParsingError::MissingField => write!(f, "missing field"),
            ParsingError::UnhandledConstant(tag) => write!(f, "unhandled constant with tag {tag}"),
        }
    }
}

#[derive(Debug)]
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}

impl ClassFile {
    pub fn parse<'a>(data: &'a [u8]) -> Result<Self, ParsingError> {
        let mut stream = Stream::new(data);

        match stream.read::<u32>() {
            Some(0xCAFEBABE) => (),
            _ => return Err(ParsingError::InvalidMagic),
        }

        let minor_version = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let major_version = stream.read::<u16>().ok_or(ParsingError::MissingField)?;

        let constant_pool_count = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let constant_pool = stream.read_array::<ConstantPool>(constant_pool_count)?;

        let access_flags = ClassAccessFlags {
            bits: stream.read::<u16>().ok_or(ParsingError::MissingField)?,
        };
        let this_class = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let super_class = stream.read::<u16>().ok_or(ParsingError::MissingField)?;

        let interfaces_count = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let mut interfaces: Vec<u16> = Vec::with_capacity(interfaces_count as _);
        for _ in 0..interfaces_count {
            interfaces.push(stream.read::<u16>().ok_or(ParsingError::MissingField)?);
        }

        let fields_count = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let mut fields = Vec::<Field>::with_capacity(fields_count as _);
        for _ in 0..fields_count {
            fields.push(Field::parse(&mut stream, &constant_pool)?);
        }

        let methods_count = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let mut methods = Vec::<Method>::with_capacity(methods_count as _);
        for _ in 0..methods_count {
            methods.push(Method::parse(&mut stream, &constant_pool)?);
        }

        let attributes = read_attributes(&mut stream, &constant_pool)?;

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

#[derive(Debug)]
pub struct Field {
    pub access_flags: FieldAccessFields,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

impl Field {
    fn parse<'a>(
        stream: &'a mut Stream,
        constant_pool: &ConstantPool,
    ) -> Result<Self, ParsingError> {
        let access_flags = FieldAccessFields {
            bits: stream.read::<u16>().ok_or(ParsingError::MissingField)?,
        };
        let name_index = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let descriptor_index = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let attributes = read_attributes(stream, constant_pool)?;

        Ok(Field {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }
}

#[derive(Debug)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

impl Method {
    fn parse<'a>(
        stream: &'a mut Stream,
        constant_pool: &ConstantPool,
    ) -> Result<Self, ParsingError> {
        let access_flags = MethodAccessFlags {
            bits: stream.read::<u16>().ok_or(ParsingError::MissingField)?,
        };
        let name_index = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let descriptor_index = stream.read::<u16>().ok_or(ParsingError::MissingField)?;
        let attributes = read_attributes(stream, constant_pool)?;

        Ok(Method {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }

    /// Returns the max stack and max locals from the code attribute.
    pub fn maxs(&self) -> Option<(u16, u16)> {
        for attr in &self.attributes {
            if let Attribute::Code {
                max_stack,
                max_locals,
                ..
            } = attr
            {
                return Some((*max_stack, *max_locals));
            }
        }
        None
    }

    /// Returns the bytes of the code implementing the method.
    pub fn code(&self) -> Option<Vec<u8>> {
        for attr in &self.attributes {
            if let Attribute::Code { code, .. } = attr {
                return Some(code.to_vec());
            }
        }
        None
    }
}

bitflags! {
    pub struct ClassAccessFlags: u16 {
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

    pub struct FieldAccessFields: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM = 0x4000;
    }

    pub struct MethodAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABSTRACT = 0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
    }
}
