use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use structopt::StructOpt;

const TARGET_SIZE: u64 = 20 * 1024 * 1024; // 20 MiB


/// Compress input into multiple gz parts not exceeding a given target size.
#[derive(Debug, StructOpt)]
struct Cli {
    /// The output file prefix
    output_prefix: String,
    /// The input file to compress, stdin if not present
    input: Option<String>,
}

fn get_part_filename(prefix: &str, part: u32) -> String {
    format!("{}-{:08}.gz", prefix, part)
}

fn open_output_part(prefix: &str, part: u32) -> std::io::Result<GzEncoder<File>> {
    let output_file = File::create(get_part_filename(prefix, part))?;
    Ok(GzEncoder::new(output_file, flate2::Compression::default()))
}

fn main() -> std::io::Result<()> {
    let args: Cli = Cli::from_args();

    let input_file: Box<dyn Read> = if let Some(input) = args.input {
        Box::new(File::open(input)?)
    } else {
        Box::new(std::io::stdin())
    };
    let input_file = GzDecoder::new(input_file);
    let input_file = BufReader::new(input_file);

    let mut part = 0;
    let mut i = 0;

    let mut output_file = open_output_part(args.output_prefix.as_str(), part)?;

    for line in input_file.lines() {
        i += 1;

        let line = line.unwrap();
        output_file.write(line.as_bytes()).expect("unexpected error while writing data");
        output_file.write("\n".as_bytes()).expect("unexpected error while writing data");

        if i > 10000 {
            i = 0;
            output_file.flush().expect("unexpected error while flushing");
            let output_file_size = output_file.get_ref().seek(std::io::SeekFrom::Current(0))?;

            if output_file_size >= TARGET_SIZE {
                // create new part
                part += 1;
                output_file = open_output_part(args.output_prefix.as_str(), part)?;
            }
        }
    }

    Ok(())
}
