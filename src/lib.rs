extern crate csv;

use std::io::Read;
use csv::ReaderBuilder;

#[cfg(test)]
use std::io::BufReader;

#[cfg(test)]
pub struct FakeCsvReader {
    src: String,
    pos: usize,
}


#[cfg(test)]
impl FakeCsvReader {
    pub fn new(strng: String) -> FakeCsvReader {
        return FakeCsvReader {
            src: strng,
            pos: 0,
        }
    }
}


#[cfg(test)]
impl Read for FakeCsvReader {

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

        Result::Ok(to_read)

    }

}


#[test]
fn fake_reader_works() {

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


struct BufferAcc {
    current_line: usize,
    max_line: usize,
    count: usize,
}

impl std::fmt::Debug for BufferAcc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "BufferAcc({},{},{})",
            self.count,
            self.max_line,
            self.current_line,
        )
    }
}

fn is_nl(c: u8) -> bool {
    if (c == 13) || (c == 10) {
        return true;
    }
    false
}


fn has_nl(input: &Vec<u8>) -> bool {
    let mut last: u8 = 0;
    for i in 0..input.len() {
        if is_nl(last) {
            return true;
        }
        last = input[i];
    }
    false
}


#[cfg(test)]
fn str_to_vec(s: String) -> Vec<u8> {
    let mut v = vec![];
    let bs = s.as_bytes();
    for i in 0..bs.len() {
        v.push(bs[i]);
    }
    v
}


#[test]
fn test_has_nl() {
    assert_eq!(has_nl(&str_to_vec("hi there\r\n".to_string())), true);
    assert_eq!(has_nl(&str_to_vec("hi there\nbob".to_string())), true);
    assert_eq!(has_nl(&str_to_vec("hi there\r".to_string())), false);
    assert_eq!(has_nl(&str_to_vec("hi there\n".to_string())), false);
    assert_eq!(has_nl(&str_to_vec("hi there".to_string())), false);
}


fn fill(rdr: &mut Box<std::io::Read>, v: &mut Vec<u8>) -> Result<(), std::io::Error> {
    let mut bytes_read = 999;
    while (bytes_read > 0) && (!has_nl(&v)) {
        let mut bytes = [0; 1024];
        bytes_read = rdr.read(&mut bytes)?;
        for i in 0..bytes_read {
            v.push(bytes[i]);
        }
    }
    Result::Ok(())
}


#[test]
fn test_fill() {
    let csv = vec![
        "Full of nonsense, rubbish and problems".to_string(),
        "but before the real data".to_string(),
        "name,age,gender".to_string(),
        "bob,22,M".to_string(),
        "jane,21,F".to_string(),
        "freddy,19,M".to_string()
    ];
    let mut fr: Box<Read> = Box::new(FakeCsvReader::new(csv.join("\n")));
    let mut unprocessed = "This is a header".to_string().as_bytes().to_vec();
    fill(&mut fr, &mut unprocessed).unwrap();
    let mut fr_buffer = String::new();
    fr.read_to_string(&mut fr_buffer).unwrap();
    assert_eq!(fr_buffer, "");
    assert_eq!(
        unprocessed,
        ("This is a header".to_string() + &csv.join("\n")).as_bytes()
    );
}


fn get_line(unprocessed: &mut Vec<u8>) -> Vec<u8> {

    let get_byte_count = || {
        let i = 0;
        for i in 1..unprocessed.len() {
            if is_nl(unprocessed[i - 1]) {
                if is_nl(unprocessed[i]) {
                    return i + 1;
                }
                return i;
            }
        }
        i
    };

    let byte_count = get_byte_count();

    let r = unprocessed[..byte_count].to_vec();
    unprocessed.drain(0..byte_count);
    r

}

#[test]
fn test_get_line() {
    let mut unp1 = str_to_vec("hi there\r\n".to_string());
    let r1 = get_line(&mut unp1);
    assert_eq!(r1, str_to_vec("hi there\r\n".to_string()));
    assert_eq!(unp1, str_to_vec("".to_string()));

    let mut unp2 = str_to_vec("hi there\nhow are you bob?".to_string());
    let r2 = get_line(&mut unp2);
    assert_eq!(r2, str_to_vec("hi there\n".to_string()));
    assert_eq!(unp2, str_to_vec("how are you bob?".to_string()));

    let mut unp3 = str_to_vec("\rhi there".to_string());
    let r3 = get_line(&mut unp3);
    assert_eq!(r3, str_to_vec("\r".to_string()));
    assert_eq!(unp3, str_to_vec("hi there".to_string()));

    let mut unp3 = str_to_vec("hi there".to_string());
    let r3 = get_line(&mut unp3);
    assert_eq!(r3, str_to_vec("".to_string()));
    assert_eq!(unp3, str_to_vec("hi there".to_string()));

    let mut unp4 = str_to_vec("".to_string());
    let r4 = get_line(&mut unp4);
    assert_eq!(r4, str_to_vec("".to_string()));
    assert_eq!(unp4, str_to_vec("".to_string()));
}


