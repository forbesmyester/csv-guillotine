use std::io::{BufReader,Read};

pub struct ReaderFrom<R> {
    rdr: std::io::BufReader<R>,
    buf: Vec<String>,
    field_seperator: char,
}

fn row_count(input: &Vec<String>) -> usize {
    let mut last: u8 = 0;
    let mut count = 0;
    for b in input {
        for c in b.as_bytes() {
            if (last == 13) && (*c == 10) {
                continue;
            }
            if (*c == 10) || (*c == 13) {
                count = count + 1;
            }
            last = *c;
        }
    }
    return count;
}

#[test]
fn it_row_count() {
    assert_eq!(
        6,
        row_count(&vec![
                  "hi there\n".to_string(),
                  "\nhow are you\r".to_string(),
                  "\n\r\nche".to_string(),
                  "ers\r\rbob".to_string()
        ])
    );
}


impl<R: Read> ReaderFrom<R> {

    pub fn new(reader: R, field_seperator: char) -> ReaderFrom<R> {
        return  ReaderFrom {
            rdr: BufReader::new(reader),
            buf: vec![],
            field_seperator,
        };
    }
}

impl<R: Read> std::io::Read for ReaderFrom<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        return Result::Ok(0);
        // let mut to_read = self.src.len() - self.pos;
        // if to_read > buf.len() {
        //     to_read = buf.len();
        // }
        // if to_read == 0 {
        //     return Result::Ok(0);
        // }
        // for i in 0..to_read {
        //     buf[i] = self.src.as_bytes()[i + self.pos];
        // }
        // self.pos = self.pos + to_read;
        // return Result::Ok(to_read);
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
}
