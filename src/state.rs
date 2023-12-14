pub(crate) struct ProgramState {
    running: bool
}

impl ProgramState {
    pub fn new() -> Self {
        ProgramState {
            running: false,
        }
    }

    pub(crate) fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    pub(crate) fn get_running(&self) -> bool {
        return self.running;
    }
}
