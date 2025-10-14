use clap::{error::ErrorKind, Error, Parser, Subcommand};
use cz::{common::CzVersion, CzFile};
use image::ColorType;
use lbee_utils::version;
use owo_colors::OwoColorize;
use std::{
    fs, num::ParseIntError, path::{Path, PathBuf}, process::exit
};

/// Utility to maniuplate CZ image files from the LUCA System game engine by
/// Prototype Ltd.
#[derive(Parser)]
#[command(name = "CZ Utility")]
#[command(author, version, about, long_about = None, disable_version_flag = true)]
#[command(arg_required_else_help(true))]
struct Cli {
    /// Show program version information
    #[arg(short('V'), long)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Decode a CZ file to a PNG
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

    /// Encode a PNG file to a CZ
    Encode {
        /// Input image to encode
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output CZ file location
        #[arg(value_name = "OUTPUT")]
        output: PathBuf,

        /// Output CZ file version
        #[arg(short, long, value_name = "CZ VERSION")]
        version: Option<u8>,

        /// Output CZ file bit depth
        #[arg(short, long, value_name = "CZ BIT DEPTH")]
        depth: Option<u16>,
    },

    /// Replace an existing CZ file's image data
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

        /// Setting this flag turns off the automatic extended header
        /// modification.
        #[arg(short, long)]
        no_auto_bounds: bool,

        /// Set the extended header crop (ex. 1280x720)
        #[arg(short, long, value_name = "CROP")]
        crop: Option<String>,

        /// Set the extended header bounds (ex. 1280x720)
        #[arg(short, long, value_name = "BOUNDS")]
        bounds: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("{}", version(env!("CARGO_BIN_NAME")));
        exit(0);
    }

    let command = match cli.command {
        Some(c) => c,
        None => exit(0),
    };

    // Check what subcommand was run
    match &command {
        Commands::Decode {
            input,
            output,
            batch,
        } => {
            if !input.exists() {
                pretty_error("The input file/folder provided does not exist");
                exit(1);
            }

            if *batch {
                if input.is_file() {
                    pretty_error("Batch input must be a directory");
                    exit(1);
                }

                if output.is_none() || output.as_ref().unwrap().is_file() {
                    pretty_error("Batch output must be a directory");
                    exit(1);
                }

                for entry in fs::read_dir(input).unwrap() {
                    let path = entry.unwrap().path();
                    if !path.is_file() {
                        continue;
                    }

                    let filename = PathBuf::from(path.file_name().unwrap());
                    let filename = filename.with_extension("png");

                    let mut final_path = output.clone().unwrap();
                    final_path.push(filename);

                    let cz = match cz::open(&path) {
                        Ok(cz) => cz,
                        Err(_) => {
                            pretty_error(&format!(
                                "Could not open input as a CZ file: {}\n",
                                path.into_os_string().to_str().unwrap()
                            ));
                            continue;
                        }
                    };

                    image::save_buffer_with_format(
                        final_path,
                        cz.as_raw(),
                        cz.header().width() as u32,
                        cz.header().height() as u32,
                        ColorType::Rgba8,
                        image::ImageFormat::Png,
                    )
                    .unwrap();
                }
            } else {
                let cz = cz::open(input).unwrap();

                if let Some(output) = output {
                    image::save_buffer_with_format(
                        output,
                        cz.as_raw(),
                        cz.header().width() as u32,
                        cz.header().height() as u32,
                        ColorType::Rgba8,
                        image::ImageFormat::Png,
                    )
                    .unwrap();
                } else {
                    let file_stem = PathBuf::from(input.file_name().unwrap());
                    image::save_buffer_with_format(
                        file_stem.with_extension("png"),
                        cz.as_raw(),
                        cz.header().width() as u32,
                        cz.header().height() as u32,
                        ColorType::Rgba8,
                        image::ImageFormat::Png,
                    )
                    .unwrap();
                }
            }
        }
        Commands::Replace {
            batch,
            input,
            replacement,
            output,
            version,
            depth,
            no_auto_bounds,
            crop,
            bounds,
        } => {
            if !input.exists() {
                pretty_error("The input file does not exist");
                exit(1);
            }

            if !replacement.exists() {
                pretty_error("The replacement file does not exist");
                exit(1);
            }

            let crop = if let Some(c) = crop {
                let Ok(o) = parse_dimensions(c) else {
                    pretty_error(&format!("\"{}\" is not a valid dimension", c));
                    exit(1);
                };

                Some(o)
            } else {
                None
            };

            let bounds = if let Some(b) = bounds {
                let Ok(o) = parse_dimensions(b) else {
                    pretty_error(&format!("\"{}\" is not a valid dimension", b));
                    exit(1);
                };

                Some(o)
            } else {
                None
            };

            // If it's a batch replacement, we want directories to search
            if *batch {
                if !input.is_dir() {
                    pretty_error("Batch input must be a directory");
                    exit(1);
                }

                if !replacement.is_dir() {
                    pretty_error("Batch replacement must be a directory");
                    exit(1);
                }

                if !output.is_dir() {
                    pretty_error("Batch output location must be a directory");
                    exit(1);
                }

                // Replace all the files within the directory and print errors for them
                for entry in fs::read_dir(input).unwrap() {
                    let path = entry.unwrap().path();
                    if !path.is_file() {
                        continue;
                    }

                    // Set the replacement image to the same name as the original file
                    let mut final_replacement = replacement.to_path_buf();
                    final_replacement
                        .push(PathBuf::from(path.file_name().unwrap()).with_extension("png"));

                    // Set the replacement image to the same name as the original file
                    let mut final_output = output.to_path_buf();
                    final_output.push(path.file_name().unwrap());

                    if let Err(error) =
                        replace_cz(
                            &path,
                            &final_output,
                            &final_replacement,
                            version,
                            depth,
                            CropBoundReplacement::default()
                        )
                    {
                        Error::raw(
                            ErrorKind::ValueValidation,
                            format!("{:?} - {}\n", path, error),
                        )
                        .print()
                        .unwrap();
                    }
                }
            } else {
                if !input.is_file() {
                    pretty_error("Input must be a file");
                    exit(1);
                }

                if !replacement.is_file() {
                    pretty_error("Replacement must be a file");
                    exit(1);
                }

                // Replace the input file with the new image
                replace_cz(
                    &input,
                    &output,
                    &replacement,
                    version,
                    depth,
                    CropBoundReplacement {
                        auto_replace: !no_auto_bounds,
                        crop,
                        bounds,
                    }
                ).unwrap();
            }
        }
        Commands::Encode {
            input,
            output,
            version,
            depth,
        } => {
            if !input.exists() {
                pretty_error("The original file provided does not exist");
                exit(1);
            }

            let version = if let Some(v) = version {
                match CzVersion::try_from(*v) {
                    Ok(v) => v,
                    Err(_) => {
                        pretty_error(&format!(
                            "Invalid CZ version {}; must be 0, 1, 2, 3, or 4",
                            v
                        ));
                        exit(1);
                    }
                }
            } else if output
                .extension()
                .is_some_and(|e| e.to_ascii_lowercase().to_string_lossy().starts_with("cz"))
            {
                let ext_string = output.extension().unwrap().to_string_lossy();
                let last_char = ext_string.chars().last().unwrap();
                match CzVersion::try_from(last_char) {
                    Ok(v) => v,
                    Err(e) => {
                        pretty_error(&format!("Invalid CZ type: {}", e));
                        exit(1);
                    }
                }
            } else {
                pretty_error("CZ version not specified or not parseable from file path");
                exit(1);
            };

            let image = match image::open(input) {
                Ok(i) => i,
                Err(e) => {
                    pretty_error(&format!("Could not open input file: {e}"));
                    exit(1);
                }
            };

            let image_depth = image.color();

            let mut cz = CzFile::from_raw(
                version,
                image.width() as u16,
                image.height() as u16,
                image.to_rgba8().into_vec(),
            );
            if let Some(d) = *depth {
                if !(d == 8 || d == 24 || d == 32) {
                    pretty_error(&format!(
                        "The color depth provided is not valid. Choose from: {}",
                        "8, 24, or 32".bright_magenta()
                    ));
                    exit(1);
                }
                cz.header_mut().set_depth(d);
            } else {
                cz.header_mut().set_depth(image_depth.bits_per_pixel());
            }
            cz.save_as_cz(output).expect("Saving CZ file failed");
        }
    }
}

