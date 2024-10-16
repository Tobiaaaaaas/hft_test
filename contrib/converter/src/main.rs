use flate2::read::GzDecoder;
use hftbacktest::backtest::data::write_npy_header;
use hftbacktest::types::Event;
use std::fs::{remove_file, File};
use std::io::{copy, BufReader, Seek, SeekFrom, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use clap::Parser;

mod bybit;
mod converter;

use converter::Converter;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    exchange: String,

    #[arg(long)]
    input: String,

    #[arg(long, default_value = "test.npz")]
    output: String,

    #[arg(long, default_value_t = 5_000_000)]
    base_latency: i64,

    #[arg(long, default_value = "/tmp/")]
    temp_dir: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut temp_file = args.temp_dir;
    temp_file.push_str("temp.npy");
    let mut file = File::create(&temp_file)?;

    let decoder = GzDecoder::new(File::open(args.input.clone())?);
    let reader = BufReader::new(decoder);

    let mut converter = Converter::new(&*args.exchange, args.base_latency);

    write_npy_header::<File, Event>(&mut file, 0)?;

    // Actually do the work..
    println!("Converting {} to {}", args.input, &temp_file);
    let counter = converter.process_file(reader, &mut file)?;

    file.seek(SeekFrom::Start(0))?;
    write_npy_header::<File, Event>(&mut file, counter)?;
    file.flush()?;

    let output = File::create(&args.output)?;
    let mut zip = ZipWriter::new(output);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::DEFLATE)
        .compression_level(Some(9));

    zip.start_file("data.npy", options)?;

    println!("Compressing {} to {}", &temp_file, &args.output);
    let mut temp_read = File::open(&temp_file)?;
    copy(&mut temp_read, &mut zip)?;
    zip.finish()?;

    println!("Removing {}", &temp_file);
    remove_file(&temp_file)?;

    Ok(())
}
