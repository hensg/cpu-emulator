use rand::random;

mod font;
mod memory;
pub mod screen;

use memory::{Ram, Stack};
use screen::Screen;

const NUM_REGS: usize = 16;

const NUM_KEYS: usize = 16;

pub struct CPU {
    // index of the current instruction, to know where the
    // program is currently executing in ram memory
    program_counter: u16,

    // 16 registers (1byte), from V0 to VF
    v_registers: [u8; NUM_REGS],

    // indexing into ram for reads and writes
    i_register: u16,

    /// The stack for the subroutines
    stack: Stack,

    // where the game program will be loaded, read/write
    ram: Ram,

    screen: Screen,
    // the keyboard keys
    keys: [bool; NUM_KEYS],

    // timer registers
    delay_timer: u8, // executes something uppon hitting 0
    sound_timer: u8, // emit a sound uppon hitting 0
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            program_counter: memory::START_ADDR,
            v_registers: [0; NUM_REGS],
            i_register: 0,
            stack: Stack::default(),
            ram: Ram::default(),
            screen: Screen::default(),
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

impl CPU {
    fn fetch(&mut self) -> u16 {
        let instruction = self.ram.fetch_instruction(self.program_counter as usize);
        self.program_counter += 2;
        instruction
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    pub fn tick(&mut self) {
        let instruction = self.fetch();
        self.execute(instruction);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen.display
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        self.ram.load(data);
    }

    // 00E0 - CLS: Clear the display.
    // 00EE - RET: Return from a subroutine.
    // 1NNN - JP addr: Jump to address NNN.
    // 2NNN - CALL addr: Call subroutine at NNN.
    // 3XNN - SE Vx, byte: Skip next instruction if Vx == NN.
    // 4XNN - SNE Vx, byte: Skip next instruction if Vx != NN.
    // 5XY0 - SE Vx, Vy: Skip next instruction if Vx == Vy.
    // 6XNN - LD Vx, byte: Set Vx = NN.
    // 7XNN - ADD Vx, byte: Set Vx = Vx + NN.
    // 8XY0 - LD Vx, Vy: Set Vx = Vy.
    // 8XY1 - OR Vx, Vy: Set Vx = Vx OR Vy.
    // 8XY2 - AND Vx, Vy: Set Vx = Vx AND Vy.
    // 8XY3 - XOR Vx, Vy: Set Vx = Vx XOR Vy.
    // 8XY4 - ADD Vx, Vy: Set Vx = Vx + Vy, set VF = carry.
    // 8XY5 - SUB Vx, Vy: Set Vx = Vx - Vy, set VF = NOT borrow.
    // 8XY6 - SHR Vx: Set Vx = Vx SHR 1.
    // 8XY7 - SUBN Vx, Vy: Set Vx = Vy - Vx, set VF = NOT borrow.
    // 8XYE - SHL Vx: Set Vx = Vx SHL 1.
    // 9XY0 - SNE Vx, Vy: Skip next instruction if Vx != Vy.
    // ANNN - LD I, addr: Set I = NNN.
    // BNNN - JP V0, addr: Jump to address NNN + V0.
    // CXNN - RND Vx, byte: Set Vx = random byte AND NN.
    // DXYN - DRW Vx, Vy, nibble: Display n-byte sprite at memory location I at (Vx, Vy), set VF = collision.
    // EX9E - SKP Vx: Skip next instruction if key with the value of Vx is pressed.
    // EXA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed.
    // FX07 - LD Vx, DT: Set Vx = delay timer value.
    // FX0A - LD Vx, K: Wait for a key press, store the value of the key in Vx.
    // FX15 - LD DT, Vx: Set delay timer = Vx.
    // FX18 - LD ST, Vx: Set sound timer = Vx.
    // FX1E - ADD I, Vx: Set I = I + Vx.
    // FX29 - LD F, Vx: Set I = location of sprite for digit Vx.
    // FX33 - LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // FX55 - LD [I], Vx: Store registers V0 through Vx in memory starting at location I.
    // FX65 - LD Vx, [I]: Read registers V0 through Vx from memory starting at location I.
    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => (),
            (0, 0, 0xE, 0) => {
                // clear screen
                self.screen.clear();
            }
            (0, 0, 0xE, 0xE) => {
                // return
                let ret_addr = self.stack.pop();
                self.program_counter = ret_addr;
            }
            (1, _, _, _) => {
                // jump nnn
                let nnn = op & 0xFFF;
                self.program_counter = nnn;
            }
            (2, _, _, _) => {
                // call nnn
                let nnn = op & 0xFFF;
                self.stack.push(self.program_counter);
                self.program_counter = nnn;
            }
            (3, _, _, _) => {
                // skip vx == nn
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_registers[x] == nn {
                    self.program_counter += 2;
                }
            }
            (4, _, _, _) => {
                // skip vx != nn
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_registers[x] != nn {
                    self.program_counter += 2;
                }
            }
            (5, _, _, 0) => {
                // skip vx == vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] == self.v_registers[y] {
                    self.program_counter += 2;
                }
            }
            (6, _, _, _) => {
                // set vx = nn
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_registers[x] = nn;
            }
            (7, _, _, _) => {
                // set vx += nn
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_registers[x] = self.v_registers[x].wrapping_add(nn);
            }
            (8, _, _, 0) => {
                // set vx = vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[y];
            }
            (8, _, _, 1) => {
                // set vx |= vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] |= self.v_registers[y];
            }
            (8, _, _, 2) => {
                // set vx &= vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] &= self.v_registers[y];
            }
            (8, _, _, 3) => {
                // set vx ^= vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] ^= self.v_registers[y];
            }
            (8, _, _, 4) => {
                // set vx += vy, set vf = carry
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (res, overflow) = self.v_registers[x].overflowing_add(self.v_registers[y]);
                self.v_registers[x] = res;
                self.v_registers[0xF] = if overflow { 1 } else { 0 };
            }
            (8, _, _, 5) => {
                // set vx -= vy, set vf = !borrow
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (res, overflow) = self.v_registers[x].overflowing_sub(self.v_registers[y]);
                self.v_registers[x] = res;
                self.v_registers[0xF] = if overflow { 0 } else { 1 };
            }
            (8, _, _, 6) => {
                // set vx >>= 1, set vf = lsb
                let x = digit2 as usize;
                self.v_registers[0xF] = self.v_registers[x] & 0x1;
                self.v_registers[x] >>= 1;
            }
            (8, _, _, 7) => {
                // set vx = vy - vx, set vf = !borrow
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (res, overflow) = self.v_registers[y].overflowing_sub(self.v_registers[x]);
                self.v_registers[x] = res;
                self.v_registers[0xF] = if overflow { 0 } else { 1 };
            }
            (8, _, _, 0xE) => {
                // set vx <<= 1, set vf = msb
                let x = digit2 as usize;
                self.v_registers[0xF] = (self.v_registers[x] & 0x80) >> 7;
                self.v_registers[x] <<= 1;
            }
            (9, _, _, 0) => {
                // skip vx != vy
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] != self.v_registers[y] {
                    self.program_counter += 2;
                }
            }
            (0xA, _, _, _) => {
                // set i = nnn
                let nnn = op & 0xFFF;
                self.i_register = nnn;
            }
            (0xB, _, _, _) => {
                // jump nnn + v0
                let nnn = op & 0xFFF;
                self.program_counter = nnn + self.v_registers[0] as u16;
            }
            (0xC, _, _, _) => {
                // set vx = rand() & nn
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rand_byte = random::<u8>();
                self.v_registers[x] = rand_byte & nn;
            }
            (0xD, _, _, _) => {
                // opcode Dxyn: Draw a sprite at coordinate (Vx, Vy) with a height of n pixels.
                // The sprite is located in memory at the address stored in the I register.

                // Extract x, y, and n from the opcode
                let x = digit2 as usize;
                let y = digit3 as usize;
                let n = digit4 as usize;

                // Get the x and y coordinates from the V registers
                let vx = self.v_registers[x] as usize;
                let vy = self.v_registers[y] as usize;

                // Reset the collision flag
                self.v_registers[0xF] = 0;

                // Loop over each row of the sprite
                for row in 0..n {
                    // Fetch the sprite byte from memory
                    let sprite = self.ram.fetch_byte((self.i_register + row as u16) as usize);

                    // Loop over each bit in the sprite byte
                    for col in 0..8 {
                        // Extract the bit value (0 or 1)
                        let bit = (sprite >> (7 - col)) & 1;

                        // Calculate the screen index, wrapping around screen dimensions
                        let idx = (vx + col) % screen::SCREEN_WIDTH
                            + ((vy + row) % screen::SCREEN_HEIGHT) * screen::SCREEN_WIDTH;

                        // Get the current bit on the screen
                        let prev_bit = self.screen.display[idx];

                        // XOR the screen bit with the sprite bit (draw the sprite)
                        self.screen.display[idx] ^= bit == 1;

                        // Check for collision (if a bit was set and is now unset)
                        if prev_bit && !self.screen.display[idx] {
                            // Set the collision flag
                            self.v_registers[0xF] = 1;
                        }
                    }
                }
            }
            (0xE, _, 9, 0xE) => {
                // skip key press
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let key = self.keys[vx as usize];
                if key {
                    self.program_counter += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                // skip key release
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.program_counter += 2;
                }
            }
            (0xF, _, 0, 7) => {
                // vx = delay timer
                let x = digit2 as usize;
                self.v_registers[x] = self.delay_timer;
            }
            (0xF, _, 0, 0xA) => {
                // wait for key press and store the key value in Vx
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_registers[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // Repeat the current opcode if no key is pressed
                    self.program_counter -= 2;
                }
            }
            (0xF, _, 1, 5) => {
                // delay_timer = vx
                let x = digit2 as usize;
                self.delay_timer = self.v_registers[x];
            }
            (0xF, _, 1, 8) => {
                // sound_timer = vx
                let x = digit2 as usize;
                self.sound_timer = self.v_registers[x];
            }
            (0xF, _, 1, 0xE) => {
                // i register += vx
                let x = digit2 as usize;
                let vx = self.v_registers[x] as u16;
                self.i_register = self.i_register.wrapping_add(vx);
            }
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_registers[x] as u16;
                self.i_register = c * 5;
                // starting memory address of the sprite for that character.
                // this is because the sprites are stored sequentially in memory,
                // and each sprite occupies 5 bytes.
            }
            (0xF, x, 3, 3) => {
                // retrieve the value from register vx
                // we need the value in vx to convert it to its binary-coded decimal (bcd) representation
                let value = self.v_registers[x as usize];

                // store the hundreds digit of the value at memory address i
                // the bcd representation requires splitting the value into hundreds, tens, and units
                self.ram.write_byte(self.i_register as usize, value / 100);

                // store the tens digit of the value at memory address i+1
                // this ensures the correct bcd representation is stored in consecutive memory locations
                self.ram
                    .write_byte((self.i_register + 1) as usize, (value / 10) % 10);

                // store the units digit of the value at memory address i+2
                // storing the units completes the bcd representation in memory
                self.ram
                    .write_byte((self.i_register + 2) as usize, value % 10);
            }
            (0xF, x, 5, 5) => {
                // store the values of registers v0 to vx in memory starting at address i
                let i = self.i_register as usize;
                for idx in 0..=x as usize {
                    self.ram.write_byte(i + idx, self.v_registers[idx]);
                }
            }
            (0xF, x, 6, 5) => {
                // load v0 - vx
                let i = self.i_register as usize;
                for idx in 0..=x as usize {
                    self.v_registers[idx] = self.ram.fetch_byte(i + idx);
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {op}"),
        }
    }
}
