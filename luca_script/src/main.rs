use std::{fs::File, io::{Read, Seek}};
use byteorder::{ReadBytesExt, LE};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

fn main() {
    let mut script = File::open("LOOPERS_scenario_01").unwrap();

    while let Ok(byte) = script.read_u8() {
        if let Some(opcode) = Opcode::from_u8(byte) {
            println!(
                "{:X?}: {:#04X?} ({:?})",
                script.stream_position().unwrap() - 1,
                opcode.to_u8().unwrap(),
                opcode
            );

            match opcode {
                Opcode::MESSAGE => {
                    let variables = script.read_u8().unwrap();
                    match variables {
                        1 => continue,
                        4 => {
                            script.read_u32::<LE>().unwrap();
                            continue;
                        }
                        3 => (),
                        _ => unimplemented!(),
                    }
                    let message = Message {
                        variables,
                        unknown1: Some(script.read_u16::<LE>().unwrap()),
                        unknown2: Some(script.read_u16::<LE>().unwrap()),
                        index: Some(script.read_u16::<LE>().unwrap()),
                        messages: Some((0..3).map(|_| ScriptString::read(&mut script)).collect()),
                    };
                    message.messages.unwrap().iter().for_each(|m| println!("{}", m.to_string()));
                    println!("-----");
                },
                Opcode::IMAGELOAD => {
                    let mode = script.read_u8().unwrap();
                    match mode {
                        0 => script.read_u16::<LE>().unwrap(),
                        _ => {
                            script.read_u16::<LE>().unwrap();
                            script.read_u16::<LE>().unwrap()
                        },
                    };
                    let image_id = script.read_u16::<LE>().unwrap();
                    println!("Image ID: {image_id}\n-----");
                },
                /*
                Opcode::SELECT => {
                    let var_id = script.read_u16::<LE>().unwrap();
                    script.read_u16::<LE>().unwrap();
                    script.read_u16::<LE>().unwrap();
                    script.read_u16::<LE>().unwrap();
                    let msg_str = script.read_u16::<LE>().unwrap();
                    script.read_u16::<LE>().unwrap();
                    script.read_u16::<LE>().unwrap();
                    script.read_u16::<LE>().unwrap();
                    println!("{var_id} & {msg_str}\n-----");
                },
                */
                Opcode::JUMP => {
                    script.read_u16::<LE>().unwrap();
                }
                _ => (),
            }
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    unknown1: Option<u16>,
    unknown2: Option<u16>,
    index: Option<u16>,

    messages: Option<Vec<ScriptString>>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            variables: 1,
            unknown1: None,
            unknown2: None,
            index: None,
            messages: None
        }
    }
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
    fn read<R: Read + ReadBytesExt>(input: &mut R) -> Self {
        let length = input.read_i16::<LE>().unwrap();
        let (mut buffer, format) = if length < 0 {
            // If the length is negative, then the length is the exact length in
            // bytes??
            (vec![0u8; length.abs() as usize + 1], StringFormat::UTF8)
        } else {
            // Otherwise double the length
            (vec![0u8; (length as usize + 1) * 2], StringFormat::UTF16)
        };

        input.read_exact(&mut buffer).unwrap();

        Self {
            length,
            format,
            buffer,
        }
    }
}

impl ToString for ScriptString {
    fn to_string(&self) -> String {
        match self.format {
            StringFormat::UTF8 => String::from_utf8_lossy(&self.buffer).to_string(),
            StringFormat::UTF16 => {
                String::from_utf16_lossy(
                    &self.buffer
                        .chunks(2)
                        .map(|c| u16::from_le_bytes(c.try_into().unwrap())).collect::<Vec<u16>>()).to_owned()
            },
            StringFormat::ASCII => String::from_utf8_lossy(&self.buffer).to_string(),
            _ => unimplemented!(),
        }
    }
}
