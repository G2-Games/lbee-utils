use clap::{
    error::{Error, ErrorKind},
    Parser, Subcommand,
};
use luca_pak::Pak;
use std::{fs, path::PathBuf, process::exit};

/// Utility to maniuplate PAK archive files from the LUCA System game engine by
/// Prototype Ltd.
#[derive(Parser)]
#[command(name = "PAK Utility")]
#[command(author, version, about, long_about = None, disable_version_flag = true)]
struct Cli {
    /// Show program version information
    #[arg(short('V'), long)]
    version: bool,

    #[arg(value_name = "PAK FILE", required_unless_present("version"))]
    input: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
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
        /// Incompatible with batch mode, and ID.
        #[arg(short, long)]
        name: Option<String>,

        /// The ID of the file within the PAK you wish to replace.
        /// If not provided, the filename will be used.
        /// Incompatible with batch mode, and name.
        #[arg(short, long)]
        id: Option<u32>,

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

    if cli.version {
        println!(
            "{}, {} v{}-{}",
            env!("CARGO_BIN_NAME"),
                 env!("CARGO_PKG_NAME"),
                 env!("CARGO_PKG_VERSION"),
                 &env!("VERGEN_GIT_SHA")[0..=6]
        );
        exit(0);
    }

    let mut pak = match Pak::open(&cli.input.unwrap()) {
        Ok(pak) => pak,
        Err(err) => fmt_error(&format!("Could not open PAK file: {}", err)).exit(),
    };

    let command = match cli.command {
        Some(c) => c,
        None => {
            exit(0);
        },
    };

    match command {
        Commands::Extract { output } => {
            if output.exists() && !output.is_dir() {
                fmt_error("The output given was not a directory").exit()
            } else if !output.exists() {
                fs::create_dir(&output).unwrap();
            }

            for entry in pak.entries() {
                let mut outpath = output.clone();
                if let Some(n) = entry.name() {
                    outpath.push(n);
                } else {
                    outpath.push(entry.index().to_string())
                }
                entry.save(&outpath).unwrap();
            }
        }
        Commands::Replace {
            batch,
            name,
            id,
            replacement,
            output,
        } => {
            if id.is_some() && name.is_some() {
                fmt_error("Cannot use ID and name together").exit()
            }
            if batch {
                if name.is_some() || id.is_some() {
                    fmt_error("Cannot use name or ID with batch").exit()
                }

                if !replacement.is_dir() {
                    fmt_error("Batch replacement must be a directory").exit()
                }

                for entry in fs::read_dir(replacement).unwrap() {
                    let entry = entry.unwrap();
                    let search_name: String =
                        entry.path().file_name().unwrap().to_string_lossy().into();

                    let parsed_id: Option<u32> = search_name.parse().ok();

                    // Read in the replacement file to a vec
                    let rep_data: Vec<u8> = std::fs::read(entry.path()).unwrap();

                    // Try replacing by name, if that fails, replace by parsed ID
                    if pak.replace_by_name(search_name, &rep_data).is_err() {
                        fmt_error("Could not replace entry in PAK: Could not find name")
                            .print()
                            .unwrap()
                    } else if parsed_id.is_some()
                        && pak.replace_by_id(parsed_id.unwrap(), &rep_data).is_err()
                    {
                        fmt_error("Could not replace entry in PAK: ID is invalid")
                            .print()
                            .unwrap()
                    }
                }
            } else {
                if !replacement.is_file() {
                    fmt_error("Replacement input must be a file").exit()
                }

                let search_name = if let Some(name) = name {
                    name
                } else {
                    replacement.file_name().unwrap().to_string_lossy().into()
                };

                let search_id = if id.is_some() {
                    id
                } else if let Ok(id) = search_name.parse::<u32>() {
                    Some(id)
                } else {
                    None
                };

                // Read in the replacement file to a vec
                let rep_data: Vec<u8> = std::fs::read(replacement).unwrap();
                if id.is_some() {
                    if pak.replace_by_id(search_id.unwrap(), &rep_data).is_err() {
                        fmt_error("Could not replace entry in PAK: ID is invalid").exit()
                    }
                } else if pak.replace_by_name(search_name, &rep_data).is_err() {
                    fmt_error("Could not replace entry in PAK: Could not find name").exit()
                }

                pak.save(&output).unwrap();
            }

            pak.save(&output).unwrap();
        }
    }
}

#[inline(always)]
fn fmt_error(message: &str) -> Error {
    Error::raw(ErrorKind::ValueValidation, format!("{}\n", message))
}
