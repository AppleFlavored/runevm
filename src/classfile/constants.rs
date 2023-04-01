pub type ConstantPool = Vec<Constant>;

#[derive(Debug)]
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
    // PackageInfo(u16)
}
