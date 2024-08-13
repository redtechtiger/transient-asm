//! Transient is, in essence, a custom virtual machine and file format. The transient processor
//! loads a transient "image' into the virtual address space and begins execution at offset 0x00.
//!
//!
//! # Opcodes
//! - 0x01: MOV byte from source1 into destination
//! - 0x02: ADD source1 and source2 and store result in destination
//! - 0x03: SUB source2 from source1 and store result in destination
//! - 0x04: MUL source1 and source2 and store result in destination
//! - 0x05: DIV source1 by source2 and store result in destination (truncated)
//! - 0x06: DIV source1 by source2 and store result in destination (rounded)
//! - 0x07: REM divides source1 by source2 and stores the remainder in destination
//! - 0x08: CGT compare if source1 is greater than source2, and if so, store 1 in destination
//! - 0x09: CLT compare if source1 is less than source2, and if so, store 1 in destination
//! - 0x0A: JMP stops current execution and jumps to code in source1
//! - 0x0B: JIE stops current execution and jumps to code in source1 ONLY IF source2 is non-zero
//! - 0x0C: JNE stops current execution and jumps to code in source1 ONLY IF source2 is zero
//! - 0x0D: PUT prints data at source1 to the screen (int)
//! - 0x0E: PUT prints data at source1 to the screen (char)
//! - 0x0F: IMZ gets the image size that was loaded to ROM and stores it in destination
//! - 0x10: EQU compare if source1 and source2 are equal, and if so, store 1 in destination
//! - 0xFF: HLT halts execution and stops processor
//!
//! # Transient addresses
//! Source1, source2, and destination are transient addresses. These can range from 0 up to TRANSIENT_MEM_MAX. Do note, however, that the transient processor will
//! fill the transient memory with program data up to the programs length. To get the length of the
//! program, see opcodes above.

/*
Mov
Layout: opcode ptr_mode add_size arg_1 arg_2
Opcode: 0x01
Description: Sets arg_1 to arg_2

Add
Layout: opcode ptr_mode add_size arg_1 arg_2 arg_3
Opcode: 0x02
Description: Adds arg_1 and arg_2 and stores in arg_3

Sub
Layout: opcode ptr_mode add_size arg_1 arg_2 arg_3
Opcode: 0x03
Description: Subtracts arg_2 from arg_1 and stores in arg_3

Mul
Layout: opcode ptr_mode add_size arg_1 arg_2 arg_3
Opcode: 0x04
Description: Multiplies arg_1 and arg_2 and stores in arg_3

Div
Layout: opcode ptr_mode add_size arg_1 arg_2 arg_3
Opcode: 0x05
Description: Divides arg_1 by arg_2 and stores quotient in arg_3

Rem
Layout: opcode ptr_mode add_size arg_1 arg_2 arg_3
Opcode: 0x06
Description: Divides arg_1 and arg_2 and stores remainder in arg_3

Equ
Layout: opcode ptr_mode add_size arg_1 arg_2 arg_3
Opcode: 0x07
Description: If arg_1 is equal to arg_2, store 0x1 in arg_3, otherwise store 0x0

Cgt
Layout: opcode ptr_mode add_size arg_1 arg_2 arg_3
Opcode: 0x08
Description: If arg_1 is greater than arg_2, store 0x1 in arg_3, otherwise store 0x0

Clt
Layout: opcode ptr_mode add_size arg_1 arg_2 arg_3
Opcode: 0x09
Description: If arg_1 is less than arg_2, store 0x1 in arg_3, otherwise store 0x0

Jmp
Layout: opcode ptr_mode arg_1
Opcode: 0x0A
Description: Set program counter to arg_1, effectively jumping to arg_1

Jie
Layout: opcode ptr_mode add_size arg_1 arg_2
Opcode: 0x0B
Description: Set program counter to arg_2 if arg_1 is 0x1.

Jne
Layout: opcode ptr_mode add_size arg_1 arg_2
Opcode: 0x0C
Description: Set program counter to arg_2 if arg_1 is 0x0.

PutI
Layout: opcode ptr_mode add_size arg_1
Opcode: 0x0D
Description: Print arg_1 to the console as an integer.

PutC
Layout: opcode ptr_mode add_size arg_1
Opcode: 0x0E
Description: Print arg_1 to the console as an ascii character.

Imz
Layout: opcode ptr_mode add_size arg_1
Opcode: 0x0F
Description: Invokes the image size (in bytes) from the virtual machine and stores it in arg_1

Hlt
Layout: opcode
Opcode: 0xFF
Description: Halts execution and exits the virtual machine
*/

