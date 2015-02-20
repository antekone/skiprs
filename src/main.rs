#![feature(fs)]
#![feature(io)]

extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;

use docopt::Docopt;
use std::io::prelude::*;
use std::fs::{OpenOptions,File};
use std::io::{BufReader,BufWriter,SeekFrom};

static USAGE: &'static str = "
Usage: skip [--from-sector N] [--from-offset N] [--to-sector N] [--to-offset N] <in> <out>

Options:
    --from-sector N    - start from this sector number,
    --from-offset N    - start from this offset,
    --to-sector N      - write from this sector number,
    --to-offset N      - write from this offset.

Example:

    ./skip --in file.in --out dumped.bin --from-sector 123 --to-offset 8716234

This will process file `file.in` and will save the result to `dumped.bin`. Additionally,
it will process `file.bin` since sector 123, and will start saving to `dumped.bin` from
offset 8716234.";

#[derive(RustcDecodable)]
struct Args {
    arg_in: String,
    arg_out: String,
    flag_from_sector: Option<u64>,
    flag_from_offset: Option<u64>,
    flag_to_sector: Option<u64>,
    flag_to_offset: Option<u64>,
}

fn get_offset(offset_opt: Option<u64>, sector_opt: Option<u64>) -> u64 {
    if let Some(x) = offset_opt { return x; }
    if let Some(x) = sector_opt { return x * 512; }
    return 0;
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|dopt| dopt.decode()).unwrap_or_else(|e| e.exit());

    let input_offset = get_offset(args.flag_from_offset, args.flag_from_sector);
    let output_offset = get_offset(args.flag_to_offset,  args.flag_to_sector);

    process_file(args.arg_in, args.arg_out, input_offset, output_offset);
}

fn process_file(name_in: String, name_out: String, read_offset: u64, write_offset: u64) {
    println!("processing in={}, out={}, read={}, write={}", name_in, name_out, read_offset, write_offset);

    let mut in_file = match File::open(&name_in) {
        Ok(x) => x,
        Err(_) => { println!("Error opening input file: {}", name_in); return; }
    };

    let mut out_file = match OpenOptions::new().write(true).create(true).truncate(false).open(&name_out) {
        Ok(x) => x,
        Err(_) => { println!("Error opening/creating output file: {}", name_out); return; }
    };

    in_file.seek(SeekFrom::Start(read_offset)).unwrap();
    out_file.seek(SeekFrom::Start(write_offset)).unwrap();

    let file_size = in_file.metadata().unwrap().len();
    println!("Input file size: {}", file_size);

    if read_offset >= file_size {
        println!("Read offset is bigger than the file size -- nothing to do.");
        return;
    }

    let mut remaining_bytes = file_size - read_offset;
    println!("Remaining bytes: {}", remaining_bytes);

    let mut reader = BufReader::new(in_file);
    let mut writer = BufWriter::new(out_file);

    let mut buf = [0u8; 512];
    let mut sbuf = [0u8; 8];

    while remaining_bytes > 0 {
        if let Ok(x) = reader.read(&mut buf) {
            if x > 0 {
                remaining_bytes -= x as u64;
            } else {
                println!("I/O error on reading");
                return;
            }
        } else {
            println!("I/O error on reading");
            return;
        }

        if let Err(_) = reader.read(&mut sbuf) {
            println!("I/O error while reading");
            return;
        }

        if let Err(_) = writer.write(&buf) {
            println!("I/O error while saving");
            return;
        }
    }

    println!("done");
}

