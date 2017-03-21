#![cfg_attr(feature = "malloc", feature(alloc_system))]

#[cfg(feature = "malloc")]
extern crate alloc_system;

extern crate clap;
extern crate itertools;

use clap::*;
use itertools::*;
use std::process::{
    Stdio,
    Command,
};
use std::str::FromStr;
use std::io::{
    stdin,
    BufRead,
    BufReader,
};
use std::fs::File;

fn main() {
    let matches = App::new("chunk")
        .version("0.1")
        .about(
            "Seperate file into chunks and run a program on each set"
        )
        .author("One F Jef")
        .arg(
            Arg::with_name("FILE")
                .help(
                    "The file to read, if not supplied (or `-`) will read from \
                     stdin"
                )
                .index(1)
        )
        .arg(
            Arg::with_name("executable")
                .short("e")
                .long("exec")
                .help("The program to run")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("argument")
                .short("a")
                .long("arg")
                .help(
                    "An argument to pass to the executable, before the \
                     chunked input (TODO: use a find-like `{}` API)"
                )
                .takes_value(true)
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .help("The number of elements per chunk")
                .takes_value(true)
                .required(true)
        )
        .get_matches();

    let file: Box<BufRead> = matches.value_of("FILE")
        .and_then(|filename|
            if filename == "-" {
                None
            } else {
                Some(
                    File::open(filename)
                        .map(|f| Box::new(BufReader::new(f)) as _)
                )
            }
        )
        .unwrap_or_else(
            || Ok(Box::new(BufReader::new(stdin())) as _)
        )
        .expect("Could not open file");

    let exec = matches.value_of("executable").unwrap();

    let n =
        FromStr::from_str(
            matches.value_of("number").unwrap()
        ).expect(
            "Could not parse"
        );

    let arguments = matches.values_of("argument")
        .map(|v| v.into_iter().collect::<Vec<_>>())
        .unwrap_or(vec![]);

    for mut line in &file.lines().chunks(n) {
        let mut cmd_base = Command::new(exec);
        cmd_base.args(&arguments);
        let mut cmd = line.fold_results(
            cmd_base,
            |mut c, a| {
                c.arg(a);
                c
            }
        ).expect("Failed to read file");

        cmd
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .spawn()
            .expect("Failed to execute");
    }
}
