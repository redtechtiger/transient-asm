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
use std::io::Read;

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
    pub fn resolve_instruction(&self, base: usize) -> [u8; 4] {
        assert!(
            self.memory.len() > base + 3,
            "Halt: Attempted instruction resolution beyond memory space"
        );
        self.memory[base..][..4].try_into().unwrap()
    }
    /// Executes an instruction and returns the next program counter
    pub fn execute_instruction(&mut self, instruction: [u8; 4]) -> usize {
        // Decodes instruction
        let opcode = instruction[0];
        let source1 = instruction[1] as usize;
        let source2 = instruction[2] as usize;
        let destination = instruction[3] as usize;

        // Validates memory pointers
        assert!(
            self.memory.len() > source1,
            "Halt: Attempt to access memory beyond memory space (source1)"
        );
        assert!(
            self.memory.len() > source2,
            "Halt: Attempt to access memory beyond memory space (source2)"
        );
        assert!(
            self.memory.len() > destination,
            "Halt: Attempt to access memory beyond memory space (destination)"
        );

        match opcode {
            MOV => {
                self.memory[destination] = self.memory[source1];
                self.program_counter + 4
            }
            ADD => {
                self.memory[destination] = self.memory[source1] + self.memory[source2];
                self.program_counter + 4
            }
            SUB => {
                self.memory[destination] = self.memory[source1] - self.memory[source2];
                self.program_counter + 4
            }
            MUL => {
                self.memory[destination] = self.memory[source1] * self.memory[source2];
                self.program_counter + 4
            }
            DIV_T => {
                self.memory[destination] = self.memory[source1] / self.memory[source2];
                self.program_counter + 4
            }
            DIV_R => {
                self.memory[destination] =
                    (self.memory[source1] as f64 / self.memory[source2] as f64) as u8;
                self.program_counter + 4
            }
            REM => {
                self.memory[destination] = (self.memory[source1]) % self.memory[source2];
                self.program_counter + 4
            }
            CGT => {
                self.memory[destination] = (self.memory[source1] > self.memory[source2]) as u8;
                self.program_counter + 4
            }
            CLT => {
                self.memory[destination] = (self.memory[source1] < self.memory[source2]) as u8;
                self.program_counter + 4
            }
            JMP => source1,
            JIE => {
                if self.memory[source2] == 0 {
                    // Zero
                    self.program_counter + 4
                } else {
                    // Nonzero
                    source1
                }
            }
            JNE => {
                if self.memory[source2] == 0 {
                    // Zero
                    source1
                } else {
                    // Nonzero
                    self.program_counter + 4
                }
            }
            PUT_I => {
                print!("{}", self.memory[source1]);
                self.program_counter + 4
            }
            PUT_C => {
                print!("{}", char::from(self.memory[source1]));
                self.program_counter + 4
            }
            IMZ => {
                self.memory[destination] = self.image_length as u8;
                self.program_counter + 4
            }
            EQU => {
                self.memory[destination] = (source1 == source2) as u8;
                self.program_counter + 4
            }
            HLT => {
                self.mode = TransientMode::HALTED;
                self.program_counter
            }
            _ => {
                panic!(
                    "Halt: Unsupported opcode! Instruction: 0x{:0>2x}{:0>2x}{:0>2x}{:0>2x}",
                    opcode, source1, source2, destination
                );
            }
        }
    }
}

fn main() {
    // Verify input arguments
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        println!("Stop: Incorrect amount of arguments!");
        return;
    }

    // Open file for reading
    let mut input_file = match File::open(&args[1]) {
        Ok(x) => x,
        Err(_) => {
            println!("Stop: Failed to open file");
            return;
        }
    };

    // Read bytes into buffer
    let mut transient_image: Vec<u8> = vec![];
    if let Err(_) = input_file.read_to_end(&mut transient_image) {
        println!("Stop: Failed to read file contents");
        return;
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
