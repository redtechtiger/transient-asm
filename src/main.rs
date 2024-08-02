/// Data to be executed by the state machine. Execution starts at offset 0 (first byte) and each
/// instruction is 4 bytes long. (opcode, source1, source2, destination)
///
/// # Opcodes
/// - 0x01: MOV from source1 into destination
/// - 0x02: ADD source1 and source2 and store result in destination
/// - 0x03: SUB source2 from source1 and store result in destination
/// - 0x04: MUL source1 and source2 and store result in destination
/// - 0x05: DIV source1 by source2 and store result in destination (truncated)
/// - 0x06: DIV source1 by source2 and store result in destination (rounded)
/// - 0x07: CGT compare if source1 is greater than source2, and if so, store 1 in destination
/// - 0x08: CLT compare if source1 is less than source2, and if so, store 1 in destination
/// - 0x09: JMP stops current execution and jumps to code in source1
/// - 0x0A: JIE stops current execution and jumps to code in source1 ONLY IF source2 is non-zero
/// - 0x0B: JNE stops current execution and jumps to code in source1 ONLY IF source2 is zero
/// - 0x0C: PUT prints data to the screen (int)
/// - 0x0D: PUT prints data to the screen (char)
/// - 0x0E: XSA gets the length of code in ROM and stores in destination
/// - 0xFF: HLT halts execution and stops processor
///
/// # Source1
/// Any transient address
///
/// # Source2
/// Any transient address
///
/// # Destination
/// Any transient address
///
/// # Transient addresses
/// These can range from 0 up to TRANSIENT_MEM_MAX. Do note, however, that the transient processor will
/// fill the transient memory with program data up to the programs length. To get the length of the
/// program, see opcodes above.
const ROM: &'static [u8] = &[255, 200];

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
    pub fn new() -> Self {
        TransientState {
            memory: [0u8; TRANSIENT_MEM_MAX],
            execution_length: 0,
            program_counter: 0,
            mode: TransientMode::HALTED,
        }
    }
    pub fn load_text(&mut self, offset: usize, text: &[u8]) {
        assert!(self.memory.len() >= text.len(), "text section doesn't fit into transient memory");
        self.memory[offset..text.len()+offset].copy_from_slice(text);
    }
}

fn main() {
    // TODO: Future code for loading data from file
    println!("TRANSIENT: Starting bootstrap");
}
