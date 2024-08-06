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

const MOV: u8 = 0x01;
const ADD: u8 = 0x02;
const SUB: u8 = 0x03;
const MUL: u8 = 0x04;
const DIV_T: u8 = 0x05;
const DIV_R: u8 = 0x06;
const REM: u8 = 0x07;
const CGT: u8 = 0x08;
const CLT: u8 = 0x09;
const JMP: u8 = 0x0A;
const JIE: u8 = 0x0B;
const JNE: u8 = 0x0C;
const PUT_I: u8 = 0x0D;
const PUT_C: u8 = 0x0E;
const IMZ: u8 = 0x0F;
const EQU: u8 = 0x10;
const HLT: u8 = 0xFF;

use std::env::args;
use std::fs::File;
use std::io::{Read,Write};

const TRANSIENT_MEM_MAX: usize = 256;

#[derive(PartialEq)]
pub enum TransientMode {
    RUNNING,
    HALTED,
}

pub struct TransientState<const TRANSIENT_MEM_MAX: usize> {
    pub memory: [u8; TRANSIENT_MEM_MAX],
    pub image_length: usize, // Length of executable code in memory
    pub program_counter: usize,
    pub mode: TransientMode,
}

impl<const TRANSIENT_MEM_MAX: usize> TransientState<TRANSIENT_MEM_MAX> {
    /// Initialize a new, empty instance of a transient processor/state with a transient memory
    /// size of TRANSIENT_MEM_MAX bytes.
    pub fn new() -> Self {
        TransientState {
            memory: [0u8; TRANSIENT_MEM_MAX],
            image_length: 0,
            program_counter: 0,
            mode: TransientMode::HALTED,
        }
    }
    /// Loads a transient memory image into a state/processor at a specified offset.
    pub fn load_image(&mut self, offset: usize, image: &[u8]) {
        assert!(
            self.memory.len() >= image.len(),
            "Halt: Text section doesn't fit into transient memory"
        );
        self.memory[offset..image.len() + offset].copy_from_slice(image);
        self.image_length = image.len();
    }
    /// Starts a loop that runs the processor until halted
    pub fn run(&mut self, start: usize) {
        self.program_counter = start;
        self.mode = TransientMode::RUNNING;
        while self.mode == TransientMode::RUNNING {
            let instruction = self.resolve_instruction(self.program_counter);
            self.program_counter = self.execute_instruction(instruction);
        }
    }
    /// Fetches an instruction at the given address (mind the alignment!)
    pub fn resolve_instruction(&self, base: usize) -> [u8; 8] {
        assert!(
            self.memory.len() > base + 7,
            "Halt: Attempted instruction resolution beyond memory space"
        );
        self.memory[base..][..8].try_into().unwrap()
    }
    pub fn memory_write(&mut self, address: usize, data: &[u8], size: usize) {
        assert!(
            self.memory.len() > address + size,
            "Halt: Attempted memory write beyond memory scope"
        );
        self.memory[address..][..size].copy_from_slice(&data[data.len()-size..][..size]);

    }
    /// Executes an instruction and returns the next program counter
    pub fn execute_instruction(&mut self, instruction: [u8; 8]) -> usize {
        // Decodes instruction
        let opcode = instruction[0];
        let size = instruction[1] as usize;
        let source1_address = u16::from_be_bytes([instruction[2], instruction[3]]) as usize;
        let source2_address = u16::from_be_bytes([instruction[4], instruction[5]]) as usize; 
        let destination_address = u16::from_be_bytes([instruction[6], instruction[7]]) as usize;
        
        // Validates memory pointers
        assert!(
            self.memory.len() > source1_address,
            "Halt: Attempt to access memory beyond memory space (source1)"
        );
        assert!(
            self.memory.len() > source2_address,
            "Halt: Attempt to access memory beyond memory space (source2)"
        );
        assert!(
            self.memory.len() > destination_address,
            "Halt: Attempt to access memory beyond memory space (destination)"
        );

        let source1_data: &[u8] = &self.memory[source1_address..][..size];
        let source1_data: u64 = u64::from_be_bytes(u64_pad_be(source1_data));
        
        let source2_data: &[u8] = &self.memory[source2_address..][..size];
        let source2_data: u64 = u64::from_be_bytes(u64_pad_be(source2_data));
        
        match opcode {
            MOV => {
                self.memory_write(destination_address, &source1_data.to_be_bytes(), size);
                self.program_counter + 8
            }
            ADD => {
                self.memory_write(destination_address, &(source1_data.checked_add(source2_data).expect("Halt: Arithmetic overflow")).to_be_bytes(), size);
                self.program_counter + 8
            }
            SUB => {
                self.memory_write(destination_address, &(source1_data.checked_sub(source2_data).expect("Halt: Arithmetic overflow")).to_be_bytes(), size);
                self.program_counter + 8
            }
            MUL => {
                self.memory_write(destination_address, &(source1_data.checked_mul(source2_data).expect("Halt: Arithmetic overflow")).to_be_bytes(), size);
                self.program_counter + 8
            }
            DIV_T => {
                self.memory_write(destination_address, &(source1_data.checked_div(source2_data).expect("Halt: Arithmetic overflow")).to_be_bytes(), size);
                self.program_counter + 8
            }
            DIV_R => {
                self.memory_write(destination_address, &
                    ((source1_data as f64 / source2_data as f64) as u64).to_be_bytes(), size);
                self.program_counter + 8
            }
            REM => {
                self.memory_write(destination_address, &(source1_data % source2_data).to_be_bytes(), size);
                self.program_counter + 8
            }
            CGT => {
                self.memory_write(destination_address, &((source1_data > source2_data) as u64).to_be_bytes(), size);
                self.program_counter + 8
            }
            CLT => {
                self.memory_write(destination_address, &((source1_data < source2_data) as u64).to_be_bytes(), size);
                self.program_counter + 8
            }
            #[rustfmt::skip]
            JMP => {
                source1_address
            },
            JIE => {
                if source2_data == 0 {
                    // Zero
                    self.program_counter + 8
                } else {
                    // Nonzero
                    source1_address
                }
            }
            JNE => {
                if source2_data == 0 {
                    // Zero
                    source1_address
                } else {
                    // Nonzero
                    self.program_counter + 8
                }
            }
            PUT_I => {
                print!("{}", source1_data);
                self.program_counter + 8
            }
            PUT_C => {
                print!("{}", char::from(source1_data as u8));
                self.program_counter + 8
            }
            IMZ => {
                self.memory_write(destination_address, &(self.image_length as u64).to_be_bytes(), size);
                self.program_counter + 8
            }
            EQU => {
                self.memory_write(destination_address, &((source1_data == source2_data) as u64).to_be_bytes(), size);
                self.program_counter + 8
            }
            HLT => {
                self.mode = TransientMode::HALTED;
                self.program_counter
            }
            _ => {
                panic!(
                    "Halt: Unsupported opcode! Instruction: 0x{:0>2x}{:0>4x}{:0>4x}{:0>4x}\n-> Program Counter: {}",
                    opcode, source1_address, source2_address, destination_address, self.program_counter
                );
            }
        }
    }
}

fn u64_pad_be(data: &[u8]) -> [u8; 8] {
    let mut padded = [0u8; 8];
    padded[(8 - data.len())..].copy_from_slice(data);
    padded
}

fn main() {
    // DEBUG: Write a fibonacci program
    let _ROM: [u8; 80] = [
    //  OPCO, SIZE, SRC1, SRC1, SRC2, SRC2, DEST, DEST
        0x0D, 0x08, 0x00, 0064, 0x00, 0x00, 0x00, 0x00,
        0x0E, 0x08, 0x00, 0072, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x08, 0x00, 0048, 0x00, 0056, 0x00, 0064,
        0x01, 0x08, 0x00, 0056, 0x00, 0x00, 0x00, 0048,
        0x01, 0x08, 0x00, 0064, 0x00, 0x00, 0x00, 0056,
        0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A,
    ];
    let mut one_plus_two = File::create("yay-2.0.bin").unwrap();
    one_plus_two.write(&_ROM).unwrap();
    one_plus_two.flush().unwrap();

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
