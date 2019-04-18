use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Shl;
use std::num::Wrapping;

struct Reader {
    buffer: [u8; 256],
    f: File,
    size: usize,
    index: usize,
}

const FLASH_SIZE: usize = 8 * 1024;

fn char_to_byte(c: u8) -> Option<u8> {
    let zero = '0' as u8;
    let nine = '9' as u8;
    let a = 'a' as u8;
    let f = 'f' as u8;
    let cap_a = 'A' as u8;
    let cap_f = 'F' as u8;

    if c >= zero && c <= nine {
        Some(c - zero)
    }
    else if c >= cap_a && c <= cap_f {
        Some(c - cap_a + 10)
    }
    else if c >= a && c <= f {
        Some(c - a + 10)
    }
    else {
        None
    }
}

#[test]
fn test_char_to_byte() {
    assert_eq!(char_to_byte('0' as u8), Some(0));
    assert_eq!(char_to_byte('9' as u8), Some(9));
    assert_eq!(char_to_byte('a' as u8), Some(10));
    assert_eq!(char_to_byte('A' as u8), Some(10));
    assert_eq!(char_to_byte('f' as u8), Some(15));
    assert_eq!(char_to_byte('F' as u8), Some(15));
    assert_eq!(char_to_byte('x' as u8), None);
}

fn two_char_to_byte(char1: u8, char2: u8) -> Option<u8> {
    let n1 = char_to_byte(char1)?;
    let n2 = char_to_byte(char2)?;
    Some((n1 << 4) + n2)
}

#[test]
fn test_two_char_to_byte() {
    assert_eq!(two_char_to_byte('a' as u8, '7' as u8), Some(0xa7));
    assert_eq!(two_char_to_byte('x' as u8, '7' as u8), None);
}

impl Reader {
    fn new(path: &str) -> Option<Reader> {
        if let Ok(mut f) = File::open(path) {
            let mut buffer: [u8; 256] = [0; 256];
            let size = f.read(&mut buffer).unwrap();
            Some(Reader {
                buffer: buffer,
                f: f,
                size: size,
                index: 0,
            })
        }
        else {
            None
        }
    }

    fn read_char(&mut self) -> Option<u8> {
        if self.index == self.size {
            self.size = self.f.read(&mut self.buffer).unwrap();
            if self.size == 0 {
                None
            }
            else {
                self.index = 1;
                Some(self.buffer[0])
            }
        }
        else {
            let b = self.buffer[self.index];
            self.index += 1;
            Some(b)
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        let c1 = self.read_char()?;
        let c2 = self.read_char()?;
        two_char_to_byte(c1, c2)
    }

    fn read_line(&mut self) -> Option<(u16, u8, Vec<u8>)> {
        let colon = ':' as u8;
        while self.read_char()? != colon {}

        let mut vec = vec![];

        let len = self.read_byte()?;

        let mut sum: Wrapping<u8> = Wrapping(len);

        let b1 = self.read_byte()?;
        let b2 = self.read_byte()?;

        sum += Wrapping(b1);
        sum += Wrapping(b2);

        let addr: u16 = Shl::shl(b1 as u16, 8) + b2 as u16;

        let kind = self.read_byte()?;
        sum += Wrapping(kind);

        for _ in 0..len {
            let b = self.read_byte()?;
            sum += Wrapping(b);
            vec.push(b);
        }
        let checksum = self.read_byte()?;
        sum += Wrapping(checksum);
        if sum == Wrapping(0) {
            Some((addr, kind, vec))
        }
        else {
            None
        }
    }

    fn read(&mut self) -> Option<Vec<u8>> {
        let mut result = Vec::new();
        result.resize(FLASH_SIZE, 0xff);
        loop {
            let (addr, kind, vec) = self.read_line()?;
            if kind == 4 {
                continue;
            }
            else if kind == 1 {
                break;
            }
            else if kind == 0 {
                for i in 0..vec.len() {
                    result[addr as usize + i] = vec[i];
                }
            }
            else {
                return None;
            }
        }
        Some(result)
    }
}

fn main() {
    let mut args: Vec<String> = vec![];

    for arg in env::args() {
        args.push(arg);
    }

    if args.len() <= 1 {
        println!("Missing file name argument");
    }
    else if args.len() == 2 {
        println!("Missing output file");
    }
    else {
        if let Some(mut reader) = Reader::new(&args[1]) {
            let vec = reader.read().expect("Error while reading file");
            let mut output = File::create(&args[2]).unwrap();
            output.write_all(&vec).unwrap();
        }
        else {
            println!("File doesn't exist");
        }
    }
}
