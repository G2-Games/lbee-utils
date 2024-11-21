mod utils;

use std::{ffi::OsString, fs::{self, File}, io::{Read, Write}, path::PathBuf, str::FromStr, sync::LazyLock};

use byteorder_lite::{ReadBytesExt, LE};
use utils::Encoding;

static OPCODES: LazyLock<Vec<String>> = LazyLock::new(|| fs::read_to_string("LBEE_opcodes")
    .unwrap()
    .split("\n")
    .map(|s| s.to_owned())
    .collect()
);

fn main() {
    let scripts_path = PathBuf::from("LBEE_SCRIPT_steam");

    for script in fs::read_dir(scripts_path).unwrap() {
        let script = script.unwrap();
        let filename = script.file_name();
        let filename = filename.to_string_lossy();
        if !script.file_type().unwrap().is_file() {
            continue;
        } else if filename.contains("8500") || filename.contains("8501") {
            continue;
        } else if filename.starts_with("_") {
            continue;
        }

        let mut script_file = File::open(&script.path()).unwrap();

        println!("Start parsing {:?}", script.file_name());
        let script_len = script_file.metadata().unwrap().len();
        let script = parse_script(
            &mut script_file,
            script.file_name().to_str().unwrap(),
            script_len
        );

        let mut out_file = File::create(format!("LBEE_SCRIPT_listing/{}_OPCODES.txt", filename)).unwrap();
        for c in script.opcodes {
            out_file.write_all(format!("{:>5}", c.position).as_bytes()).unwrap();
            out_file.write_all(format!("{:>12}: ", c.string).as_bytes()).unwrap();
            if let Some(o) = c.opcode_specifics {
                if o == SpecificOpcode::Unknown {
                    out_file.write_all(format!("{:02X?}", c.param_bytes).as_bytes()).unwrap();
                } else {
                    out_file.write_all(format!("{:?}", o).as_bytes()).unwrap();
                }
            } else if let Some(r) = c.fixed_param {
                out_file.write_all(format!("{:?}", r).as_bytes()).unwrap();
            }
            out_file.write_all(b"\n").unwrap();
        }
    }
    println!("Done!");

    /*
    for c in script.opcodes {
        print!("{:>5}", c.position);
        print!("{:>12}: ", c.string);
        if let Some(o) = c.opcode_specifics {
            if o == SpecificOpcode::Unknown {
                print!("{:02X?}", c.param_bytes);
            } else {
                print!("{:?}", o);
            }
        } else if let Some(r) = c.fixed_param {
            print!("{:?}", r);
        }
        println!();
    }
    */
}

