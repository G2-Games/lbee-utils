use std::{fs::File, io::{self, BufRead, BufReader, Read, Seek}};
use byteorder::{ReadBytesExt, LE};
use encoding_rs::SHIFT_JIS;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use safe_transmute::{transmute_many, SingleManyGuard};

fn main() {
    let mut script = BufReader::new(File::open("LOOPERS_scenario_01").unwrap());

    let mut unknown_count = 0;
    while let Ok(byte) = script.read_u8() {
        if let Some(opcode) = Opcode::from_u8(byte) {
            println!(
                "{:X?}: {:#04X?} ({:?})",
                script.stream_position().unwrap() - 1,
                opcode.to_u8().unwrap(),
                opcode
            );

            parse_opcode(opcode, &mut script).unwrap();
        } else {
            println!(
                "{:X?}: {:#04X?} (\x1b[0;41m{:?}\x1b[0m)",
                script.stream_position().unwrap() - 1,
                byte,
                Opcode::UNKNOWN,
            );
            unknown_count += 1;
        }
    }

    println!("Encountered {unknown_count} unknown opcodes, it is very likely these are incorrect");
}

fn parse_opcode<R: Read + BufRead>(opcode: Opcode, mut input: R) -> Result<(), io::Error> {
    match opcode {
        Opcode::MESSAGE => {
            let variables = input.read_u8().unwrap();
            match variables {
                // Empty message!?
                1 => return Ok(()),
                3 => (),
                // Unknown data
                4 => {
                    let mut buf = vec![0; 4];
                    input.read_exact(&mut buf).unwrap();
                    //println!("{:0X?}", buf);
                    return Ok(());
                }
                n => unimplemented!("{n}"),
            }
            let message = Message {
                variables,
                unknown1: input.read_u16::<LE>().unwrap(),
                unknown2: input.read_u16::<LE>().unwrap(),
                index: input.read_u16::<LE>().unwrap(),
                strings: (0..3).map(|_| ScriptString::read(&mut input)).collect::<Result<Vec<_>, _>>()?,
            };
            message.strings.iter().enumerate().for_each(|m| println!("{}: {}, {}", m.0, m.1.length, m.1.to_string()));
            println!("-----");
        },
        Opcode::IMAGELOAD => {
            let mode = input.read_u8()?;
            println!("Mode: {mode}");
            if mode == 0 {
                println!("Unknown: {}", input.read_u16::<LE>()?);
            } else {
                println!("Unknown: {}", input.read_u16::<LE>()?);
                println!("Unknown: {}", input.read_u16::<LE>()?);
            }

            let image_id = input.read_u32::<LE>()?;
            println!("Image ID: {image_id}");
            println!("-----");
        },
        Opcode::BGM => {
            input.read_u8().unwrap(); // ?
            input.read_u16::<LE>().unwrap(); // ?
            let bgm_id = input.read_u32::<LE>().unwrap();

            println!("ID: \x1b[0;45m{bgm_id}\x1b[0m");
            println!("-----");
        }
        Opcode::JUMP => {
            input.read_u8().unwrap();
            input.read_u16::<LE>().unwrap();
        }
        Opcode::VARSTR => {
            let id = input.read_u16::<LE>().unwrap();
            println!("ID: {id}");
            println!("-----");
        }
        Opcode::VARSTR_SET => {
            let varstr = VarStrSet {
                opcode,
                variant: input.read_u8()?,
                unknown1: input.read_u16::<LE>()?,
                unknown2: input.read_u16::<LE>()?,
                string: ScriptString::read(&mut input)?,
                unknown3: input.read_u16::<LE>()?,
            };
            //println!("{}", varstr.string.to_string());
            println!("{:02X?}", varstr.variant);
            println!("{:02X?}", varstr.unknown1);
            println!("ID: {}", varstr.unknown2);
            println!("{:02X?}", varstr.unknown3);
            println!("-----");
        }
        _ => (),
    }

    Ok(())
}

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[derive(FromPrimitive, ToPrimitive)]
enum Opcode {
    EQU,
    EQUN,
    EQUV,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    AND,
    OR,
    RANDOM,
    VARSTR,
    VARSTR_ADD,
    SET,
    FLAGCLR,
    GOTO,
    ONGOTO,
    GOSUB,
    IFY,
    IFN,
    RETURN,
    JUMP,
    FARCALL,
    FARRETURN,
    JUMPPOINT,
    END,
    STARTUP_BEGIN,
    STARTUP_END,
    TASKSCRVAR,
    VARSTR_SET,
    VARSTR_ALLOC,
    ARFLAGSET,
    COLORBG_SET,
    SPLINE_SET,
    SHAKELIST_SET,
    SCISSOR_TRIANGLELIST_SET,
    MESSAGE,
    MESSAGE_CLEAR,
    MESSAGE_WAIT,
    MESSAGE_AR_SET,
    SELECT,
    CLOSE_WINDOW,
    FADE_WINDOW,
    LOG_BEGIN,
    LOG_PAUSE,
    LOG_END,
    VOICE,
    VOICE_STOP,
    WAIT_COUNT,
    WAIT_TIME,
    WAIT_TEXTFEED,
    FFSTOP,
    INIT,
    STOP,
    IMAGELOAD,
    IMAGEUPDATE,
    ARC,
    MOVE,
    MOVE_SKIP,
    ROT,
    PEND,
    FADE,
    SCALE,
    SHAKE,
    SHAKELIST,
    BASE,
    MCMOVE,
    MCARC,
    MCROT,
    MCSHAKE,
    MCFADE,
    WAIT,
    WAIT_BSKIP,
    DRAW,
    WIPE,
    FRAMEON,
    FRAMEOFF,
    FW,
    SCISSOR,
    DELAY,
    RASTER,
    TONE,
    SCALECOSSIN,
    BMODE,
    SIZE,
    SPLINE,
    DISP,
    MASK,
    FACE,
    SEPIA,
    SEPIA_COLOR,
    CUSTOMMOVE,
    SWAP,
    ADDCOLOR,
    SUBCOLOR,
    SATURATION,
    CONTRAST,
    PRIORITY,
    UVWH,
    EVSCROLL,
    COLORLEVEL,
    NEGA,
    TONECURVE,
    SKIP_SCOPE_BEGIN,
    SKIP_SCOPE_END,
    QUAKE,
    BGM,
    BGM_WAIT_START,
    BGM_WAIT_FADE,
    BGM_PUSH,
    BGM_POP,
    SE,
    SE_STOP,
    SE_WAIT,
    SE_WAIT_COUNT,
    SE_WAIT_FADE,
    VOLUME,
    MOVIE,
    SETCGFLAG,
    EX,
    TROPHY,
    SETBGMFLAG,
    TASK,
    PRINTF,
    DIALOG,
    VIB_PLAY,
    VIB_FILE,
    VIB_STOP,
    CHAR_VOLUME,
    SCENE_REPLAY_END,
    SAVE_THUMBNAIL,
    MANPU,
    SCENARIO,
    SCRIPTLINE,
    COUNTER_SET,
    COUNTER_WAIT,
    UNKNOWN
}

