use std::{env, fs, path::Path, process::Command};

use cc::Build;

fn main() {
    const FILES: &[&str] = &["string/memcpy", "string/memmove", "string/memset"];

    let out_dir = env::var("OUT_DIR").unwrap();

    let src_paths: Vec<_> = FILES
        .iter()
        .copied()
        .map(|this_file| format!("src/{}.asm", this_file))
        .collect();
    let out_paths: Vec<_> = FILES
        .iter()
        .copied()
        .map(|this_file| format!("{}/{}.o", out_dir, this_file))
        .collect();

    println!("cargo:rerun-if-changed=src/crt0/_start.asm");

    for this_src in src_paths.iter() {
        println!("cargo:rerun-if-changed={}", this_src);
    }

    let mut cmd = Command::new("nasm");
    cmd.args(&[
        "-f",
        "elf64",
        "-o",
        &format!("{}/crt0.o", env::var("OUT_DIR").unwrap()),
        "src/crt0/_start.asm",
    ]);
    println!("running: {:?}", cmd);
    cmd.status()
        .expect("nasm failed to build 'src/crt0/_start.asm'");

    for (this_src, this_out) in src_paths.iter().zip(out_paths.iter()) {
        let parent_dir = Path::new(this_out).parent().unwrap();
        fs::create_dir_all(parent_dir)
            .unwrap_or_else(|_| panic!("couldn't create directory '{}'", parent_dir.display()));

        let mut cmd = Command::new("nasm");
        cmd.args(&["-f", "elf64", "-o", this_out, this_src]);

        println!("running: {:?}", cmd);

        cmd.status()
            .unwrap_or_else(|_| panic!("nasm failed to build '{}'", this_src));
    }

    let mut build = Build::new();
    build.cargo_metadata(true);

    for this_out in out_paths.iter() {
        build.object(this_out);
    }

    build.compile("kns-asm");

    println!("cargo:rerun-if-changed=rpmalloc/rpmalloc.c");
    println!("cargo:rerun-if-changed=rpmalloc/rpmalloc.h");

    Build::new()
        .cargo_metadata(true)
        .no_default_flags(true)
        .flag("-nostdinc")
        .flag("-nostdlib")
        .flag("-nodefaultlibs")
        .flag("-isysteminclude")
        .file("rpmalloc/rpmalloc.c")
        .compile("kns-rpmalloc");
}