fn parse_script<S: Read>(script_stream: &mut S, name: &str, length: u64) -> Script {
    let mut opcodes = Vec::new();
    let mut offset = 0;
    let mut i = 0;
    let mut pos = 0;
    while offset < length as usize {
        // Read all base info
        let (length, number, flag) = (
            script_stream.read_u16::<LE>().unwrap() as usize,
            script_stream.read_u8().unwrap(),
            script_stream.read_u8().unwrap()
        );
        let string = OPCODES[number as usize].clone();

        offset += 4;

        let raw_len = length - 4;
        let mut raw_bytes = vec![0u8; raw_len];
        script_stream.read_exact(&mut raw_bytes).unwrap();
        offset += raw_len;

        // Read extra align byte if alignment needed
        if length % 2 != 0 {
            offset += 1;
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
    opcode_number: u8,
    string: String,

    flag: u8,
    fixed_param: Option<Vec<u16>>,
    param_bytes: Vec<u8>,
    opcode_specifics: Option<SpecificOpcode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Unknown,
}

impl SpecificOpcode {
    pub fn decode(opcode_str: &str, param_bytes: &[u8]) -> Option<Self> {
        if param_bytes.is_empty() {
            return None
        }

        Some(match opcode_str {
            "MESSAGE" => Self::parse_message(param_bytes),
            "SAYAVOICETEXT" => Self::parse_sayavoicetext(param_bytes),
            "SELECT" => Self::parse_select(param_bytes),
            "TASK" => Self::parse_task(param_bytes),

            "ADD" => Self::parse_add(param_bytes),
            "EQUN" => Self::parse_equn(param_bytes),

            "RANDOM" => Self::parse_random(param_bytes),
            "IFY" => Self::parse_ifn_ify(param_bytes, false),
            "IFN" => Self::parse_ifn_ify(param_bytes, true),
            "JUMP" => Self::parse_jump(param_bytes),
            "GOTO" => Self::parse_goto(param_bytes),
            "GOSUB" => Self::parse_gosub(param_bytes),
            "FARCALL" => Self::parse_farcall(param_bytes),
            "VARSTR_SET" => Self::parse_varstr_set(param_bytes),

            "IMAGELOAD" => Self::parse_imageload(param_bytes),
            "BGM" => Self::parse_bgm(param_bytes),
            _ => Self::Unknown
        })
    }

    fn parse_message(param_bytes: &[u8]) -> Self {
        let (mut offset, voice_id) = utils::get_u16(param_bytes, 0).unwrap();

        // TODO: This will need to change per-game based on the number of
        // languages and their encodings
        let mut messages = Vec::new();
        for _ in 0..2 {
            let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
            messages.push(string);
            offset = o;
        }

        Self::Message {
            voice_id,
            messages,
            end: param_bytes[offset..].to_vec()
        }
    }

    fn parse_add(param_bytes: &[u8]) -> Self {
        let (offset, var1) = utils::get_u16(param_bytes, 0).unwrap();
        let (_, expr) = utils::get_string(param_bytes, offset, Encoding::ShiftJIS, None).unwrap();

        Self::Add { var1, expr }
    }

    fn parse_equn(param_bytes: &[u8]) -> Self {
        let (offset, var1) = utils::get_u16(param_bytes, 0).unwrap();

        let mut value = None;
        if offset < param_bytes.len() {
            let (_, v) = utils::get_u16(param_bytes, offset).unwrap();
            value = Some(v);
        }

        Self::EquN { var1, value }
    }

    fn parse_select(param_bytes: &[u8]) -> Self {
        let (offset, var_id) = utils::get_u16(param_bytes, 0).unwrap();
        let (offset, var0) = utils::get_u16(param_bytes, offset).unwrap();
        let (offset, var1) = utils::get_u16(param_bytes, offset).unwrap();
        let (mut offset, var2) = utils::get_u16(param_bytes, offset).unwrap();

        // TODO: This will need to change per-game based on the number of
        // languages and their encodings
        let mut messages = Vec::new();
        for _ in 0..2 {
            let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
            messages.push(string);
            offset = o;
        }

        let (offset, var3) = utils::get_u16(param_bytes, offset).unwrap();
        let (offset, var4) = utils::get_u16(param_bytes, offset).unwrap();
        let (_, var5) = utils::get_u16(param_bytes, offset).unwrap();

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

    fn parse_random(param_bytes: &[u8]) -> Self {
        let (offset, var1) = utils::get_u16(param_bytes, 0).unwrap();
        let (offset, rnd_from) = utils::get_string(param_bytes, offset, Encoding::ShiftJIS, None).unwrap();
        let (_, rnd_to) = utils::get_string(param_bytes, offset, Encoding::ShiftJIS, None).unwrap();

        Self::Random { var1, rnd_from, rnd_to }
    }

    fn parse_ifn_ify(param_bytes: &[u8], ifn: bool) -> Self {
        let (offset, condition) = utils::get_string(param_bytes, 0, Encoding::ShiftJIS, None).unwrap();
        let (_, jump_pos) = utils::get_u32(param_bytes, offset).unwrap();

        if ifn {
            Self::IfN { condition, jump_pos }
        } else {
            Self::IfY { condition, jump_pos }
        }
    }

    fn parse_jump(param_bytes: &[u8]) -> Self {
        let (offset, filename) = utils::get_string(param_bytes, 0, Encoding::ShiftJIS, None).unwrap();

        let jump_pos = if param_bytes.len() > offset {
            let (_, j) = utils::get_u32(param_bytes, offset).unwrap();
            Some(j)
        } else {
            None
        };

        Self::Jump { filename, jump_pos }
    }

    fn parse_imageload(param_bytes: &[u8]) -> Self {
        let (offset, mode) = utils::get_u16(param_bytes, 0).unwrap();
        let (mut offset, image_id) = utils::get_u16(param_bytes, offset).unwrap();

        let mut var1 = None;
        let mut pos_x = None;
        let mut pos_y = None;
        if mode != 0 && mode != 8 && param_bytes.len() > offset {
            let var1_2 = utils::get_u16(param_bytes, offset).unwrap();
            var1 = Some(var1_2.1);
            let pos_x_2 = utils::get_u16(param_bytes, var1_2.0).unwrap();
            pos_x = Some(pos_x_2.1);
            let pos_y_2 = utils::get_u16(param_bytes, pos_x_2.0).unwrap();
            pos_y = Some(pos_y_2.1);
            offset = pos_y_2.0;
        }

        Self::ImageLoad {
            mode,
            image_id,
            var1,
            pos_x,
            pos_y,
            end: param_bytes[offset..].to_vec(),
        }
    }

    fn parse_goto(param_bytes: &[u8]) -> Self {
        let (_, jump_pos) = utils::get_u32(param_bytes, 0).unwrap();

        Self::GoTo { jump_pos }
    }

    fn parse_gosub(param_bytes: &[u8]) -> Self {
        let (offset, arg1) = utils::get_u16(param_bytes, 0).unwrap();
        let (offset, jump_pos) = utils::get_u32(param_bytes, offset).unwrap();

        Self::GoSub { arg1, jump_pos, end: param_bytes[offset..].to_vec() }
    }

    fn parse_varstr_set(param_bytes: &[u8]) -> Self {
        let (offset, varstr_id) = utils::get_u16(param_bytes, 0).unwrap();
        let (_, varstr_str) = utils::get_string(param_bytes, offset, Encoding::ShiftJIS, None).unwrap();

        Self::VarStrSet { varstr_id, varstr_str }
    }

    fn parse_farcall(param_bytes: &[u8]) -> Self {
        let (offset, index) = utils::get_u16(param_bytes, 0).unwrap();
        let (offset, filename) = utils::get_string(param_bytes, offset, Encoding::ShiftJIS, None).unwrap();
        let (offset, jump_pos) = utils::get_u32(param_bytes, offset).unwrap();

        Self::FarCall { index, filename, jump_pos, end: param_bytes[offset..].to_vec() }
    }

    fn parse_sayavoicetext(param_bytes: &[u8]) -> Self {
        let (mut offset, voice_id) = utils::get_u16(param_bytes, 0).unwrap();

        // TODO: This will need to change per-game based on the number of
        // languages and their encodings
        let mut messages = Vec::new();
        for _ in 0..2 {
            let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
            messages.push(string);
            offset = o;
        }

        Self::SayAVoiceText {
            voice_id,
            messages,
        }
    }

    fn parse_bgm(param_bytes: &[u8]) -> Self {
        // TODO: invesigate the accuracy of this
        let (offset, bgm_id) = utils::get_u32(param_bytes, 0).unwrap();

        let arg2 = if bgm_id == 0 {
            Some(utils::get_u16(param_bytes, offset).unwrap().1)
        } else {
            None
        };

        Self::Bgm {
            bgm_id,
            arg2,
        }
    }

    fn parse_task(param_bytes: &[u8]) -> Self {
        let (offset, task_type) = utils::get_u16(param_bytes, 0).unwrap();

        let mut var1 = None;
        let mut var2 = None;
        let mut var3 = None;
        let mut var4 = None;
        let mut message_1 = None;
        let mut message_2 = None;
        let raw_args: Option<Vec<u8>> = None;

        let abort_task = Self::Task {
            task_type,
            var1,
            var2,
            var3,
            var4,
            message_1: message_1.clone(),
            message_2: message_2.clone(),
            raw_args: Some(param_bytes.to_vec())
        };

        if param_bytes.len() <= offset {
            return abort_task;
        }

        match task_type {
            4 => {
                let (offset, v1) = utils::get_u16(param_bytes, offset).unwrap();
                var1 = Some(v1);
                if param_bytes.len() <= offset {
                    return abort_task;
                }

                if [0, 4, 5].contains(&v1) {
                    let (mut offset, v2) = utils::get_u16(param_bytes, offset).unwrap();
                    var2 = Some(v2);

                    let mut messages = Vec::new();
                    for _ in 0..2 {
                        let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
                        messages.push(string);
                        offset = o;
                    }
                    message_1 = Some(messages);
                } else if v1 == 1 {
                    let (offset, v2) = utils::get_u16(param_bytes, offset).unwrap();
                    var2 = Some(v2);
                    let (offset, v3) = utils::get_u16(param_bytes, offset).unwrap();
                    var3 = Some(v3);
                    let (mut offset, v4) = utils::get_u16(param_bytes, offset).unwrap();
                    var4 = Some(v4);

                    // Get first set of messages
                    let mut messages = Vec::new();
                    for _ in 0..2 {
                        let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
                        messages.push(string);
                        offset = o;
                    }
                    message_1 = Some(messages);

                    // Get second set of messages
                    let mut messages = Vec::new();
                    for _ in 0..2 {
                        let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
                        messages.push(string);
                        offset = o;
                    }
                    message_2 = Some(messages);
                } else if v1 == 6 {
                    let (offset, v2) = utils::get_u16(param_bytes, offset).unwrap();
                    var2 = Some(v2);
                    let (mut offset, v3) = utils::get_u16(param_bytes, offset).unwrap();
                    var3 = Some(v3);

                    let mut messages = Vec::new();
                    for _ in 0..2 {
                        let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
                        messages.push(string);
                        offset = o;
                    }
                    message_1 = Some(messages);
                } else {
                    return abort_task;
                }
            }
            54 => {
                let (_, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
                message_1 = Some(vec![string]);
            }
            69 => {
                let (mut offset, v1) = utils::get_u16(param_bytes, offset).unwrap();
                var1 = Some(v1);

                // Get first set of messages
                let mut messages = Vec::new();
                for _ in 0..2 {
                    let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
                    messages.push(string);
                    offset = o;
                }
                message_1 = Some(messages);

                // Get second set of messages
                let mut messages = Vec::new();
                for _ in 0..2 {
                    let (o, string) = utils::get_string(param_bytes, offset, Encoding::UTF16, None).unwrap();
                    messages.push(string);
                    offset = o;
                }
                message_2 = Some(messages);
            }
            _ => return abort_task
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
