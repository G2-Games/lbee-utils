use cz::DynamicCz;
use std::path::{Path, PathBuf};
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
        depth: Option<u16>,
    }
}

fn main() {
    let cli = Cli::parse();

    // Check what subcommand was run
    match &cli.command {
        Commands::Decode { input, output, batch } => {
            if !input.exists() {
                Error::raw(ErrorKind::ValueValidation, "The input file/folder provided does not exist\n").exit()
            }

            if *batch {
                if input.is_file() {
                    Error::raw(ErrorKind::ValueValidation, "Batch input must be a directory\n").exit()
                }

                if output.is_none() || output.as_ref().unwrap().is_file() {
                    Error::raw(ErrorKind::ValueValidation, "Batch output must be a directory\n").exit()
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
                Error::raw(ErrorKind::ValueValidation, "The original file provided does not exist\n").exit()
            }

            if !replacement.exists() {
                Error::raw(ErrorKind::ValueValidation, "The replacement file provided does not exist\n").exit()
            }

            // If it's a batch replacement, we want directories to search
            if *batch {
                if !input.is_dir() {
                    Error::raw(ErrorKind::ValueValidation, "Batch input location must be a directory\n").exit()
                }

                if !replacement.is_dir() {
                    Error::raw(ErrorKind::ValueValidation, "Batch replacement location must be a directory\n").exit()
                }

                if !output.is_dir() {
                    Error::raw(ErrorKind::ValueValidation, "Batch output location must be a directory\n").exit()
                }

                // Replace all the files within the directory and print errors for them
                for entry in walkdir::WalkDir::new(input)
                    .max_depth(1)
                    .same_file_system(true)
                {
                    let path = entry.unwrap().into_path();
                    if !path.is_file() {
                        continue
                    }

                    // Set the replacement image to the same name as the original file
                    let mut final_replacement = replacement.to_path_buf();
                    final_replacement.push(PathBuf::from(path.file_name().unwrap()).with_extension("png"));

                    // Set the replacement image to the same name as the original file
                    let mut final_output = output.to_path_buf();
                    final_output.push(path.file_name().unwrap());

                    if let Err(error) = replace_cz(
                        &path,
                        &final_output,
                        &final_replacement,
                        version,
                        depth
                    ) {
                        Error::raw(ErrorKind::ValueValidation, format!("{:?} - {}\n", path, error)).print().unwrap();
                    }
                }
            } else {
                if !input.is_file() {
                    Error::raw(ErrorKind::ValueValidation, "Input must be a file\n").exit()
                }

                if !replacement.is_file() {
                    Error::raw(ErrorKind::ValueValidation, "Replacement must be a file\n").exit()
                }

                if !output.is_file() {
                    Error::raw(ErrorKind::ValueValidation, "Replacement output must be a file\n").exit()
                }

                // Replace the input file with the new image
                replace_cz(
                    &input,
                    &output,
                    &replacement,
                    version,
                    depth
                ).unwrap();
            }
        },
    }
}

/// Replace a CZ file with the bitmap of a PNG file
fn replace_cz<P: ?Sized + AsRef<Path>>(
    input_path: &P,
    output_path: &P,
    replacement_path: &P,
    version: &Option<u8>,
    depth: &Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = input_path.as_ref();
    if !path.is_file() {
        return Err("Input path is not a file".into())
    }

    if !replacement_path.as_ref().exists() || !replacement_path.as_ref().is_file() {
        return Err("Replacement path does not exist or is not a file".into())
    }

    // Open the replacement image and convert it to RGBA8
    let repl_img = image::open(&replacement_path)?.to_rgba8();

    // Open the original CZ file
    let mut cz = DynamicCz::open(&path)?;

    // Set CZ header parameters and the new bitmap
    cz.header_mut().set_width(repl_img.width() as u16);
    cz.header_mut().set_height(repl_img.height() as u16);
    cz.set_bitmap(repl_img.into_raw());
    cz.remove_palette();

    if let Some(depth) = depth {
        cz.header_mut().set_depth(*depth)
    }

    if let Some(ver) = version {
        cz.header_mut().set_version(*ver)?;
    }

    // Save the file to the proper output location
    cz.save_as_cz(&output_path.as_ref()).unwrap();

    Ok(())
}
