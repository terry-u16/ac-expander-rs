use std::io::{stdin, BufRead, BufReader};

pub fn read_int() -> i64 {
    let stdin = stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    line.trim().parse().unwrap()
}

pub fn read_string() -> String {
    let stdin = stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    line.trim().to_string()
}