#[derive(Default, Clone, Copy)]
struct CropBoundReplacement {
    pub auto_replace: bool,
    pub crop: Option<(u16, u16)>,
    pub bounds: Option<(u16, u16)>,
}

/// Replace a CZ file with the bitmap of a PNG file
fn replace_cz<P: ?Sized + AsRef<Path>>(
    input_path: &P,
    output_path: &P,
    replacement_path: &P,
    version: &Option<u8>,
    depth: &Option<u16>,
    crop_bound_replacement: CropBoundReplacement,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = input_path.as_ref();
    if !path.is_file() {
        return Err("Input path is not a file".into());
    }

    if !replacement_path.as_ref().exists() || !replacement_path.as_ref().is_file() {
        return Err("Replacement path does not exist or is not a file".into());
    }

    // Open the replacement image and convert it to RGBA8
    let repl_img = image::open(replacement_path)?.to_rgba8();

    // Open the original CZ file
    let mut cz = cz::open(&path)?;

    let crop_equal = cz.extended_header().is_some_and(|e| {
        e.crop_width == cz.header().width() && e.crop_height == cz.header().height()
    });

    let bounds_equal = cz.extended_header().is_some_and(|e| {
        e.bounds_width == cz.header().width() && e.bounds_height == cz.header().height()
    });

    if !crop_equal {
        println!("Crop will not be auto-modified for \"{}\"", path.to_string_lossy());
    }

    if !bounds_equal {
        println!("Bounds will not be auto-modified for \"{}\"", path.to_string_lossy());
    }

    // Set CZ header parameters and the new bitmap
    cz.header_mut().set_width(repl_img.width() as u16);
    cz.header_mut().set_height(repl_img.height() as u16);
    cz.set_bitmap(repl_img.into_raw());
    cz.clear_palette();

    // If the extended header exists and the width and height are the same
    // as the crop width and crop height, fix them to be the same.
    let header_ref = *cz.header();
    if let Some(ext) = cz.extended_header_mut() {
        if crop_equal && crop_bound_replacement.auto_replace {
            ext.crop_width = header_ref.width();
            ext.crop_height = header_ref.height();
        }

        if bounds_equal && crop_bound_replacement.auto_replace {
            ext.bounds_width = header_ref.width();
            ext.bounds_height = header_ref.height();
        }

        // Replace crop
        if let Some(c) = crop_bound_replacement.crop {
            ext.crop_width = c.0;
            ext.crop_height = c.1;
        }

        // Replace bounds
        if let Some(b) = crop_bound_replacement.bounds {
            ext.bounds_width = b.0;
            ext.bounds_height = b.1;
        }
    }

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

fn parse_dimensions(dim: &str) -> Result<(u16, u16), ParseIntError> {
    let mut out = [0, 0];
    for (i, dimension) in dim.split('x').enumerate().take(2) {
        out[i] = dimension.parse::<u16>()?;
    }

    Ok((out[0], out[1]))
}

fn pretty_error(message: &str) {
    eprintln!("{}: {}", "Error".red().italic(), message);
}
