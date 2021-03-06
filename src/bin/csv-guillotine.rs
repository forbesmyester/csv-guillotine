extern crate argparse;
extern crate csv_guillotine;
use std::io::Write;
use std::io::{Read, BufReader};
use std::fs::File;
use argparse::{ArgumentParser, Store, StoreOption};
use csv_guillotine::Blade;

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

    let separator = match args_separator {
        ref x if x == "\\t" => 9,
        x => x.as_bytes()[0],
    };

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

    let mut rdr = Blade::new(stdin, separator, consider);

    let mut exit_code = 0;
    let mut read_size = 1;

    while (exit_code == 0) && (read_size != 0) {
        let mut buffer = [0; 1024];

        read_size = match rdr.read(&mut buffer) {
            Ok(r) => r,
            Err(e) => {
                exit_code = 1;
                eprintln!("ERROR READING: {}", e);
                0
            }
        };

        match stdout.write(&buffer[..read_size]) {
            Ok(_w) => (),
            Err(e) => {
                exit_code = 1;
                eprintln!("ERROR WRITING: {}", e);
            }
        }

    }

    std::process::exit(exit_code);

}
