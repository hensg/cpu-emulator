pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub(crate) struct Screen {
    pub display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Screen {
    pub(crate) fn clear(&mut self) {
        self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }
}
