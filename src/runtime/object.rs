use super::method::Method;
use runevm_classfile::ConstantPool;

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub descriptor: String,
}

#[derive(Debug)]
pub struct Object {
    pub constant_pool: ConstantPool,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}