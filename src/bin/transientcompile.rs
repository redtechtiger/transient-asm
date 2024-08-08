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
use std::process::exit;
use std::hash::{DefaultHasher, Hash, Hasher};

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
    // Remove all comments
    source_code.retain(|x| {!x.starts_with("//")});

    // Pass 2
    // Calculate all intermediates
    let mut intermediates: HashMap<u64, (usize, usize)> = HashMap::new();
    for line in source_code.iter() {
        let line_tokens: Vec<String> = line.split(" ").map(|x| {x.to_owned()}).collect();
        for token in line_tokens {
            if !token.starts_with("!") {
                continue;
            }
            let intermediate_parts: Vec<String> = token.split("_").map(|x| {x.to_owned()}).collect();
            if intermediate_parts.len() != 2 {
                halt_compilation("[E011] Intermediate syntax incorrect. Did you remember to specify the size?", line);
            }
            let size = usize::from_str_radix(&intermediate_parts[0][1..], 10).unwrap_or_else(|_| { halt_compilation("[E003] Failed to parse size: Did you remember to specify the size of the operation?", &line)});
            let value = usize::from_str_radix(&intermediate_parts[1], 10).unwrap_or_else(|_| { halt_compilation("[E012] Failed to parse intermediate value: Only integers are allowed", &line) });
            let mut hasher = DefaultHasher::new();
            token.hash(&mut hasher);
            let hash = hasher.finish();
            if intermediates.get(&hash).is_some() {
                continue;
            }
            intermediates.insert(hash, (value, size));
        }
    }
    // Pass 3
    // Insert new intermediate variable declarations
    for (hash, (value, size)) in intermediates.iter() {
        source_code.insert(0, format!("set{size} ${hash} {value}"));
        for line in source_code.iter_mut() {
            *line = line.replace(&format!("!{size}_{value}"), &format!("${hash}"));
        }
    }

    // Pass 4
    // Count IR LoC
    let mut lines_of_ir = 0usize;
    for line in &source_code {
        // Check if it's actual IR
        if !line.is_empty() && !line.starts_with("#") && !line.starts_with("//") && !line.starts_with("set") {
            lines_of_ir += 1;
        }
    }
    let ir_size_bytes = lines_of_ir * 8;

    // Pass 5
    // Build hashmap of variables to memory
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
        if line_tokens.len() != 3 {
            halt_compilation("[E001] Invalid set syntax: Did you remember to initialize the variable?", line);
        }
        if !line_tokens[1].starts_with("$") {
            halt_compilation("[E002] Invalid variable: Did you remember to preface it with a dollar sign? ($)", line);
        }
        // Check if variable exists
        if memory_map.get(&line_tokens[1][1..]).is_some() {
            halt_compilation("[E010] Variable memory collision: Did you initialize the same variable twice?", &line);
        }
        let size = match usize::from_str_radix(&line_tokens[0][3..], 10) {
            Ok(x) => x / 8,
            Err(..) => halt_compilation("[E003] Failed to parse size: Did you remember to specify the size of the operation?", line),
        };
        let value = match u64::from_str_radix(&line_tokens[2], 10) {
            Ok(x) => x,
            Err(..) => halt_compilation("[E004] Failed to parse value: Only integer values are allowed", line)
        };

        memory_map.insert(
            line_tokens[1][1..].to_string(),
            (ir_size_bytes + memory_offset, value, size)
        );
        memory_offset += size
    }

    // Pass 6
    // Erase sets, and empty lines
    source_code.retain(|line| {
        !line.is_empty() && !line.starts_with("set")
    });

    // Pass 7
    // Repeatedly scan and generate tag addresses
    let mut jump_addresses: HashMap<String, usize> = HashMap::new();
    loop {
        let mut clean = true;
        let mut index_to_remove: usize = 0;
        for (index, line) in source_code.iter().enumerate() {
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

    // Pass 8
    // Build abstract syntax tree
    let mut abstract_syntax_tree: Vec<Operation> = Vec::new();
    for line in source_code {
        let line_tokens: Vec<String> = line.split(" ").map(|x| {x.to_owned()}).collect();
        // Extract 'add' from 'add64'
        let opcode: String = line_tokens[0].chars().filter(|x|{x.is_alphabetic()}).collect::<String>();
        let size: usize = usize::from_str_radix(&line_tokens[0].chars().filter(|x|{x.is_numeric()}).collect::<String>(), 10).unwrap_or_else(|_| { halt_compilation("[E003] Failed to parse size: Did you remember to specify the size of the operation?", &line)}) / 8;
        let args: Vec<usize> = line_tokens[1..].iter().map(|x|{
            if x.starts_with("#") {
                jump_addresses.get(&x[1..]).unwrap_or_else(|| { halt_compilation("[E005] Jump address resolution failed: Try checking your spelling", &line) }).clone()
            } else if x.starts_with("$") {
                memory_map.get(&x[1..]).unwrap_or_else(|| { halt_compilation("[E006] Memory resolution failed: Try checking your spelling", &line) }).0
            } else {
                halt_compilation("[E007] Invalid argument to function: Only variables and tags are allowed as arguments", &line);
            }
        }).collect();
        abstract_syntax_tree.push(match &opcode[..] {
            "mov" => {
                if args.len() != 2 {
                    halt_compilation("[E008] This function takes 2 arguments", &line);
                }
                Operation::Mov(size, args[0], args[1])
            }
            "add" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 arguments", &line);
                }
                Operation::Add(size, args[0], args[1], args[2])
            },
            "sub" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 arguments", &line);
                }
                Operation::Sub(size, args[0], args[1], args[2])
            }
            "mul" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 arguments", &line);
                }
                Operation::Mul(size, args[0], args[1], args[2])
            }
            "divt" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 arguments", &line);
                }
                Operation::DivT(size, args[0], args[1], args[2])
            }
            "divr" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 arguments", &line);
                }
                Operation::DivR(size, args[0], args[1], args[2])
            }
            "rem" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 arguments", &line);
                }
                Operation::Rem(size, args[0], args[1], args[2])
            }
            "cgt" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 arguments", &line);
                }
                Operation::Cgt(size, args[0], args[1], args[2])
            }
            "clt" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 arguments", &line);
                }
                Operation::Clt(size, args[0], args[1], args[2])
            }
            "jmp" => {
                if args.len() != 1 {
                    halt_compilation("[E008] This function takes 1 argument", &line);
                }
                Operation::Jmp(args[0])
            }
            "jie" => {
                if args.len() != 2 {
                    halt_compilation("[E008] This function takes 2 arguments", &line);
                }
                Operation::Jie(size, args[0], args[1])
            }
            "jne" => {
                if args.len() != 2 {
                    halt_compilation("[E008] This function takes 2 arguments", &line);
                }
                Operation::Jne(size, args[0], args[1])
            }
            "puti" => {
                if args.len() != 1 {
                    halt_compilation("[E008] This function takes 1 argument", &line);
                }
                Operation::PutI(size, args[0])
            }
            "putc" => {
                if args.len() != 1 {
                    halt_compilation("[E008] This function takes 1 argument", &line);
                }
                Operation::PutC(size, args[0])
            }
            "imz" => {
                if args.len() != 1 {
                    halt_compilation("[E008] This function takes 1 argument", &line);
                }
                Operation::Imz(size, args[0])
            }
            "equ" => {
                if args.len() != 3 {
                    halt_compilation("[E008] This function takes 3 argument", &line);
                }
                Operation::Equ(size, args[0], args[1], args[2])
            }
            "hlt" => {
                Operation::Hlt()
            }
            _ => {
                halt_compilation("[E009] Invalid opcode. Check your spelling", &line);
            }
        })
    }

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