const MOV: u8 = 0x01;
const ADD: u8 = 0x02;
const SUB: u8 = 0x03;
const MUL: u8 = 0x04;
const DIV: u8 = 0x05;
const REM: u8 = 0x06;
const EQU: u8 = 0x07;
const CGT: u8 = 0x08;
const CLT: u8 = 0x09;
const JMP: u8 = 0x0A;
const JIE: u8 = 0x0B;
const JNE: u8 = 0x0C;
const PUT_I: u8 = 0x0D;
const PUT_C: u8 = 0x0E;
const IMZ: u8 = 0x0F;
const HLT: u8 = 0xFF;

use std::env::args;
use std::fs::File;
use std::io::Read;

const TRANSIENT_MEM_MAX: usize = 0xFFFF;

#[derive(PartialEq)]
pub enum TransientMode {
    RUNNING,
    HALTED,
}

pub struct TransientState<const TRANSIENT_MEM_MAX: usize> {
    pub memory: Vec<u8>,
    pub memory_limit: usize,
    pub image_length: usize, // Length of executable code in memory
    pub program_counter: usize,
    pub mode: TransientMode,
}

impl<const TRANSIENT_MEM_MAX: usize> TransientState<TRANSIENT_MEM_MAX> {
    /// Initialize a new, empty instance of a transient processor/state with a transient memory
    /// size of TRANSIENT_MEM_MAX bytes.
    pub fn new() -> Self {
        TransientState {
            memory: vec![],
            memory_limit: TRANSIENT_MEM_MAX,
            image_length: 0,
            program_counter: 0,
            mode: TransientMode::HALTED,
        }
    }
    /// Loads a transient memory image into a state/processor at a specified offset.
    pub fn load_image(&mut self, offset: usize, image: &[u8]) {
        // Allocate space for image and set it to 0x00
        self.memory.resize(image.len(), 0x00);
        // Copy over image data
        self.memory[offset..image.len() + offset].copy_from_slice(image);
        // Set image lengt of processor data
        self.image_length = image.len();
    }
    /// Starts a loop that runs the processor until halted
    pub fn run(&mut self, start: usize) {
        self.program_counter = start;
        self.mode = TransientMode::RUNNING;
        while self.mode == TransientMode::RUNNING {
            let instruction = self.resolve_instruction(self.program_counter);
            self.program_counter = self.execute_instruction(&instruction);
        }
    }
    pub fn resolve_instruction(&self, base_ptr: usize) -> Vec<u8> {
        // Fetch correct number of bytes depending on instruction
        match self.memory[base_ptr] {
            MOV => &self.memory[base_ptr..][..5],
            ADD => &self.memory[base_ptr..][..6],
            SUB => &self.memory[base_ptr..][..6],
            MUL => &self.memory[base_ptr..][..6],
            DIV => &self.memory[base_ptr..][..6],
            REM => &self.memory[base_ptr..][..6],
            EQU => &self.memory[base_ptr..][..6],
            CGT => &self.memory[base_ptr..][..6],
            CLT => &self.memory[base_ptr..][..6],
            JMP => &self.memory[base_ptr..][..3],
            JIE => &self.memory[base_ptr..][..5],
            JNE => &self.memory[base_ptr..][..5],
            PUT_I => &self.memory[base_ptr..][..4],
            PUT_C => &self.memory[base_ptr..][..4],
            IMZ => &self.memory[base_ptr..][..4],
            HLT => &self.memory[base_ptr..][..1],
            _ => panic!("[Halt]: Instruction resolution failed: Invalid opcode")
        }.to_vec()
    }
    /// Executes an instruction and returns the next program counter
    pub fn execute_instruction(&mut self, instruction: &[u8]) -> usize {
        // Decodes instruction
        let opcode = instruction[0];
    }
}

fn u64_pad_be(data: &[u8]) -> [u8; 8] {
    let mut padded = [0u8; 8];
    padded[(8 - data.len())..].copy_from_slice(data);
    padded
}

fn main() {
    // Verify input arguments
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!("Stop: Incorrect amount of arguments!");
    }

    // Open file for reading
    let mut input_file = match File::open(&args[1]) {
        Ok(x) => x,
        Err(_) => {
            panic!("Stop: Failed to open file");
        }
    };

    // Read bytes into buffer
    let mut transient_image: Vec<u8> = vec![];
    if let Err(_) = input_file.read_to_end(&mut transient_image) {
        panic!("Stop: Failed to read file contents");
    }
    println!("Info: File read");

    // Initialize transient processor
    let mut transient_state = TransientState::<TRANSIENT_MEM_MAX>::new();
    println!("Info: Transient processor initialized");

    // Copy over image at offset 0 (at the start)
    transient_state.load_image(0, &transient_image);
    println!("Info: Transient image loaded");

    // Begin executing
    transient_state.run(0);

    println!("Info: End of program reached");
}
