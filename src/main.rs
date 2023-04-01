use clap::Parser;
use classfile::ClassFile;
use std::{fs::File, path::PathBuf};

mod classfile;

#[derive(Parser)]
struct Args {
    class_file: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut f = match File::open(args.class_file) {
        Ok(f) => f,
        Err(e) => panic!("{e}"),
    };

    let classfile = ClassFile::new(&mut f);
    println!("{classfile:#?}");
}
