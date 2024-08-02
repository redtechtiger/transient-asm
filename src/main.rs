//! Transient is, in essence, a custom virtual machine and file format. The transient processor
//! loads a transient "image' into the virtual address space and begins execution at offset 0x00.
//!
//!
//! # Opcodes
//! - 0x01: MOV from source1 into destination
//! - 0x02: ADD source1 and source2 and store result in destination
//! - 0x03: SUB source2 from source1 and store result in destination
//! - 0x04: MUL source1 and source2 and store result in destination
//! - 0x05: DIV source1 by source2 and store result in destination (truncated)
//! - 0x06: DIV source1 by source2 and store result in destination (rounded)
//! - 0x07: CGT compare if source1 is greater than source2, and if so, store 1 in destination
//! - 0x08: CLT compare if source1 is less than source2, and if so, store 1 in destination
//! - 0x09: JMP stops current execution and jumps to code in source1
//! - 0x0A: JIE stops current execution and jumps to code in source1 ONLY IF source2 is non-zero
//! - 0x0B: JNE stops current execution and jumps to code in source1 ONLY IF source2 is zero
//! - 0x0C: PUT prints data to the screen (int)
//! - 0x0D: PUT prints data to the screen (char)
//! - 0x0E: XSA gets the length of code in ROM and stores in destination
//! - 0xFF: HLT halts execution and stops processor
//!
//! # Transient addresses
//! Source1, source2, and destination are transient addresses. These can range from 0 up to TRANSIENT_MEM_MAX. Do note, however, that the transient processor will
//! fill the transient memory with program data up to the programs length. To get the length of the
//! program, see opcodes above.

use std::env::args;
use std::fs::File;
use std::io::Read;

const TRANSIENT_MEM_MAX: usize = 40_000;

pub enum TransientMode {
    RUNNING,
    HALTED,
}

pub struct TransientState<const TRANSIENT_MEM_MAX: usize> {
    pub memory: [u8; TRANSIENT_MEM_MAX],
    pub execution_length: usize, // Length of executable code in memory
    pub program_counter: usize,
    pub mode: TransientMode,
}

impl<const TRANSIENT_MEM_MAX: usize> TransientState<TRANSIENT_MEM_MAX> {
    /// Initialize a new, empty instance of a transient processor/state with a transient memory
    /// size of TRANSIENT_MEM_MAX bytes.
    pub fn new() -> Self {
        TransientState {
            memory: [0u8; TRANSIENT_MEM_MAX],
            execution_length: 0,
            program_counter: 0,
            mode: TransientMode::HALTED,
        }
    }
    /// Loads a transient memory image into a state/processor at a specified offset.
    pub fn load_text(&mut self, offset: usize, text: &[u8]) {
        assert!(
            self.memory.len() >= text.len(),
            "text section doesn't fit into transient memory"
        );
        self.memory[offset..text.len() + offset].copy_from_slice(text);
    }
    /// Starts a loop that runs the processor until halted
    pub fn run(&mut self, start: usize) {
        self.program_counter = start;
        self.mode = TransientMode::RUNNING;
        
    }
    /// Fetches an instruction at the given address (mind the alignment!)
    pub fn resolve_instruction(&self, base: usize) -> [u8; 4]{
        assert!(
            self.memory.len() > base+3,
            "attempted instruction resolution beyond memory space"
        );
        self.memory[base..][..4].try_into().unwrap()
    }
}

fn main() {
    // Verify input arguments
    let args: Vec<String> = args().collect();
    if args.len() != 1 {
        println!("Stop: Incorrect amount of arguments!");
        return;
    }

    // Open file for reading
    let mut input_file = match File::open(&args[0]) {
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
    transient_state.load_text(0, &transient_image);
    println!("Info: Transient image loaded");

    // Begin executing
    transient_state.run(0);
}
