extern crate argparse;
use std::io::Write;
use std::io::{Read, BufRead, BufReader};
use std::fs::File;
mod lib;
use argparse::{ArgumentParser, Store, StoreOption};

fn main() {

    let mut args_separator = ",".to_string();
    let mut consider: usize = 20;
    let mut args_input: Option<String> = Option::None;
    let mut args_output: Option<String> = Option::None;

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Remove metadata from top of CSV files");
        ap.refer(&mut args_separator)
            .add_option(
                &["-s", "--separator"],
                Store,
                "The separator between fields (default ',')"
            );
        ap.refer(&mut consider)
            .add_option(
                &["-c", "--consider"],
                Store,
                "The number of lines to consider (default 20)");
        ap.refer(&mut args_input)
            .add_option(
                &["-i", "--input"],
                StoreOption,
                "The filename to read (default '-' is STDIN)");
        ap.refer(&mut args_output)
            .add_option(
                &["-o", "--output"],
                StoreOption,
                "The filename to write (default '-' is STDOUT)");
        ap.parse_args_or_exit();
    }

    let separator = args_separator.as_bytes()[0];

    let stdin: Box<Read> = match args_input {
        Some(p) => {
            let f = File::open(&p).unwrap_or_else(
                |e| {
                    eprintln!("ERROR OPENING READ FILE: {}", p);
                    panic!(e);
                }
            );
            Box::new(BufReader::new(f))
        },
        None => {
            Box::new(std::io::stdin())
        }
    };

    let mut stdout: Box<Write> = match args_output {
        Some(p) => {
            let f = File::create(&p).unwrap_or_else(
                |e| {
                    eprintln!("ERROR OPENING WRITE FILE: {}", p);
                    panic!(e);
                }
            );
            Box::new(f)
        },
        None => {
            Box::new(std::io::stdout())
        }
    };

    let blade = lib::Blade::new(stdin, separator, consider);
    let mut buf_reader = BufReader::new(blade);

    let mut exit_code = 0;
    let mut read_size = 1;

    while (exit_code == 0) && (read_size != 0) {
        let mut buffer = String::new();

        let to_write = match buf_reader.read_line(&mut buffer) {
            Ok(r) => {
                read_size = r;
                buffer.as_bytes()
            },
            Err(e) => {
                exit_code = 1;
                eprintln!("ERROR READING: {}", e);
                &[]
            }
        };

        match stdout.write(to_write) {
            Ok(_w) => (),
            Err(e) => {
                exit_code = 1;
                eprintln!("ERROR WRITING: {}", e);
            }
        }

    }

    std::process::exit(exit_code);

}
