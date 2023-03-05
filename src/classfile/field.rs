use super::attribute::Attribute;
use bitflags::bitflags;

bitflags! {
    pub struct AccessFlags: u16 {
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

impl From<u16> for AccessFlags {
    fn from(value: u16) -> Self {
        Self { bits: value }
    }
}

#[derive(Debug)]
pub struct Field {
    pub access_flags: AccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}
