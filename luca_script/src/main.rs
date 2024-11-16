mod utils;

use std::{cell::LazyCell, fs::{self, File}, io::Read, path::PathBuf};

use byteorder_lite::{ReadBytesExt, LE};
use utils::Encoding;

const OPCODES: LazyCell<Vec<String>> = LazyCell::new(|| fs::read_to_string("LBEE_opcodes")
    .unwrap()
    .split("\n")
    .map(|s| s.to_owned())
    .collect()
);

fn main() {
    let file_path = PathBuf::from("SEEN0513");

    let mut script = File::open(&file_path).unwrap();

    println!("Start parsing script");
    let script = parse_script(
        &mut script,
        file_path.file_name().unwrap().to_str().unwrap()
    );
    println!("Parsing finished");

    for c in script.opcodes {
        let ascii_string = String::from_utf8_lossy(&c.param_bytes);
        //println!("{:>4}: '{:>11}' — {}", c.index, c.string, ascii_string);
        SpecificOpcode::decode(&c.string, c.param_bytes);
    }
}

fn parse_script<S: Read>(script_stream: &mut S, name: &str) -> Script {
    let mut opcodes = Vec::new();
    let mut _offset = 0;
    let mut i = 0;
    let mut pos = 0;
    loop {
        // Read all base info
        let (length, number, flag) = (
            script_stream.read_u16::<LE>().unwrap() as usize,
            script_stream.read_u8().unwrap(),
            script_stream.read_u8().unwrap()
        );
        let string = OPCODES[number as usize].clone();

        _offset += 4;

        let raw_len = length - 4;
        let mut raw_bytes = vec![0u8; raw_len];
        script_stream.read_exact(&mut raw_bytes).unwrap();
        _offset += raw_len;

        // Read extra align byte if alignment needed
        let align = if length % 2 != 0 {
            _offset += 1;
            Some(script_stream.read_u8().unwrap())
        } else {
            None
        };

        let mut fixed_param = None;
        let param_bytes = match flag {
            0 => raw_bytes.clone(),
            f if f < 2 => {
                fixed_param = Some(vec![
                    u16::from_le_bytes(raw_bytes[..2].try_into().unwrap()),
                ]);
                raw_bytes[2..].to_vec()
            }
            _ => {
                fixed_param = Some(vec![
                    u16::from_le_bytes(raw_bytes[..2].try_into().unwrap()),
                    u16::from_le_bytes(raw_bytes[2..4].try_into().unwrap()),
                ]);
                raw_bytes[4..].to_vec()
            }
        };

        opcodes.push(Opcode {
            index: i,
            position: pos,
            length,
            number,
            string: string.clone(),
            flag,
            raw_bytes,
            align,
            fixed_param,
            param_bytes
        });

        // Break if END opcode reached
        if &string == "END" {
            break;
        }

        pos += (length + 1) & !1;
        i += 1;
    }

    Script {
        name: name.to_string(),
        code_count: opcodes.len(),
        opcodes,
    }
}

#[derive(Debug, Clone)]
struct Script {
    name: String,
    opcodes: Vec<Opcode>,
    code_count: usize,
}

#[derive(Debug, Clone)]
struct Opcode {
    index: usize,
    position: usize,
    length: usize,
    number: u8,
    string: String,

    flag: u8,
    raw_bytes: Vec<u8>,
    align: Option<u8>,
    fixed_param: Option<Vec<u16>>,
    param_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
enum SpecificOpcode {
    Message {
        voice_id: u16,
        messages: Vec<String>,
        end: Vec<u8>,
    },
    Select,
    Battle,
    Task,
    SayAVoiceText,
    VarStrSet,
    GoTo,
    GoSub,
    Jump,
    FarCall,
    IFN,
    IFY,
    Random,
    ImageLoad,
    Unknown,
}

impl SpecificOpcode {
    pub fn decode(opcode_str: &str, param_bytes: Vec<u8>) -> Self {
        match opcode_str {
            "MESSAGE" => Self::message(param_bytes),
            _ => Self::Unknown
        }
    }

    fn message(param_bytes: Vec<u8>) -> Self {
        let voice_id = u16::from_le_bytes(param_bytes[0..2].try_into().unwrap());

        let mut messages = Vec::new();
        let mut offset = 2;
        for _ in 0..2 {
            let (o, string) = utils::get_string(&param_bytes, offset, Encoding::UTF16, None).unwrap();
            messages.push(string);
            offset = o;
        }
        dbg!(&messages);

        Self::Message {
            voice_id,
            messages,
            end: param_bytes[offset..].to_vec()
        }
    }
}
