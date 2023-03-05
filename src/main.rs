use clap::Parser;
use classfile::ClassFile;
use std::{fs::File, path::PathBuf};

mod classfile;
mod errors;

#[derive(Parser)]
struct Args {
    class_file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut f = File::open(args.class_file)?;
    let classfile = ClassFile::new(&mut f)?;

    println!("{classfile:?}");

    Ok(())
}
