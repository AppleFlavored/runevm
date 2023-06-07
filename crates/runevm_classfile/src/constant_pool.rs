use crate::{stream::FromSeries, ParsingError, Stream};

#[derive(Clone, Debug)]
pub enum Constant {
    Class(u16),
    FieldRef {
        class_index: u16,
        nametype_index: u16,
    },
    MethodRef {
        class_index: u16,
        nametype_index: u16,
    },
    // InterfaceMethodRef { class_index: u16, nametype_index: u16 },
    String(u16),
    // Integer(u32),
    // Float(f32),
    // Long(u64),
    // Double(f64),
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    Utf8(String),
    // MethodType(u16),
    // ModuleInfo(u16),
    // PackageInfo(u16),
}

#[derive(Debug)]
pub struct ConstantPool {
    items: Vec<Constant>,
}

impl<'a> FromSeries<'a> for ConstantPool {
    fn parse(stream: &'a mut Stream, count: u16) -> Result<Self, ParsingError> {
        let mut constants: Vec<Constant> = Vec::with_capacity(count as usize - 1);

        for _ in 1..count {
            let tag = stream.read::<u8>().ok_or(ParsingError::MissingField)?;

            match read_constant(stream, tag) {
                Some(constant) => constants.push(constant),
                None => return Err(ParsingError::UnhandledConstant(tag)),
            };
        }

        Ok(ConstantPool { items: constants })
    }
}

impl ConstantPool {
    pub fn resolve_name(&self, name_index: u16) -> Option<String> {
        match &self.items[name_index as usize - 1] {
            Constant::Utf8(data) => Some(data.to_string()),
            _ => None,
        }
    }
}

fn read_constant(stream: &mut Stream, tag: u8) -> Option<Constant> {
    match tag {
        1 => {
            let length = stream.read::<u16>()?;
            let buf = match stream.read_bytes(length as _) {
                Some(bytes) => unsafe { String::from_utf8_unchecked(bytes.to_vec()) }, // oops, unsafe code
                None => return None,
            };

            Some(Constant::Utf8(buf))
        }
        7 => {
            let class_index = stream.read::<u16>()?;
            Some(Constant::Class(class_index))
        }
        8 => {
            let string_index = stream.read::<u16>()?;
            Some(Constant::String(string_index))
        }
        9 => {
            let class_index = stream.read::<u16>()?;
            let nametype_index = stream.read::<u16>()?;
            Some(Constant::FieldRef {
                class_index,
                nametype_index,
            })
        }
        10 => {
            let class_index = stream.read::<u16>()?;
            let nametype_index = stream.read::<u16>()?;
            Some(Constant::MethodRef {
                class_index,
                nametype_index,
            })
        }
        12 => {
            let name_index = stream.read::<u16>()?;
            let descriptor_index = stream.read::<u16>()?;
            Some(Constant::NameAndType {
                name_index,
                descriptor_index,
            })
        }
        _ => None,
    }
}