fn count_seperators(field_seperator: u8, line: &Vec<u8>) -> usize {

    let mut rdr = ReaderBuilder::new()
        .delimiter(field_seperator)
        .has_headers(false)
        .from_reader(line.as_slice());

    match rdr.byte_records().next() {
        Some(rec) => {
            rec.unwrap_or_default().len()
        }
        None => 0
    }

}


#[test]
fn test_count_seperators() {
    assert_eq!(
        count_seperators(44, &str_to_vec("This,has,4,fields".to_string())),
        4
        );
}


pub struct Blade {
    rdr: Box<Read>,
    field_seperator: u8,
    buffer: Vec<Vec<u8>>,
    unprocessed: Vec<u8>,
    prepared: bool,
    consider_lines: usize,
}


/// Takes either a line (sub vector) or part of a line (if `return_buf` is too
/// small) from `src_buffer` and moves it into `return_buf`.
fn read_from_buffer(src_buffer: &mut Vec<Vec<u8>>, return_buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let mut count = src_buffer[0].len();
    let mut shift = true;
    let as_bytes = src_buffer.remove(0);
    if return_buf.len() < as_bytes.len() {
        count = return_buf.len();
        shift = false;
    }

    return_buf[..count].clone_from_slice(&as_bytes[..count]);

    if !shift {
        src_buffer.insert(0, as_bytes[count..].to_vec());
    }

    Result::Ok(count)
}


#[test]
fn test_read_from_buffer_full_line() {
    let mut return_buffer = [0; 4];
    let mut src_buffer = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9]];
    assert_eq!(
        read_from_buffer(&mut src_buffer, &mut return_buffer).unwrap_or_default(),
        4
    );
    println!("{:?}", return_buffer);
    assert_eq!(return_buffer, [1, 2, 3, 4]);
    assert_eq!(src_buffer, vec![vec![5, 6, 7, 8], vec![9]]);
}


#[test]
fn test_read_from_buffer_partial_line() {
    let mut return_buffer = [0; 8];
    let mut src_buffer = vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]];
    assert_eq!(
        read_from_buffer(&mut src_buffer, &mut return_buffer).unwrap_or_default(),
        8
    );
    println!("{:?}", return_buffer);
    assert_eq!(return_buffer, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(src_buffer, vec![vec![9, 10, 11, 12, 13, 14]]);
}


impl Blade {

    fn prepare(&mut self) -> Result<usize, std::io::Error> {

        let mut process_buffer = vec![];

        let mut did_read = true;

        while (process_buffer.len() < self.consider_lines) && did_read {
            let mut read_buffer = vec![];
            fill(&mut self.rdr, &mut read_buffer)?;
            let mut added_length = 9;
            did_read = false;
            while (process_buffer.len() < self.consider_lines) && added_length > 0 {
                let line = get_line(&mut read_buffer);
                added_length = line.len();
                if added_length > 0 {
                    process_buffer.push(line);
                    did_read = true;
                }
            }
            for i in 0..read_buffer.len() {
                self.unprocessed.push(read_buffer[i]);
            }
        }

        let max = (&process_buffer).iter().fold(
            BufferAcc { count: 0, current_line: 0, max_line: 0 },
            |acc, line| {
                let c = count_seperators(
                    self.field_seperator,
                    line
                );
                // println!("> {:?}", c);
                if c <= acc.count {
                    let r = BufferAcc { current_line: acc.current_line + 1, ..acc };
                    // println!("{:?}", r);
                    return r;
                }
                let r = BufferAcc {
                    count: c,
                    current_line: acc.current_line + 1,
                    max_line: acc.current_line
                };
                // println!("{:?}", r);
                r
            }
        );

        // println!("{:?}", max);

        while process_buffer.len() > max.max_line {
            self.buffer.push(process_buffer.remove(max.max_line).to_vec());
        }

        Result::Ok(self.buffer.len())

    }


    pub fn new(reader: Box<Read>, field_seperator: u8, consider_lines: usize) -> Blade {
        Blade {
            rdr: reader,
            field_seperator,
            unprocessed: vec![],
            buffer: vec![],
            prepared: false,
            consider_lines
        }
    }

    fn read_rest(&mut self, return_buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let length = self.unprocessed.len();
        if length > 0 {
            for i in 0..self.unprocessed.len() {
                return_buf[i] = self.unprocessed[i];
            }
            self.unprocessed.clear();
            return Result::Ok(length);
        }
        self.rdr.read(return_buf)
    }
}


impl Read for Blade {

    fn read(&mut self, return_buf: &mut [u8]) -> Result<usize, std::io::Error> {

        if !self.prepared {
            self.prepare()?;
            self.prepared = true;
        }

        if self.buffer.is_empty() {
            return self.read_rest(return_buf);
        }

        read_from_buffer(&mut self.buffer, return_buf)

    }

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
    let rf = Blade::new(Box::new(fr), 44, 20);
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
    let rf = Blade::new(Box::new(fr), 44, 3);
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


