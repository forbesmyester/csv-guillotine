extern crate csv;

use std::io::{BufRead,BufReader,Read};
use csv::ReaderBuilder;


pub struct ReaderFrom<R> {
    rdr: std::io::BufReader<R>,
    field_seperator: u8,
    buffer: Vec<Vec<u8>>,
    prepared: bool,
    consider_lines: usize,
}


struct BufferAcc {
    current_line: usize,
    max_line: usize,
    count: usize,
}


impl<R: Read> ReaderFrom<R> {

    fn prepare(&mut self) -> Result<usize, std::io::Error> {

        let mut read_count = 999;
        let mut process_buffer = vec![];

        fn count_seperators(field_seperator: u8, line: &String) -> usize {

            let mut rdr = ReaderBuilder::new()
                .delimiter(field_seperator)
                .has_headers(false)
                .from_reader(line.as_bytes());

            match rdr.records().next() {
                Some(rec) => {
                    return rec.unwrap_or(csv::StringRecord::new()).len();
                }
                None => 0
            }

        }

        while (process_buffer.len() < self.consider_lines) && (read_count > 0) {
            let mut read_buffer = String::new();
            read_count = self.rdr.read_line(&mut read_buffer)?;
            if read_count > 0 {
                process_buffer.push(read_buffer.clone());
            }
        }

        let max = (&process_buffer).into_iter().fold(
            BufferAcc { count: 0, current_line: 0, max_line: 0 },
            |acc, line| {
                let c = count_seperators(
                    self.field_seperator,
                    line
                );
                if c <= acc.count {
                    return BufferAcc { current_line: acc.current_line + 1, ..acc };
                }
                return BufferAcc {
                    count: c,
                    current_line: acc.current_line + 1,
                    max_line: acc.current_line
                };
            }
        );

        while process_buffer.len() > max.max_line {
            self.buffer.push(process_buffer.remove(max.max_line).as_bytes().to_vec());
        }

        return Result::Ok(self.buffer.len());

    }


    fn read_from_buffer(&mut self, return_buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let mut count = self.buffer[0].len();
        let mut shift = true;
        let as_bytes = self.buffer.remove(0);
        if return_buf.len() < as_bytes.len() {
            count = return_buf.len();
            shift = false;
        }
        for i in 0..count {
            return_buf[i] = as_bytes[i];
        }
        if !shift {
            self.buffer.insert(0, as_bytes[count..].to_vec());
        }
        return Result::Ok(count);
    }


    pub fn new(reader: R, field_seperator: u8, consider_lines: usize) -> ReaderFrom<R> {
        return  ReaderFrom {
            rdr: BufReader::new(reader),
            field_seperator,
            buffer: vec![],
            prepared: false,
            consider_lines
        };
    }

}


impl<R: Read> std::io::Read for ReaderFrom<R> {

    fn read(&mut self, return_buf: &mut [u8]) -> Result<usize, std::io::Error> {

        if !self.prepared {
            self.prepare()?;
            self.prepared = true;
        }

        if self.buffer.len() > 0 {
            return self.read_from_buffer(return_buf);
        }
        return self.rdr.read(return_buf);

    }

}

#[cfg(test)]
mod tests {


    use std::io::BufReader;
    use std::io::prelude::*;


    pub struct FakeCsvReader {
        src: String,
        pos: usize,
    }


    impl FakeCsvReader {
        pub fn new(strng: String) -> FakeCsvReader {
            return FakeCsvReader {
                src: strng,
                pos: 0,
            }
        }
    }


    impl std::io::Read for FakeCsvReader {

        fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {

            let mut to_read = self.src.len() - self.pos;

            if to_read > buf.len() {
                to_read = buf.len();
            }
            if to_read == 0 {
                return Result::Ok(0);
            }

            for i in 0..to_read {
                buf[i] = self.src.as_bytes()[i + self.pos];
            }

            self.pos = self.pos + to_read;

            return Result::Ok(to_read);

        }

    }


    #[test]
    fn it_works() {

        let fr = FakeCsvReader::new("hi there".to_string());
        let mut f = BufReader::new(fr);
        let mut buffer = String::new();

        match f.read_to_string(&mut buffer) {
            Ok(r) => {
                println!("RESULT: {}: {}", r, buffer);
            },
            Err(e) => println!("ERROR: {}", e)
        }

        assert_eq!(buffer, "hi there".to_string());
    }


    #[test]
    fn it_skips_header() {

        let csv = vec![
            "This is a header".to_string(),
            "Full of nonsense, rubbish and problems".to_string(),
            "but before the real data".to_string(),
            "name,age,gender".to_string(),
            "bob,22,M".to_string(),
            "jane,21,F".to_string(),
            "freddy,19,M".to_string()
        ];
        let fr = FakeCsvReader::new(csv.join("\n"));
        let rf = super::ReaderFrom::new(fr, 44, 20);
        let mut br = BufReader::new(rf);
        let mut buffer = String::new();

        match br.read_to_string(&mut buffer) {
            Ok(r) => {
                println!("RESULT: {}: {}", r, buffer);
            },
            Err(e) => println!("ERROR: {}", e)
        }

        assert_eq!(buffer, csv[3..].join("\n"));
    }


    #[test]
    fn it_only_considers_upto_considers() {
        let csv = vec![
            "This is a header".to_string(),
            "Full of nonsense, rubbish and problems".to_string(),
            "but before the real data".to_string(),
            "name,age,gender".to_string(),
            "bob,22,M".to_string(),
            "jane,21,F".to_string(),
            "freddy,19,M".to_string()
        ];
        let fr = FakeCsvReader::new(csv.join("\n"));
        let rf = super::ReaderFrom::new(fr, 44, 3);
        let mut br = BufReader::new(rf);
        let mut buffer = String::new();

        match br.read_to_string(&mut buffer) {
            Ok(r) => {
                println!("RESULT: {}: {}", r, buffer);
            },
            Err(e) => println!("ERROR: {}", e)
        }

        assert_eq!(buffer, csv[1..].join("\n"));
    }


}
