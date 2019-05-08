extern crate csv;

use std::io::Read;
use csv::ReaderBuilder;

#[cfg(test)]
use std::io::BufReader;

#[cfg(test)]
pub struct FakeCsvReader {
    src: String,
    pos: usize,
    max_read: Option<usize>,
}


#[cfg(test)]
impl FakeCsvReader {
    pub fn new_by_size(strng: String, size: usize) -> FakeCsvReader {
        return FakeCsvReader {
            src: strng,
            pos: 0,
            max_read: Option::Some(size),
        }
    }
    pub fn new(strng: String) -> FakeCsvReader {
        return FakeCsvReader {
            src: strng,
            pos: 0,
            max_read: Option::None,
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

        if to_read > self.max_read.unwrap_or(to_read) {
            to_read = self.max_read.unwrap_or(to_read);
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


fn has_nl(input: &[u8]) -> bool {
    let mut last: u8 = 0;
    for inp in input {
        if is_nl(last) {
            return true;
        }
        last = *inp;
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


/// Writes bytes to `rdr` to `v`, however it will keep doing this until at least
/// one new line is found.
fn fill(rdr: &mut std::io::Read, v: &mut Vec<u8>) -> Result<(), std::io::Error> {
    let mut number_of_bytes_read = 999;
    while (number_of_bytes_read > 0) && (!has_nl(&v)) {
        #[cfg(test)]
        let mut bytes = [0; 8];
        #[cfg(not(test))]
        let mut bytes = [0; 8192];
        number_of_bytes_read = rdr.read(&mut bytes)?;
        v.append(&mut bytes[0..number_of_bytes_read].to_vec());
    }
    Result::Ok(())
}


#[test]
fn test_fill() {
    let csv = vec![
        "full of trash".to_string(),
        "but before the real data".to_string(),
        "a".to_string(),
    ];
    let mut fr: Box<Read> = Box::new(FakeCsvReader::new(csv.join("\n")));
    let mut unprocessed = "This is a header ".to_string().as_bytes().to_vec();
    fill(&mut fr, &mut unprocessed).unwrap();
    let mut fr_buffer = String::new();
    fr.read_to_string(&mut fr_buffer).unwrap();
    assert_eq!(fr_buffer, "t before the real data\na");
    assert_eq!(
        unprocessed,
        "This is a header full of trash\nbu".to_string().as_bytes()
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


fn count_seperators(field_seperator: u8, line: &[u8]) -> usize {

    let mut rdr = ReaderBuilder::new()
        .delimiter(field_seperator)
        .has_headers(false)
        .from_reader(line);

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


type Buffer = Vec<Vec<u8>>;

pub struct Blade {
    rdr: Box<Read>,
    field_seperator: u8,
    buffer: Buffer,
    unprocessed: Buffer,
    prepared: bool,
    consider_lines: usize,
}


/// Takes either a line (sub vector) or part of a line (if `return_buf` is too
/// small) from `src_buffer` and moves it into `return_buf`.
fn read_from_buffer(src_buffer: &mut Buffer, return_buf: &mut [u8]) -> Result<usize, std::io::Error> {
    if src_buffer.is_empty() {
        return Result::Ok(0);
    }
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
fn test_read_from_buffer_empty() {
    let mut return_buffer = [0; 4];
    let mut src_buffer: Buffer = vec![vec![]];
    let expected: Buffer = vec![];
    assert_eq!(
        read_from_buffer(&mut src_buffer, &mut return_buffer).unwrap_or_default(),
        0
    );
    assert_eq!(return_buffer, [0; 4]);
    assert_eq!(src_buffer, expected);
}


#[test]
fn test_read_from_buffer_full_line() {
    let mut return_buffer = [0; 4];
    let mut src_buffer = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9]];
    assert_eq!(
        read_from_buffer(&mut src_buffer, &mut return_buffer).unwrap_or_default(),
        4
    );
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
    assert_eq!(return_buffer, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(src_buffer, vec![vec![9, 10, 11, 12, 13, 14]]);
}


/// Reads `consider_lines` from `rdr` putting them into `process_buffer`. Any
/// left over lines will be put in `unprocessed`.
fn prepare_fill(consider_lines: usize, rdr: &mut std::io::Read, process_buffer: &mut Buffer, unprocessed: &mut Buffer) -> Result<(), std::io::Error> {
    let mut did_read = true;

    let mut read_buffer = vec![];

    while (process_buffer.len() < consider_lines) && did_read {
        fill(rdr, &mut read_buffer)?;
        let mut added_length = 9;
        did_read = false;
        while (process_buffer.len() < consider_lines) && added_length > 0 {
            let line = get_line(&mut read_buffer);
            added_length = line.len();
            if added_length > 0 {
                process_buffer.push(line);
                did_read = true;
            }
        }
    }

    let mut unproc = vec![];
    for r in read_buffer {
        unproc.push(r);
    }

    unprocessed.push(unproc);

    Result::Ok(())

}


#[test]
fn test_prepare_fill_needs_multiple_reads() {
    let csv = vec![
        "01234".to_string(),
        "56789".to_string(),
        "abcde".to_string(),
        "defgh".to_string(),
    ];
    let fr = FakeCsvReader::new_by_size(csv.join("\n"), 7);
    let mut b: Box<std::io::Read> = Box::new(fr);
    
    let mut return_buffer: Buffer = vec![];
    let mut unprocessed: Buffer = vec![];
    assert_eq!(
        prepare_fill(2, &mut b, &mut return_buffer, &mut unprocessed).unwrap(),
        ()
    );

    let expected: Buffer = vec![
        vec![48, 49, 50, 51, 52, 10],
        vec![53, 54, 55, 56, 57, 10]
    ];
    assert_eq!(return_buffer, expected);
    assert_eq!(unprocessed, vec![vec![97, 98]]);
}


impl Blade {

    fn prepare(&mut self) -> Result<usize, std::io::Error> {

        let mut process_buffer = vec![];
        let mut unprocessed = vec![];

        prepare_fill(self.consider_lines, &mut self.rdr, &mut process_buffer, &mut unprocessed)?;

        self.unprocessed = unprocessed;

        let max = (&process_buffer).iter().fold(
            BufferAcc { count: 0, current_line: 0, max_line: 0 },
            |acc, line| {
                let c = count_seperators(
                    self.field_seperator,
                    line
                );
                if c <= acc.count {
                    let r = BufferAcc { current_line: acc.current_line + 1, ..acc };
                    return r;
                }

                BufferAcc {
                    count: c,
                    current_line: acc.current_line + 1,
                    max_line: acc.current_line
                }
            }
        );

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
            return read_from_buffer(&mut self.unprocessed, return_buf);
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


