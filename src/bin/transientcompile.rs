//! Compiler that transforms Transient-C into TransientIR. (TIR)
//! Currently under development.


/*
    0x01: MOV byte from source1 into destination
    0x02: ADD source1 and source2 and store result in destination
    0x03: SUB source2 from source1 and store result in destination
    0x04: MUL source1 and source2 and store result in destination
    0x05: DIV source1 by source2 and store result in destination (truncated)
    0x06: DIV source1 by source2 and store result in destination (rounded)
    0x07: REM divides source1 by source2 and stores the remainder in destination
    0x08: CGT compare if source1 is greater than source2, and if so, store 1 in destination
    0x09: CLT compare if source1 is less than source2, and if so, store 1 in destination
    0x0A: JMP stops current execution and jumps to code in source1
    0x0B: JIE stops current execution and jumps to code in source1 ONLY IF source2 is non-zero
    0x0C: JNE stops current execution and jumps to code in source1 ONLY IF source2 is zero
    0x0D: PUT prints data at source1 to the screen (int)
    0x0E: PUT prints data at source1 to the screen (char)
    0x0F: IMZ gets the image size that was loaded to ROM and stores it in destination
    0x10: EQU compare if source1 and source2 are equal, and if so, store 1 in destination
    0xFF: HLT halts execution and stops processor
*/


use std::env::args;
use std::io::{Read, Write};
use std::fs::File;
use std::collections::HashMap;

#[derive(Debug, Hash, Eq, PartialEq)]
enum Operation {
    Mov(usize, usize, usize),
    Add(usize, usize, usize, usize),
    Sub(usize, usize, usize, usize),
    Mul(usize, usize, usize, usize),
    DivT(usize, usize, usize, usize),
    DivR(usize, usize, usize, usize),
    Rem(usize, usize, usize, usize),
    Cgt(usize, usize, usize, usize),
    Clt(usize, usize, usize, usize),
    Jmp(usize),
    Jie(usize, usize, usize),
    Jne(usize, usize, usize),
    PutI(usize, usize),
    PutC(usize, usize),
    Imz(usize, usize),
    Equ(usize, usize, usize, usize),
    Hlt(),
}

fn resolve_operation_opcode(operation: &Operation) -> u8 {
    match operation {
        Operation::Mov(..) => 0x01,
        Operation::Add(..) => 0x02,
        Operation::Sub(..) => 0x03,
        Operation::Mul(..) => 0x04,
        Operation::DivT(..) => 0x05,
        Operation::DivR(..) => 0x06,
        Operation::Rem(..) => 0x07,
        Operation::Cgt(..) => 0x08,
        Operation::Clt(..) => 0x09,
        Operation::Jmp(..) => 0x0A,
        Operation::Jie(..) => 0x0B,
        Operation::Jne(..) => 0x0C,
        Operation::PutI(..) => 0x0D,
        Operation::PutC(..) => 0x0E,
        Operation::Imz(..) => 0x0F,
        Operation::Equ(..) => 0x10,
        Operation::Hlt(..) => 0xFF,
    }
}

