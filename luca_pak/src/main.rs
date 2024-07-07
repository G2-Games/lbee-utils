use std::{fs, path::PathBuf};
use clap::{error::ErrorKind, Error, Parser, Subcommand};
use luca_pak::Pak;

#[derive(Parser)]
#[command(name = "CZ Utils")]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "PAK FILE")]
    input: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extracts the contents of a PAK file into a folder.
    Extract {
        /// Output folder for PAK contents
        #[arg(value_name = "OUTPUT FOLDER")]
        output: PathBuf,
    },

    /// Replace the entries in a PAK file
    Replace {
        /// Replace a whole folder, and output to another folder,
        /// using a folder of replacements
        #[arg(short, long)]
        batch: bool,

        /// The name of the file within the PAK you wish to replace.
        /// If not provided, the filename will be used.
        /// Icompatible with batch mode.
        #[arg(short, long)]
        name: Option<String>,

        /// File or folder to use as a replacement
        #[arg(value_name = "REPLACEMENT")]
        replacement: PathBuf,

        /// Output PAK file location
        #[arg(value_name = "OUTPUT PATH")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    if !cli.input.is_file() {
        Error::raw(ErrorKind::ValueValidation,
            "The input file/folder provided is not a file\n",
        ).exit()
    }

    let mut pak = Pak::open(&cli.input).unwrap();

    match cli.command {
        Commands::Extract { output } => {
            if !output.is_dir() {
                Error::raw(ErrorKind::ValueValidation,
                    "The output given was not a directory\n",
                ).exit()
            }

            for entry in pak.entries() {
                entry.save(&output).unwrap();
            }
        },
        Commands::Replace { batch, name, replacement, output } => {
            if !output.is_file() {
                Error::raw(ErrorKind::ValueValidation,
                    "Replacement output must be a file\n",
                ).exit()
            }

            if batch {
                if name.is_some() {
                    Error::raw(ErrorKind::ValueValidation,
                        "Cannot use name with batch\n",
                    ).exit()
                }

                if !replacement.is_dir() {
                    Error::raw(ErrorKind::ValueValidation,
                        "Batch replacement must be a directory\n",
                    ).exit()
                }

                for entry in fs::read_dir(replacement).unwrap() {
                    let entry = entry.unwrap();
                    let search_name: String = entry
                            .path()
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into();

                    dbg!(&search_name);

                    // Read in the replacement file to a vec
                    let rep_data: Vec<u8> = std::fs::read(entry.path()).unwrap();
                    if let Err(err) = pak.replace_by_name(search_name, &rep_data) {
                        Error::raw(ErrorKind::ValueValidation,
                            format!("Failed to replace file in PAK: {}\n", err),
                        ).exit()
                    }
                }

                pak.save(&output).unwrap();
            } else {
                if !replacement.is_file() {
                    Error::raw(ErrorKind::ValueValidation,
                        "Replacement input must be a file\n",
                    ).exit()
                }

                let search_name = if let Some(name) = name {
                    name
                } else {
                    replacement
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into()
                };

                // Read in the replacement file to a vec
                let rep_data: Vec<u8> = std::fs::read(replacement).unwrap();
                if let Err(err) = pak.replace_by_name(search_name, &rep_data) {
                    Error::raw(ErrorKind::ValueValidation,
                        format!("Failed to replace file in PAK: {}\n", err),
                    ).exit()
                }

                pak.save(&output).unwrap();
            }
        },
    }

    /*
    let rep_cz_data: Vec<u8> = std::fs::read("en_manual01_Linkto_2_6.cz1").unwrap();
    pak.replace(4, &rep_cz_data).unwrap();

    let mut output = BufWriter::new(File::create("MANUAL-modified.PAK").unwrap());
    pak.encode(&mut output).unwrap();
    */
}
