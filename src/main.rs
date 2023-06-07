use clap::Parser;
use runevm_classfile::ClassFile;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Parser)]
struct Args {
    classfile: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut file = match File::open(args.classfile) {
        Ok(f) => f,
        Err(err) => panic!("{err}"),
    };

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .expect("could not read class file");

    let classfile = ClassFile::parse(&buf);
    println!("{classfile:?}");
}