fn codegen(abstract_syntax_tree: &Vec<Operation>, memory_map: &HashMap<String, (usize, u64, usize)>) -> Vec<u8> {
    let mut image: Vec<u8> = vec![];
    
    // Write instructions to image
    for (_index, instruction) in abstract_syntax_tree.iter().enumerate() {
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

    // Calculate amount of space that variables take
    let mut var_size = 0;
    for (_address, _value, size) in memory_map.values() {
        var_size += size;
    }

    // Allocate size for new vars
    image.resize(image.len()+var_size, 0);

    // Write variables to image
    for (address, value, size) in memory_map.values() {
        image[*address..][..*size].copy_from_slice(value.to_be_bytes()[value.to_be_bytes().len()-size..].try_into().expect("[COMPILER PANIC]: Failed to write variable to image"))
    }

    image
}

fn halt_compilation(message: &str, line: &str) -> ! {
    eprintln!("--------------------------------------------");
    eprintln!("Error: {}", message);
    eprintln!("-> Compilation failed on line `{}`", line);
    eprintln!("--------------------------------------------");
    exit(-1);
}

fn format_ast(ast: &Vec<Operation>) -> String {
    let mut out = String::new();
    for operation in ast {
        out += &format!("{:?}\n", operation);
    }
    out
}

fn format_mm(mm: &HashMap<String, (usize, u64, usize)>) -> String {
    let mut out = String::new();
    for (name, (address, value, size)) in mm {
        out += &format!("[{}]: {} = {} ({}b)\n", address, name, value, size);
    }
    out
}
    
fn main() {
    // Verify input parameters
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Stop: Incorrect amount of arguments!");
        return;
    }

    let mut verbose = false;
    if args.len() > 2 {
        verbose = args[2] == "--asm";
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
    print!("Compiling... [          ]\r");
    std::io::stdout().flush().unwrap();

    // Preprocess, resolve memory addresses, and generate abstract syntax tree
    let (abstract_syntax_tree, memory_map) = preprocess_source_code(source_code);
    print!("Compiling... [======    ]\r");
    std::io::stdout().flush().unwrap();

    // Codegen
    let executable = codegen(&abstract_syntax_tree, &memory_map);
    print!("Compiling... [========= ]\r");
    std::io::stdout().flush().unwrap();

    // Write output file
    let mut output_file = File::create("out.bin").expect("Failed to create output file");
    output_file.write(&executable).expect("Failed to write to output file");
    print!("Compiling... [==========]\n");
    
    if verbose {
        println!("AST:\n{}\nMM:\n{}", format_ast(&abstract_syntax_tree), format_mm(&memory_map))
    }

    // Done!
    println!("Success: Compilation finished ✔");
}
