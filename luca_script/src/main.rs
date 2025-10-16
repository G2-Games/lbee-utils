mod utils;

use std::{fs, io::{Cursor, Read, Write}, sync::LazyLock};

use byteorder::{WriteBytesExt, ReadBytesExt, LE};
use serde::{Deserialize, Serialize};
use utils::Encoding;

static OPCODES: LazyLock<Vec<String>> = LazyLock::new(|| fs::read_to_string("LBEE_opcodes")
    .unwrap()
    .split("\n")
    .map(|s| s.to_owned())
    .collect()
);

fn main() {
    let mut script_file = fs::File::open("SEEN0513").unwrap();
    let script_len = script_file.metadata().unwrap().len();
    let script = decode_script(&mut script_file, script_len);

    /*
    for c in script.opcodes {
        print!("{:>5}", c.position);
        print!("{:>12}: ", c.string);
        if let Some(o) = c.opcode_specifics {
            print!("{}", serde_json::ser::to_string(&o).unwrap());
        } else if let Some(r) = c.fixed_param {
            print!("{:?}", r);
        }
        println!();
    }
    */

    //println!("{}", serde_json::ser::to_string_pretty(&script).unwrap());

    let mut rewrite_script = fs::File::create("SEEN0513-rewritten").unwrap();
    write_script(&mut rewrite_script, script).unwrap();
    println!("Wrote out successfully");
}

fn decode_script<S: Read>(script_stream: &mut S, length: u64) -> Script {
    let mut opcodes = Vec::new();
    let mut offset = 0;
    let mut i = 0;
    let mut pos = 0;
    while offset < length as usize {
        // Read all base info
        let length = script_stream.read_u16::<LE>().unwrap() as usize;
        let number = script_stream.read_u8().unwrap();
        let flag = script_stream.read_u8().unwrap();
        let string = OPCODES[number as usize].clone();

        offset += 4;

        let raw_len = length - 4;
        let mut raw_bytes = vec![0u8; raw_len];
        script_stream.read_exact(&mut raw_bytes).unwrap();
        offset += raw_len;

        // Read extra align byte if alignment needed
        if length.is_multiple_of(2) {
            offset += 1;
            Some(script_stream.read_u8().unwrap())
        } else {
            None
        };

        let mut fixed_param = Vec::new();
        let param_bytes = match flag {
            0 => raw_bytes.clone(),
            f if f < 2 => {
                fixed_param = vec![
                    u16::from_le_bytes(raw_bytes[..2].try_into().unwrap()),
                ];
                raw_bytes[2..].to_vec()
            }
            _ => {
                fixed_param = vec![
                    u16::from_le_bytes(raw_bytes[..2].try_into().unwrap()),
                    u16::from_le_bytes(raw_bytes[2..4].try_into().unwrap()),
                ];
                raw_bytes[4..].to_vec()
            }
        };

        opcodes.push(Opcode {
            index: i,
            position: pos,
            length,
            opcode_number: number,
            string: string.clone(),
            flag,
            fixed_param,
            opcode_specifics: SpecificOpcode::decode(&string, &param_bytes),
            param_bytes,
        });

        // Break if END opcode reached
        if &string == "END" {
            break;
        }

        pos += (length + 1) & !1;
        i += 1;
    }

    Script {
        code_count: opcodes.len(),
        opcodes,
    }
}

