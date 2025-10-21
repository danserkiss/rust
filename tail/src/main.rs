use clap::Parser;
use std::{
    fs::File,
    io::{Read, Seek},
};

/// Simple program to read last n fields in file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of strings to output
    #[arg(short, long, default_value_t = 10)]
    n: i64,

    file: String,
}

fn read_byte(file: &mut File, row: &mut i64, size: &mut u64, i: i64) {
    let mut buf: [u8; 1] = [0; 1];
    let res1 = file.seek(std::io::SeekFrom::End(-1 * i));
    let res2 = file.read_exact(&mut buf);

    for a in buf {
        if a == b'\n' {
            *row += 1;
        }
        *size += 1;
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut i = 1;
    let mut file = File::open(args.file).unwrap();

    let mut row: i64 = 1;
    let mut size: u64 = 0;

    let mut buffer = String::new();

    let filelen = file.metadata()?.len();
    while true {
        read_byte(&mut file, &mut row, &mut size, i);
        i += 1;
        if row > args.n || size >= filelen {
            break;
        }
    }
    let i64_size = size as i64;
    let res1 = file.seek(std::io::SeekFrom::End(-i64_size));
    let res2 = file.read_to_string(&mut buffer);
    println!("{}", buffer);

    Ok(())
}