fn preprocess_source_code(source_code: Vec<String>) -> (Vec<Operation>, HashMap<String, (usize, u64, usize)>) {
    let mut source_code = source_code;

    // Pass 1
    // Count IR LoC
    let mut lines_of_ir = 0usize;
    for line in &source_code {
        // Check if it's actual IR
        if !line.is_empty() && !line.starts_with("#") && !line.starts_with("//") && !line.starts_with("set") {
            lines_of_ir += 1;
        }
    }
    let ir_size_bytes = lines_of_ir * 8;

    // Pass 2
    // Build hashmap of variables
    let mut memory_map: HashMap<String, (usize, u64, usize)> = HashMap::new(); // Address, value,
                                                                               // size
    let mut memory_offset = 0usize;
    for line in &source_code {
        // Skip if not declaration
        if !line.starts_with("set") {
            continue;
        }
        // set{bits} $variable value
        let line_tokens: Vec<String> = line.split(" ").map(|x| {x.to_owned()}).collect();
        assert!(
            line_tokens.len() == 3,
            "Invalid set syntax"
        );
        assert!(
            line_tokens[1].starts_with("$"),
            "Invalid variable"
        );
        let size = usize::from_str_radix(&line_tokens[0][3..], 10).expect("Failed to parse size") / 8;

        memory_map.insert(
            line_tokens[1][1..].to_string(),
            (ir_size_bytes + memory_offset, u64::from_str_radix(&line_tokens[2], 10).expect("Failed to parse value"), size),
        );
        memory_offset += size
    }

    // Pass 3
    // Erase comments, sets, and empty lines
    source_code.retain(|line| {
        !line.is_empty() && !line.starts_with("//") && !line.starts_with("set")
    });

    // Pass 4
    // Repeatedly scan and generate tag addresses
    let mut jump_addresses: HashMap<String, usize> = HashMap::new();
    loop {
        let mut clean = true;
        let mut index_to_remove: usize = 0;
        for     (index, line) in source_code.iter().enumerate() {
            if line.starts_with("#") {
                clean = false;
                jump_addresses.insert(line[1..].to_owned(), index*8);
                index_to_remove = index;
                break;
            }
        }
        if clean {
            break;
        } else {
            source_code.remove(index_to_remove);
        }
    }

    // Pass 5
    // Build abstract syntax tree
    let mut abstract_syntax_tree: Vec<Operation> = Vec::new();
    for line in source_code {
        let line_tokens: Vec<String> = line.split(" ").map(|x| {x.to_owned()}).collect();
        // Extract 'add' from 'add64'
        let opcode: String = line_tokens[0].chars().filter(|x|{x.is_alphabetic()}).collect::<String>();
        let size: usize = usize::from_str_radix(&line_tokens[0].chars().filter(|x|{x.is_numeric()}).collect::<String>(), 10).expect("Failed to parse size");
        let args: Vec<usize> = line_tokens[1..].iter().map(|x|{
            if x.starts_with("#") {
                jump_addresses.get(&x[1..]).expect("Jump address resolution failed").clone()
            } else if x.starts_with("$") {
                memory_map.get(&x[1..]).expect("Memory resolution failed").0
            } else {
                panic!("Argument parsing fail");
            }
        }).collect();
        abstract_syntax_tree.push(match &opcode[..] {
            "mov" => {
                Operation::Mov(size, args[0], args[1])
            }
            "add" => {
                Operation::Add(size, args[0], args[1], args[2])
            },
            "sub" => {
                Operation::Sub(size, args[0], args[1], args[2])
            }
            "mul" => {
                Operation::Mul(size, args[0], args[1], args[2])
            }
            "divt" => {
                Operation::DivT(size, args[0], args[1], args[2])
            }
            "divr" => {
                Operation::DivR(size, args[0], args[1], args[2])
            }
            "rem" => {
                Operation::Rem(size, args[0], args[1], args[2])
            }
            "cgt" => {
                Operation::Cgt(size, args[0], args[1], args[2])
            }
            "clt" => {
                Operation::Clt(size, args[0], args[1], args[2])
            }
            "jmp" => {
                Operation::Jmp(args[0])
            }
            "jie" => {
                Operation::Jie(size, args[0], args[1])
            }
            "jne" => {
                Operation::Jne(size, args[0], args[1])
            }
            "puti" => {
                Operation::PutI(size, args[0])
            }
            "putc" => {
                Operation::PutC(size, args[0])
            }
            "imz" => {
                Operation::Imz(size, args[0])
            }
            "equ" => {
                Operation::Equ(size, args[0], args[1], args[2])
            }
            "hlt" => {
                Operation::Hlt()
            }
            _ => {
                panic!("Unknown operation");
            }
        })
    }

    dbg!(&abstract_syntax_tree, &memory_map);
    (abstract_syntax_tree, memory_map)
}

fn gen_binary_instruction(opcode: u8, size: usize, src1: usize, src2: usize, dest: usize) -> [u8; 8] {
    [
        opcode,
        size as u8,
        (src1 as u16).to_be_bytes()[0],
        (src1 as u16).to_be_bytes()[1],
        (src2 as u16).to_be_bytes()[0],
        (src2 as u16).to_be_bytes()[1],
        (dest as u16).to_be_bytes()[0],
        (dest as u16).to_be_bytes()[1],
    ]
}

fn codegen(abstract_syntax_tree: Vec<Operation>, memory_map: HashMap<String, (usize, u64, usize)>) -> Vec<u8> {
    let mut image: Vec<u8> = vec![];
    
    // Write instructions to image
    for (index, instruction) in abstract_syntax_tree.iter().enumerate() {
        let opcode = resolve_operation_opcode(&instruction);
        match *instruction {
            Operation::Mov(size, src1, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, 0x00, dest));
            }
            Operation::Add(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::Sub(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::Mul(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::DivT(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::DivR(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::Rem(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::Cgt(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::Clt(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::Jmp(src1) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, 0x00, src1, 0x00, 0x00));
            }
            Operation::Jie(size, src1, src2) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, 0x00));
            }
            Operation::Jne(size, src1, src2) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, 0x00));
            }
            Operation::PutI(size, src1) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, 0x00, 0x00));
            }
            Operation::PutC(size, src1) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, 0x00, 0x00));
            }
            Operation::Imz(size, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, 0x00, 0x00, dest));
            }
            Operation::Equ(size, src1, src2, dest) => {
                image.extend_from_slice(&gen_binary_instruction(opcode, size, src1, src2, dest));
            }
            Operation::Hlt() => {
                image.extend_from_slice(&gen_binary_instruction(opcode, 0x00, 0x00, 0x00, 0x00));
            }
        }
    }

    // Write variables to image
    for (address, value, size) in memory_map.values() {
        image.extend_from_slice(value.to_be_bytes()[value.to_be_bytes().len()-size..].try_into().expect("Failed to write variable to image"))
    }

    image
}
    
fn main() {
    // Verify input parameters
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        println!("Stop: Incorrect amount of arguments!");
        return;
    }

    // Open file for reading
    let mut input_file = match File::open(&args[1]) {
        Ok(x) => x,
        Err(_) => {
            panic!("Stop: Failed to open file");
        }
    };

    // Read bytes into buffer
    let mut source_code: String = String::new();
    if let Err(_) = input_file.read_to_string(&mut source_code) {
        panic!("Stop: Failed to read file contents");
    }
    let source_code: Vec<String> = source_code.split("\n").map(|x| {x.to_owned()}).collect();
    println!("Info: File read");

    // Preprocess, resolve memory addresses, and generate abstract syntax tree
    let (abstract_syntax_tree, memory_map) = preprocess_source_code(source_code);

    // Codegen
    let executable = codegen(abstract_syntax_tree, memory_map);
    dbg!(executable);
}
