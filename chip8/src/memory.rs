use crate::font::{FONTSET, FONTSET_SIZE};

pub(crate) const RAM_SIZE: usize = 4096;
pub(crate) const START_ADDR: u16 = 0x200;

pub(crate) const STACK_SIZE: usize = 16;

/// The stack for the subroutines
pub(crate) struct Stack {
    stack_point: u16, // index in the 'stack' as we are using raw arrays
    stack: [u16; STACK_SIZE],
}

impl Stack {
    pub(crate) fn push(&mut self, value: u16) {
        self.stack[self.stack_point as usize] = value;
        self.stack_point += 1;
    }

    pub(crate) fn pop(&mut self) -> u16 {
        self.stack_point -= 1;
        self.stack[self.stack_point as usize]
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            stack_point: 0,
            stack: [0; STACK_SIZE],
        }
    }
}

pub(crate) struct Ram {
    data: [u8; RAM_SIZE],
}

impl Ram {
    /// Fetches a 2-byte instruction from the RAM at the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The starting address of the 2-byte instruction.
    ///
    /// # Returns
    ///
    /// A 2-byte instruction (u16) fetched from the RAM that is [u8; 4096].
    ///
    /// # Example
    ///
    /// ```
    /// let instruction = ram.fetch(0x200);
    /// ```
    pub(crate) fn fetch_instruction(&self, address: usize) -> u16 {
        let higher_byte = self.data[address] as u16;
        let lower_byte = self.data[address + 1] as u16;
        // big endian
        let op = (higher_byte << 8) | lower_byte;
        op
    }

    pub(crate) fn fetch_byte(&self, address: usize) -> u8 {
        self.data[address]
    }

    pub(crate) fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.data[start..end].copy_from_slice(data);
    }

    pub(crate) fn write_byte(&mut self, address: usize, value: u8) {
        self.data[address] = value;
    }
}

impl Default for Ram {
    fn default() -> Self {
        let mut ram = Self {
            data: [0; RAM_SIZE],
        };
        ram.data[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        ram
    }
}
