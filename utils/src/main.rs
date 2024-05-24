use cz::DynamicCz;

use std::path::PathBuf;

use clap::{error::ErrorKind, Error, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "CZ Utils")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Converts a CZ file to a PNG
    Decode {
        /// Input CZ file of any type
        #[arg(value_name = "CZ FILE")]
        input: PathBuf,

        /// Output PNG file location
        #[arg(value_name = "PATH")]
        output: Option<PathBuf>,
    },

    /// Replace a CZ file's image data
    Replace {
        /// Original input CZ file of any type
        #[arg(value_name = "CZ FILE")]
        input: PathBuf,

        /// Image to use as the replacement
        #[arg(value_name = "IMAGE")]
        replacement: PathBuf,

        /// Output CZ file location
        #[arg(value_name = "PATH")]
        output: PathBuf,

        /// Output CZ file version
        #[arg(short, long, value_name = "CZ VERSION")]
        version: Option<u8>,
    }
}

fn main() {
    let cli = Cli::parse();

    // Check what subcommand was run
    match &cli.command {
        Commands::Decode { input, output } => {
            if !input.exists() {
                Error::raw(
                    ErrorKind::ValueValidation,
                    format!("The input file provided does not exist\n")
                ).exit()
            }

            let cz = match DynamicCz::open(input) {
                Ok(cz) => cz,
                Err(err) => {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Could not open input as a CZ file: {}\n", err)
                    ).exit()
                },
            };

            if let Some(output) = output {
                cz.save_as_png(output).unwrap();
            } else {
                let file_stem = PathBuf::from(input.file_stem().unwrap());
                cz.save_as_png(&file_stem.with_extension("png")).unwrap();
            }
        }
        Commands::Replace { input, replacement, output, version } => {
            if !input.exists() {
                Error::raw(
                    ErrorKind::ValueValidation,
                    format!("The original file provided does not exist\n")
                ).exit()
            }

            if !replacement.exists() {
                Error::raw(
                    ErrorKind::ValueValidation,
                    format!("The replacement file provided does not exist\n")
                ).exit()
            }

            let mut cz = match DynamicCz::open(input) {
                Ok(cz) => cz,
                Err(err) => {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Could not open input as a CZ file: {}\n", err)
                    ).exit()
                },
            };

            let repl_img = match image::open(replacement) {
                Ok(img) => img,
                Err(err) => {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Could not open replacement file as an image: {}\n", err)
                    ).exit()
                },
            };
            let repl_img = repl_img.to_rgba8();

            cz.header_mut().set_width(repl_img.width() as u16);
            cz.header_mut().set_height(repl_img.height() as u16);
            cz.set_bitmap(repl_img.into_raw());

            if let Some(ver) = version {
                match cz.header_mut().set_version(*ver) {
                    Ok(_) => (),
                    Err(_) => {
                        Error::raw(
                            ErrorKind::ValueValidation,
                            format!("Invalid CZ Version {}; expected 0, 1, 2, 3, or 4\n", ver)
                        ).exit()
                    },
                };
            }

            match cz.save_as_cz(output) {
                Ok(_) => (),
                Err(err) => {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Failed to save CZ file: {}\n", err)
                    ).exit()
                },
            }
        },
    }
}