fn write_script<W: Write>(script_output: &mut W, script: Script) -> Result<(), ()> {
    let mut position = 0;
    for opcode in script.opcodes {
        let mut total = 0;
        script_output.write_u16::<LE>(opcode.length as u16).unwrap();
        script_output.write_u8(OPCODES.iter().position(|l| *l == opcode.string).unwrap() as u8).unwrap();
        script_output.write_u8(opcode.flag).unwrap();
        total += 4;

        for p in opcode.fixed_param {
            script_output.write_u16::<LE>(p).unwrap();
            total += 2;
        }

        script_output.write_all(&opcode.param_bytes).unwrap();
        total += opcode.param_bytes.len();
        if (position + total) % 2 != 0 {
            script_output.write_u8(0).unwrap();
            total += 1;
        }
        position += total;
    }

    Ok(())
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
struct Script {
    opcodes: Vec<Opcode>,
    code_count: usize,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
struct Opcode {
    index: usize,
    position: usize,
    length: usize,
    opcode_number: u8,
    string: String,

    flag: u8,
    fixed_param: Vec<u16>,
    param_bytes: Vec<u8>,
    opcode_specifics: Option<SpecificOpcode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
enum SpecificOpcode {
    Message {
        voice_id: u16,
        messages: Vec<String>,
        end: Vec<u8>,
    },
    Add {
        var1: u16,
        expr: String,
    },
    EquN {
        var1: u16,
        value: Option<u16>, //?
    },
    Select {
        var_id: u16,
        var0: u16,
        var1: u16,
        var2: u16,
        messages: Vec<String>,
        var3: u16,
        var4: u16,
        var5: u16,
    },
    _Battle,
    Task {
        task_type: u16,
        var1: Option<u16>,
        var2: Option<u16>,
        var3: Option<u16>,
        var4: Option<u16>,
        message_1: Option<Vec<String>>,
        message_2: Option<Vec<String>>,
        raw_args: Option<Vec<u8>>,
    },
    SayAVoiceText {
        voice_id: u16,
        messages: Vec<String>,
    },
    VarStrSet {
        varstr_id: u16,
        varstr_str: String,
    },
    GoTo {
        jump_pos: u32,
    },
    GoSub {
        arg1: u16,
        jump_pos: u32,
        end: Vec<u8>,
    },
    Jump {
        filename: String,
        jump_pos: Option<u32>,
    },
    FarCall {
        index: u16,
        filename: String,
        jump_pos: u32,
        end: Vec<u8>,
    },
    IfN {
        condition: String,
        jump_pos: u32,
    },
    IfY {
        condition: String,
        jump_pos: u32,
    },
    Random {
        var1: u16,
        rnd_from: String,
        rnd_to: String,
    },
    ImageLoad {
        mode: u16,
        image_id: u16,
        var1: Option<u16>,
        pos_x: Option<u16>,
        pos_y: Option<u16>,
        end: Vec<u8>,
    },
    Bgm {
        bgm_id: u32,
        arg2: Option<u16>,
    },
    Unknown(Vec<u8>),
}

impl SpecificOpcode {
    pub fn decode(opcode_str: &str, param_bytes: &[u8]) -> Option<Self> {
        if param_bytes.is_empty() {
            return None
        }

        let mut cursor_param = Cursor::new(param_bytes);

        Some(match opcode_str {
            "MESSAGE" => Self::decode_message(&mut cursor_param),
            "SAYAVOICETEXT" => Self::decode_sayavoicetext(&mut cursor_param),
            "SELECT" => Self::decode_select(&mut cursor_param),
            "TASK" => Self::decode_task(&mut cursor_param),

            "ADD" => Self::decode_add(&mut cursor_param),
            "EQUN" => Self::decode_equn(&mut cursor_param),

            "RANDOM" => Self::decode_random(&mut cursor_param),
            "IFY" => Self::decode_ifn_ify(&mut cursor_param, false),
            "IFN" => Self::decode_ifn_ify(&mut cursor_param, true),
            "JUMP" => Self::decode_jump(&mut cursor_param),
            "GOTO" => Self::decode_goto(&mut cursor_param),
            "GOSUB" => Self::decode_gosub(&mut cursor_param),
            "FARCALL" => Self::decode_farcall(&mut cursor_param),
            "VARSTR_SET" => Self::decode_varstr_set(&mut cursor_param),

            "IMAGELOAD" => Self::decode_imageload(&mut cursor_param),
            "BGM" => Self::decode_bgm(&mut cursor_param),
            _ => Self::Unknown(param_bytes.to_vec())
        })
    }

    fn decode_message<R: Read>(param_bytes: &mut R) -> Self {
        let voice_id = param_bytes.read_u16::<LE>().unwrap();

        // TODO: This will need to change per-game based on the number of
        // languages and their encodings
        let mut messages = Vec::new();
        for _ in 0..2 {
            let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
            messages.push(string);
        }

        let mut end = Vec::new();
        param_bytes.read_to_end(&mut end).unwrap();

        Self::Message {
            voice_id,
            messages,
            end,
        }
    }

    fn decode_add<R: Read>(param_bytes: &mut R) -> Self {
        let var1 = param_bytes.read_u16::<LE>().unwrap();
        let expr = utils::decode_string_v1(param_bytes, Encoding::ShiftJIS).unwrap();

        Self::Add { var1, expr }
    }

    fn decode_equn<R: Read>(param_bytes: &mut R) -> Self {
        let var1 = param_bytes.read_u16::<LE>().unwrap();
        let value = param_bytes.read_u16::<LE>().ok();

        Self::EquN { var1, value }
    }

    fn decode_select<R: Read>(param_bytes: &mut R) -> Self {
        let var_id = param_bytes.read_u16::<LE>().unwrap();
        let var0 = param_bytes.read_u16::<LE>().unwrap();
        let var1 = param_bytes.read_u16::<LE>().unwrap();
        let var2 = param_bytes.read_u16::<LE>().unwrap();

        // TODO: This will need to change per-game based on the number of
        // languages and their encodings
        let mut messages = Vec::new();
        for _ in 0..2 {
            let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
            messages.push(string);
        }

        let var3 = param_bytes.read_u16::<LE>().unwrap();
        let var4 = param_bytes.read_u16::<LE>().unwrap();
        let var5 = param_bytes.read_u16::<LE>().unwrap();

        Self::Select {
            var_id,
            var0,
            var1,
            var2,
            messages,
            var3,
            var4,
            var5
        }
    }

    fn decode_random<R: Read>(param_bytes: &mut R) -> Self {
        let var1 = param_bytes.read_u16::<LE>().unwrap();
        let rnd_from = utils::decode_string_v1(param_bytes, Encoding::ShiftJIS).unwrap();
        let rnd_to = utils::decode_string_v1(param_bytes, Encoding::ShiftJIS).unwrap();

        Self::Random { var1, rnd_from, rnd_to }
    }

    fn decode_ifn_ify<R: Read>(param_bytes: &mut R, ifn: bool) -> Self {
        let condition = utils::decode_string_v1(param_bytes, Encoding::ShiftJIS).unwrap();
        let jump_pos = param_bytes.read_u32::<LE>().unwrap();

        if ifn {
            Self::IfN { condition, jump_pos }
        } else {
            Self::IfY { condition, jump_pos }
        }
    }

    fn decode_jump<R: Read>(param_bytes: &mut R) -> Self {
        let filename = utils::decode_string_v1(param_bytes, Encoding::ShiftJIS).unwrap();

        let jump_pos = param_bytes.read_u32::<LE>().ok();

        Self::Jump { filename, jump_pos }
    }

    fn decode_imageload<R: Read>(param_bytes: &mut R) -> Self {
        let mode = param_bytes.read_u16::<LE>().unwrap();
        let image_id = param_bytes.read_u16::<LE>().unwrap();

        // These will only be read if there is anything to be read
        let var1 = param_bytes.read_u16::<LE>().ok();
        let pos_x = param_bytes.read_u16::<LE>().ok();
        let pos_y = param_bytes.read_u16::<LE>().ok();

        let mut end = Vec::new();
        param_bytes.read_to_end(&mut end).unwrap();

        Self::ImageLoad {
            mode,
            image_id,
            var1,
            pos_x,
            pos_y,
            end,
        }
    }

    fn decode_goto<R: Read>(param_bytes: &mut R) -> Self {
        let jump_pos = param_bytes.read_u32::<LE>().unwrap();

        Self::GoTo { jump_pos }
    }

    fn decode_gosub<R: Read>(param_bytes: &mut R) -> Self {
        let arg1 = param_bytes.read_u16::<LE>().unwrap();
        let jump_pos = param_bytes.read_u32::<LE>().unwrap();

        let mut end = Vec::new();
        param_bytes.read_to_end(&mut end).unwrap();

        Self::GoSub {
            arg1,
            jump_pos,
            end,
        }
    }

    fn decode_varstr_set<R: Read>(param_bytes: &mut R) -> Self {
        let varstr_id = param_bytes.read_u16::<LE>().unwrap();
        let varstr_str = utils::decode_string_v1(param_bytes, Encoding::ShiftJIS).unwrap();

        Self::VarStrSet { varstr_id, varstr_str }
    }

    fn decode_farcall<R: Read>(param_bytes: &mut R) -> Self {
        let index = param_bytes.read_u16::<LE>().unwrap();
        let filename = utils::decode_string_v1(param_bytes, Encoding::ShiftJIS).unwrap();
        let jump_pos = param_bytes.read_u32::<LE>().unwrap();

        let mut end = Vec::new();
        param_bytes.read_to_end(&mut end).unwrap();

        Self::FarCall {
            index,
            filename,
            jump_pos,
            end,
        }
    }

    fn decode_sayavoicetext<R: Read>(param_bytes: &mut R) -> Self {
        let voice_id = param_bytes.read_u16::<LE>().unwrap();

        // TODO: This will need to change per-game based on the number of
        // languages and their encodings
        let mut messages = Vec::new();
        for _ in 0..2 {
            let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
            messages.push(string);
        }

        Self::SayAVoiceText {
            voice_id,
            messages,
        }
    }

    fn decode_bgm<R: Read>(param_bytes: &mut R) -> Self {
        // TODO: invesigate the accuracy of this
        let bgm_id = param_bytes.read_u32::<LE>().unwrap();

        let arg2 = param_bytes.read_u16::<LE>().ok();

        Self::Bgm {
            bgm_id,
            arg2,
        }
    }

    fn decode_task<R: Read>(param_bytes: &mut R) -> Self {
        let task_type = param_bytes.read_u16::<LE>().unwrap();

        let mut var1 = None;
        let mut var2 = None;
        let mut var3 = None;
        let mut var4 = None;
        let mut message_1 = None;
        let mut message_2 = None;
        let raw_args: Option<Vec<u8>> = None;

        if false {
            return Self::Task { task_type, var1, var2, var3, var4, message_1, message_2, raw_args };
        }

        match task_type {
            4 => {
                let var1 = param_bytes.read_u16::<LE>().ok();

                if false {
                    return Self::Task { task_type, var1, var2, var3, var4, message_1, message_2, raw_args };
                }

                if [0, 4, 5].contains(&var1.unwrap()) {
                    var2 = param_bytes.read_u16::<LE>().ok();

                    let mut messages = Vec::new();
                    for _ in 0..2 {
                        let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
                        messages.push(string);
                    }
                    message_1 = Some(messages);
                } else if var1.unwrap() == 1 {
                    var2 = param_bytes.read_u16::<LE>().ok();
                    var3 = param_bytes.read_u16::<LE>().ok();
                    var4 = param_bytes.read_u16::<LE>().ok();

                    // Get first set of messages
                    let mut messages = Vec::new();
                    for _ in 0..2 {
                        let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
                        messages.push(string);
                    }
                    message_1 = Some(messages);

                    // Get second set of messages
                    let mut messages = Vec::new();
                    for _ in 0..2 {
                        let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
                        messages.push(string);
                    }
                    message_2 = Some(messages);
                } else if var1.unwrap() == 6 {
                    var2 = param_bytes.read_u16::<LE>().ok();
                    var3 = param_bytes.read_u16::<LE>().ok();

                    let mut messages = Vec::new();
                    for _ in 0..2 {
                        let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
                        messages.push(string);
                    }
                    message_1 = Some(messages);
                } else {
                    return Self::Task { task_type, var1, var2, var3, var4, message_1, message_2, raw_args };
                }
            }
            54 => {
                let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
                message_1 = Some(vec![string]);
            }
            69 => {
                var1 = param_bytes.read_u16::<LE>().ok();

                // Get first set of messages
                let mut messages = Vec::new();
                for _ in 0..2 {
                    let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
                    messages.push(string);
                }
                message_1 = Some(messages);

                // Get second set of messages
                let mut messages = Vec::new();
                for _ in 0..2 {
                    let string = utils::decode_string_v1(param_bytes, Encoding::UTF16).unwrap();
                    messages.push(string);
                }
                message_2 = Some(messages);
            }
            _ => return Self::Task {
                task_type,
                var1,
                var2,
                var3,
                var4,
                message_1,
                message_2,
                raw_args,
            }
        }

        Self::Task {
            task_type,
            var1,
            var2,
            var3,
            var4,
            message_1,
            message_2,
            raw_args,
        }
    }
}
