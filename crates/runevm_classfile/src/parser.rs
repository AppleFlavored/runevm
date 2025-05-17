use crate::{
    instructions::instruction, ClassAccessFlags, FieldAccessFields, Instruction, MethodAccessFlags,
};
use nom::{
    bytes::complete::tag,
    combinator::{fail, map, success},
    multi::{count, length_count, length_data, length_value, many0},
    number::complete::{be_f32, be_f64, be_i32, be_i64, be_u16, be_u32, be_u8},
    sequence::tuple,
    IResult,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

fn version(input: &[u8]) -> IResult<&[u8], Version> {
    map(tuple((be_u16, be_u16)), |(minor, major)| Version {
        major,
        minor,
    })(input)
}

#[derive(Debug, Clone)]
pub enum Constant {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(u16),
    String(u16),
    Field {
        class_index: u16,
        nametype_index: u16,
    },
    Method {
        class_index: u16,
        nametype_index: u16,
    },
    InterfaceMethod {
        class_index: u16,
        nametype_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
}

fn constant(input: &[u8]) -> IResult<&[u8], Constant> {
    let (input, tag) = be_u8(input)?;

    match tag {
        1 => map(length_data(be_u16), |bytes: &[u8]| unsafe {
            Constant::Utf8(String::from_utf8_unchecked(bytes.to_vec()))
        })(input),
        3 => map(be_i32, |value| Constant::Integer(value))(input),
        4 => map(be_f32, |value| Constant::Float(value))(input),
        5 => map(be_i64, |value| Constant::Long(value))(input),
        6 => map(be_f64, |value| Constant::Double(value))(input),
        7 => map(be_u16, |name_index| Constant::Class(name_index))(input),
        8 => map(be_u16, |string_index| Constant::String(string_index))(input),
        9 => map(tuple((be_u16, be_u16)), |(class_index, nametype_index)| {
            Constant::Field {
                class_index,
                nametype_index,
            }
        })(input),
        10 => map(tuple((be_u16, be_u16)), |(class_index, nametype_index)| {
            Constant::Method {
                class_index,
                nametype_index,
            }
        })(input),
        11 => map(tuple((be_u16, be_u16)), |(class_index, nametype_index)| {
            Constant::InterfaceMethod {
                class_index,
                nametype_index,
            }
        })(input),
        12 => map(tuple((be_u16, be_u16)), |(name_index, descriptor_index)| {
            Constant::NameAndType {
                name_index,
                descriptor_index,
            }
        })(input),
        _ => fail(input),
    }
}

#[derive(Debug, Clone)]
pub struct ConstantPool {
    pub(crate) items: Vec<Constant>,
}

impl ConstantPool {
    pub fn get(&self, index: u16) -> &Constant {
        &self.items[index as usize - 1]
    }

    pub fn utf8(&self, index: u16) -> &str {
        match &self.items[index as usize - 1] {
            Constant::Utf8(data) => data.as_str(),
            _ => panic!(),
        }
    }

    pub fn integer(&self, index: u16) -> i32 {
        match self.items[index as usize - 1] {
            Constant::Integer(value) => value,
            _ => panic!(),
        }
    }

    pub fn name_and_type(&self, index: u16) -> (&str, &str) {
        let (name_index, descriptor_index) = match self.items[index as usize - 1] {
            Constant::NameAndType {
                name_index,
                descriptor_index,
            } => (name_index, descriptor_index),
            _ => panic!(),
        };
        (self.utf8(name_index), self.utf8(descriptor_index))
    }

    pub fn class(&self, index: u16) -> &str {
        let name_index = match self.items[index as usize - 1] {
            Constant::Class(name_index) => name_index,
            _ => panic!(),
        };
        self.utf8(name_index)
    }

    pub fn field(&self, index: u16) -> (u16, u16) {
        match self.items[index as usize - 1] {
            Constant::Field {
                class_index,
                nametype_index,
            } => (class_index, nametype_index),
            _ => panic!(),
        }
    }

    pub fn method(&self, index: u16) -> (u16, u16) {
        match self.items[index as usize - 1] {
            Constant::Method {
                class_index,
                nametype_index,
            } => (class_index, nametype_index),
            _ => panic!(),
        }
    }
}

fn constant_pool(input: &[u8]) -> IResult<&[u8], ConstantPool> {
    let (input, contant_pool_count) = be_u16(input)?;
    map(count(constant, contant_pool_count as usize - 1), |items| {
        ConstantPool { items }
    })(input)
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub access_flags: FieldAccessFields,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

fn field(pool: ConstantPool) -> impl Fn(&[u8]) -> IResult<&[u8], FieldInfo> {
    move |input| {
        map(
            tuple((
                map(be_u16, |bits| FieldAccessFields::from_bits_truncate(bits)),
                be_u16,
                be_u16,
                length_count(be_u16, attribute(pool.clone())),
            )),
            |(access_flags, name_index, descriptor_index, attributes)| FieldInfo {
                access_flags,
                name_index,
                descriptor_index,
                attributes,
            },
        )(input)
    }
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

impl MethodInfo {
    pub fn code(&self) -> &Vec<Instruction> {
        self.attributes
            .iter()
            .find_map(|attr| {
                if let Attribute::Code { code, .. } = attr {
                    Some(code)
                } else {
                    None
                }
            })
            .unwrap() // This is fine for now...
    }
}

fn method(pool: ConstantPool) -> impl Fn(&[u8]) -> IResult<&[u8], MethodInfo> {
    move |input| {
        map(
            tuple((
                map(be_u16, |bits| MethodAccessFlags::from_bits_truncate(bits)),
                be_u16,
                be_u16,
                length_count(be_u16, attribute(pool.clone())),
            )),
            |(access_flags, name_index, descriptor_index, attributes)| MethodInfo {
                access_flags,
                name_index,
                descriptor_index,
                attributes,
            },
        )(input)
    }
}

#[derive(Debug, Clone)]
pub enum Attribute {
    ConstantValue(u16),
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<Instruction>,
    },
    Unknown(u16),
}

fn attribute(constant_pool: ConstantPool) -> impl Fn(&[u8]) -> IResult<&[u8], Attribute> {
    move |input| {
        let (input, name_index) = be_u16(input)?;
        let (remaining, attribute_data) = length_data(be_u32)(input)?;

        if let Constant::Utf8(str) = &constant_pool.items[name_index as usize - 1] {
            let (_, attr) = match str.as_str() {
                "ConstantValue" => {
                    map(be_u16, |index| Attribute::ConstantValue(index))(attribute_data)?
                }
                "Code" => map(
                    tuple((be_u16, be_u16, length_value(be_u32, many0(instruction)))),
                    |(max_stack, max_locals, code)| Attribute::Code {
                        max_stack,
                        max_locals,
                        code,
                    },
                )(attribute_data)?,
                _ => success(Attribute::Unknown(name_index))(attribute_data)?,
            };
            Ok((remaining, attr))
        } else {
            Ok((remaining, Attribute::Unknown(name_index)))
        }
    }
}

#[derive(Debug)]
pub struct ClassFile {
    pub version: Version,
    pub constant_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<Attribute>,
}

impl ClassFile {
    pub fn get_method(&self, name: &str, descriptor: &str) -> &MethodInfo {
        self.methods
            .iter()
            .find(|method| {
                let method_name = self.constant_pool.utf8(method.name_index);
                let method_descriptor = self.constant_pool.utf8(method.descriptor_index);
                method_name == name && method_descriptor == descriptor
            })
            .unwrap() // This is fine for now; this should only be used to get a known method.
    }
}

pub fn parse_class(input: &[u8]) -> IResult<&[u8], ClassFile> {
    let (input, _) = tag([0xCA, 0xFE, 0xBA, 0xBE])(input)?;
    let (input, version) = version(input)?;
    let (input, constant_pool) = constant_pool(input)?;

    let mut parser = map(
        tuple((
            map(be_u16, |bits| ClassAccessFlags::from_bits_truncate(bits)),
            be_u16,
            be_u16,
            length_count(be_u16, be_u16),
            length_count(be_u16, field(constant_pool.clone())),
            length_count(be_u16, method(constant_pool.clone())),
            length_count(be_u16, attribute(constant_pool.clone())),
        )),
        |(access_flags, this_class, super_class, interfaces, fields, methods, attributes)| {
            ClassFile {
                version,
                constant_pool: constant_pool.clone(),
                access_flags,
                this_class,
                super_class,
                interfaces,
                fields,
                methods,
                attributes,
            }
        },
    );

    parser(input)
}