#[derive(Debug, PartialEq, Eq)]
struct Message {
    variables: u8,
    unknown1: u16,
    unknown2: u16,
    index: u16,

    strings: Vec<ScriptString>,
}

#[derive(Debug, PartialEq, Eq)]
struct VarStrSet {
    opcode: Opcode,
    variant: u8,
    unknown1: u16,
    unknown2: u16,
    string: ScriptString,
    unknown3: u16,
}

#[derive(Debug, PartialEq, Eq)]
struct ScriptString {
    length: i16,
    format: StringFormat,
    buffer: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
enum StringFormat {
    UTF8,
    UTF16,
    ShiftJIS,
    ASCII,
}

impl ScriptString {
    fn read<R: Read + BufRead>(input: &mut R) -> Result<Self, io::Error> {
        let length = input.read_i16::<LE>()?;

        // Catch very long strings, these should be investigated (they're probably broken)
        assert!(length < 2_000);

        let (mut buffer, format) = if length < 0 {
            // If the length is negative, then the length is the exact length in
            // bytes of the absolute value?!?
            (vec![0u8; length.unsigned_abs() as usize], StringFormat::UTF8)
        } else {
            // Otherwise double the length
            (vec![0u8; (length as usize) * 2], StringFormat::UTF16)
        };

        // Read the string into the buffer
        input.read_exact(&mut buffer)?;

        // Ensure the string is null terminated
        let string_end = match format {
            StringFormat::UTF16 => input.read_u16::<LE>()?,
            _ => input.read_u8()? as u16,
        };

        assert!(!(string_end != 0), "String not null terminated!");

        Ok(Self {
            length,
            format,
            buffer,
        })
    }
}

impl ToString for ScriptString {
    fn to_string(&self) -> String {
        match self.format {
            StringFormat::UTF8 | StringFormat::ASCII => String::from_utf8_lossy(&self.buffer).to_string(),
            StringFormat::UTF16 => String::from_utf16_lossy(transmute_many::<u16, SingleManyGuard>(&self.buffer).unwrap()),
            StringFormat::ShiftJIS => SHIFT_JIS.decode(&self.buffer).0.to_string(),
        }
    }
}
