extern crate diff;
extern crate pancakes;

use pancakes::{FrameRegisters, Options, Registers};
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;

#[test]
fn cargo_readme_up_to_date() {
    if env::var("CI").is_ok() {
        return;
    }

    let expected = Command::new("cargo")
        .arg("readme")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run `cargo readme` OK")
        .stdout;
    let expected = String::from_utf8_lossy(&expected);

    let actual = {
        let mut file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
            .expect("should open README.md file");
        let mut s = String::new();
        file.read_to_string(&mut s)
            .expect("should read contents of file to string");
        s
    };

    if actual != expected {
        println!();
        println!("+++ expected README.md");
        println!("--- actual README.md");
        for d in diff::lines(&expected, &actual) {
            match d {
                diff::Result::Left(l) => println!("+{}", l),
                diff::Result::Right(r) => println!("-{}", r),
                diff::Result::Both(b, _) => println!(" {}", b),
            }
        }
        panic!("Run `cargo readme > README.md` to update README.md")
    }
}

#[test]
fn smoke_test_unwind() {
    #[inline(never)]
    fn one(walker: &mut pancakes::Walker) {
        two(walker);
    }

    #[inline(never)]
    fn two(walker: &mut pancakes::Walker) {
        three(walker);
    }

    #[inline(never)]
    fn three(walker: &mut pancakes::Walker) {
        let walk_result = FrameRegisters::with_current(|regs| {
            println!("start regs = {:#?}", regs);

            walker.walk(regs, |frame| {
                println!("FITZGEN: frame = {:#?}", frame);
            })
        });

        panic!("temp: {:#?}", walk_result);
    }

    let mut opts = Options::new();
    opts.find_eh_frame_entries()
        .expect("should parse eh_frame entries OK");

    let mut walker = opts.build();

    one(&mut walker);
}
