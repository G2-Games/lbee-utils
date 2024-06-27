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
        /// Decode a whole folder, and output to another folder
        #[arg(short, long)]
        batch: bool,

        /// Input CZ file of any type
        #[arg(value_name = "CZ FILE")]
        input: PathBuf,

        /// Output PNG file location
        #[arg(value_name = "PATH")]
        output: Option<PathBuf>,
    },

    /// Replace a CZ file's image data
    Replace {
        /// Replace a whole folder, and output to another folder,
        /// using a folder of replacements
        #[arg(short, long)]
        batch: bool,

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

        /// Output CZ file bit depth
        #[arg(short, long, value_name = "BIT DEPTH")]
        depth: Option<u8>,
    }
}

fn main() {
    let cli = Cli::parse();

    // Check what subcommand was run
    match &cli.command {
        Commands::Decode { input, output, batch } => {
            if !input.exists() {
                Error::raw(
                    ErrorKind::ValueValidation,
                    format!("The input file/folder provided does not exist\n")
                ).exit()
            }

            if *batch {
                if input.is_file() {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Batch input must be a directory\n")
                    ).exit()
                }

                if output.is_none() || output.as_ref().unwrap().is_file() {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Batch output must be a directory\n")
                    ).exit()
                }

                for entry in walkdir::WalkDir::new(input).max_depth(1) {
                    let path = entry.unwrap().into_path();
                    if !path.is_file() {
                        continue;
                    }

                    let filename = PathBuf::from(path.file_name().unwrap());
                    let filename = filename.with_extension("png");

                    let mut final_path = output.clone().unwrap();
                    final_path.push(filename);

                    let cz = match DynamicCz::open(&path) {
                        Ok(cz) => cz,
                        Err(_) => {
                            Error::raw(
                                ErrorKind::ValueValidation,
                                format!("Could not open input as a CZ file: {}\n", path.into_os_string().to_str().unwrap())
                            ).print().unwrap();
                            continue;
                        },
                    };

                    cz.save_as_png(&final_path).unwrap();
                }
            } else {
                let cz = DynamicCz::open(input).unwrap();

                if let Some(output) = output {
                    cz.save_as_png(output).unwrap();
                } else {
                    let file_stem = PathBuf::from(input.file_name().unwrap());
                    cz.save_as_png(&file_stem.with_extension("png")).unwrap();
                }
            }
        }
        Commands::Replace { batch, input, replacement, output, version, depth } => {
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

            if *batch {
                if input.is_file() {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Batch input must be a directory\n")
                    ).exit()
                }

                if replacement.is_file() {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Batch replacement location must be a directory\n")
                    ).exit()
                }

                if output.is_file() {
                    Error::raw(
                        ErrorKind::ValueValidation,
                        format!("Batch output must be a directory\n")
                    ).exit()
                }

                for entry in walkdir::WalkDir::new(input).max_depth(1) {
                    let path = entry.unwrap().into_path();
                    if !path.is_file() {
                        continue;
                    }

                    let filename = PathBuf::from(path.file_name().unwrap());

                    let mut final_path = output.clone();
                    final_path.push(&filename);

                    let mut final_replacement = replacement.clone();
                    final_replacement.push(filename.with_extension("png"));

                    let repl_img = match image::open(&final_replacement) {
                        Ok(img) => img,
                        Err(_) => {
                            Error::raw(
                                ErrorKind::ValueValidation,
                                format!("Could not open replacement file as an image: {}\n", final_replacement.into_os_string().to_str().unwrap())
                            ).exit()
                        },
                    };
                    let repl_img = repl_img.to_rgba8();

                    let mut cz = match DynamicCz::open(&path) {
                        Ok(cz) => cz,
                        Err(_) => {
                            Error::raw(
                                ErrorKind::ValueValidation,
                                format!("Could not open input as a CZ file: {}\n", path.into_os_string().to_str().unwrap())
                            ).print().unwrap();
                            continue;
                        },
                    };

                    cz.header_mut().set_width(repl_img.width() as u16);
                    cz.header_mut().set_height(repl_img.height() as u16);
                    cz.set_bitmap(repl_img.into_raw());
                    cz.remove_palette();

                    if let Some(depth) = depth {
                        cz.header_mut().set_depth(*depth as u16)
                    }

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

                    cz.save_as_cz(&final_path).unwrap();
                }
            } else {
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
                cz.remove_palette();

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
            }
        },
    }
}
