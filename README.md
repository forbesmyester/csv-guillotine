# CSV Guillotine

[![Build Status](https://travis-ci.com/forbesmyester/csv-guillotine.svg?branch=master)](https://travis-ci.com/forbesmyester/csv-guillotine)

## Purpose

Many banks, stockbrokers and other large institutions will allow you to download your account history in a CSV file. This is good and to be applauded but they often include an extra metadata header at the top of the file explaining what it is. The CSV file may look something like the following:

    Account:,****07493
    £4536.24
    £4536.24

    Transaction type,Description,Paid out,Paid in,Balance
    Bank credit YOUR EMPLOYER,Bank credit YOUR EMPLOYER,,£2016.12,£4536.24
    Direct debit CREDIT PLASTIC,CREDIT PLASTIC,£402.98,,£520.12

For many users this is fine as it can still be loaded into a spreadsheet application.

For my use case, I need to download many of these files, which makes up one large data set and these extra metadata headers are quite an issue because I can no longer use [xsv](https://github.com/BurntSushi/xsv) to parse them directly.

This library is a form of buffer which removes this metadata header. It does this by looking at the field count in a given number of rows and removes the lines before the maximum is reached.

## Compiling

To compile install rust from [rustup](https://rustup.rs/), check out this repository and run:

```bash
    cargo install --path .
```

## Command Line Usage

This can be used like the following:

```bash
    cat with_metadata_headers.csv | csv-guillotine --separator=',' --consider=20 > csv_header_and_data only.csv
```

or 

```bash
    csv-guillotine -i with_metadata_headers.csv -o csv_header_and_data only.csv
```

see `csv-guillotine --help` for full usage instructions

Errors will be printed to STDERR and their existence can be detected via the exit status.

NOTE: This software makes no attempt to actually validate that your CSV.

## Library Usage

This library exposes a `Blade` class which is constructed with a [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html) as well as a character (expressed as a u8) and a line limit. The `Blade` class can be used as a `Read` to get the actual data out.

Example below:

```rust
    extern crate csv_guillotine;
    use std::io::{BufRead, BufReader};
    use csv_guillotine::Blade;

    fn main() {

        let stdin = std::io::stdin();
        let blade = Blade::new(stdin, 44, 20);
        let mut buf_reader = BufReader::new(blade);

        let mut read_size = 1;
        while read_size != 0 {
            let mut buffer = String::new();
            match buf_reader.read_line(&mut buffer) {
                Ok(r) => {
                    print!("{}", buffer);
                    read_size = r;
                },
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                }
            }
        }

    }
```

## Versions

 * 0.1.0 - Initial version
 * 0.1.1 - Rename main class to Blade to keep with the guillotine theme
 * 0.2.0 - Add a command line program
 * 0.3.0 - Use bytes instead of String for everything so it can process non UTF8 files.
 * 0.3.1 - Improve test coverage and fix bugs.
 * 0.3.2 - More normal project layout and nicer code.
 * 0.3.3 - More normal project layout and nicer code.
