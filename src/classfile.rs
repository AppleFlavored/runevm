use self::{
    attributes::{read_attributes, Attribute},
    constants::{Constant, ConstantPool},
    error::Error,
};
use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt};
use std::{io::Read, marker};

mod attributes;
mod constants;
mod error;

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
}

#[derive(Debug)]
pub struct ClassFile<R> {
    read: marker::PhantomData<R>,
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

impl<R> ClassFile<R>
where
    R: Read,
{
    pub fn new(mut read: R) -> Result<Self, Error> {
        let magic = read.read_u32::<BigEndian>()?;
        if magic != 0xCAFEBABE {
            return Err(Error::InvalidMagic(magic));
        }

        let minor_version = read.read_u16::<BigEndian>()?;
        let major_version = read.read_u16::<BigEndian>()?;
        let constant_pool = read_constant_pool(&mut read)?;
        let access_flags = ClassAccessFlags {
            bits: read.read_u16::<BigEndian>()?,
        };
        let this_class = read.read_u16::<BigEndian>()?;
        let super_class = read.read_u16::<BigEndian>()?;
        let interfaces = read_interfaces(&mut read)?;
        let fields = read_fields(&mut read, &constant_pool)?;
        let methods = read_methods(&mut read, &constant_pool)?;
        let attributes = read_attributes(&mut read, &constant_pool)?;

        Ok(ClassFile {
            read: marker::PhantomData,
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

fn read_constant_pool<R: Read>(r: &mut R) -> Result<Vec<Constant>, Error> {
    let pool_size = r.read_u16::<BigEndian>()?;
    let mut pool: Vec<Constant> = Vec::with_capacity(pool_size as usize);

    for _ in 1..pool_size {
        let tag = r.read_u8()?;
        pool.push(match tag {
            1 => {
                let length = r.read_u16::<BigEndian>()?;
                let mut buf = String::with_capacity(length as usize);
                r.take(length as u64).read_to_string(&mut buf)?;
                Constant::Utf8(buf)
            }
            7 => {
                let class_index = r.read_u16::<BigEndian>()?;
                Constant::Class(class_index)
            }
            8 => {
                let string_index = r.read_u16::<BigEndian>()?;
                Constant::String(string_index)
            }
            9 => {
                let class_index = r.read_u16::<BigEndian>()?;
                let nametype_index = r.read_u16::<BigEndian>()?;
                Constant::FieldRef {
                    class_index,
                    nametype_index,
                }
            }
            10 => {
                let class_index = r.read_u16::<BigEndian>()?;
                let nametype_index = r.read_u16::<BigEndian>()?;
                Constant::MethodRef {
                    class_index,
                    nametype_index,
                }
            }
            12 => {
                let name_index = r.read_u16::<BigEndian>()?;
                let descriptor_index = r.read_u16::<BigEndian>()?;
                Constant::NameAndType {
                    name_index,
                    descriptor_index,
                }
            }
            _ => return Err(Error::UnhandledConstant(tag)),
        });
    }

    Ok(pool)
}

fn read_interfaces<R: Read>(r: &mut R) -> Result<Vec<u16>, Error> {
    let interface_count = r.read_u16::<BigEndian>()?;
    let mut interfaces: Vec<u16> = Vec::with_capacity(interface_count as usize);

    for _ in 0..interface_count {
        interfaces.push(r.read_u16::<BigEndian>()?);
    }

    Ok(interfaces)
}

bitflags! {
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
}

#[derive(Debug)]
pub struct Field {
    pub access_flags: FieldAccessFields,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

fn read_fields<R: Read>(r: &mut R, pool: &ConstantPool) -> Result<Vec<Field>, Error> {
    let field_count = r.read_u16::<BigEndian>()?;
    let mut fields: Vec<Field> = Vec::with_capacity(field_count as usize);

    for _ in 0..field_count {
        let access_flags = FieldAccessFields {
            bits: r.read_u16::<BigEndian>()?,
        };
        let name_index = r.read_u16::<BigEndian>()?;
        let descriptor_index = r.read_u16::<BigEndian>()?;
        let attributes = read_attributes(r, pool)?;

        fields.push(Field {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }

    Ok(fields)
}

bitflags! {
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

#[derive(Debug)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

fn read_methods<R: Read>(r: &mut R, pool: &ConstantPool) -> Result<Vec<Method>, Error> {
    let method_count = r.read_u16::<BigEndian>()?;
    let mut methods: Vec<Method> = Vec::with_capacity(method_count as usize);

    for _ in 0..method_count {
        let access_flags = MethodAccessFlags {
            bits: r.read_u16::<BigEndian>()?,
        };
        let name_index = r.read_u16::<BigEndian>()?;
        let descriptor_index = r.read_u16::<BigEndian>()?;
        let attributes = read_attributes(r, pool)?;

        methods.push(Method {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }

    Ok(methods)
}
