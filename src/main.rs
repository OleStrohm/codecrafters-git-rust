use anyhow::Context;
use clap::Parser;
use clap::Subcommand;
use flate2::read::ZlibDecoder;
use std::fs;
use std::fs::File;
use std::io::stdout;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init,
    CatFile {
        #[arg(short)]
        pretty_print: bool,
        hash: String,
    },
}

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Commands::CatFile { pretty_print, hash } => {
            if !pretty_print {
                anyhow::bail!("Only pretty printing is supported");
            }
            if hash.len() != 40 {
                anyhow::bail!("hash must be 40 characters");
            }

            let (hash_initial, hash_rest) = hash.split_at(2);
            let object_file = &format!(".git/objects/{}/{}", hash_initial, hash_rest);
            let file =
                File::open(object_file).with_context(|| format!("Opening `{object_file}`"))?;
            let mut decoder = BufReader::new(ZlibDecoder::new(file));

            let mut header = Vec::new();
            let _ = decoder
                .read_until(b'\0', &mut header)
                .context("Reading header")?;

            let (kind, size) = header.split_once(b' ').ok_or(anyhow::anyhow!("Header has no space"));
            eprintln!("Decompressed size of object file: {size}");

            let mut data = Vec::new();
            let size = decoder
                .read_to_end(&mut data)
                .context("Decompressing object file")?;

            write!(stdout(), "Data: {}", String::from_utf8_lossy(&data))
                .context("Writing data to stdout")?;
        }
    }

    Ok(())
}
