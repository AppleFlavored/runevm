use clap::Parser;
use runevm_classfile::parse_class;
use std::{fs::File, io::Read, path::PathBuf};

mod runtime;

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

    let classfile = match parse_class(buf.as_slice()) {
        Ok((_, classfile)) => classfile,
        Err(e) => panic!("{}", e),
    };

    println!("{classfile:#?}\n");
}
