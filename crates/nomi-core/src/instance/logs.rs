pub trait GameLogsWriter: Send + Sync {
    fn write(&self, data: GameLogsEvent);
}

pub struct GameLogsEvent {
    message: String,
}

impl GameLogsEvent {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn into_message(self) -> String {
        self.message
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

/// `GameLogsWriter` that does nothing with provided events.
pub struct IgnoreLogs;

impl GameLogsWriter for IgnoreLogs {
    fn write(&self, _data: GameLogsEvent) {}
}

/// `GameLogsWriter` that prints logs into stdout.
pub struct PrintLogs;

impl GameLogsWriter for PrintLogs {
    fn write(&self, data: GameLogsEvent) {
        println!("{}", data.into_message());
    }
}
