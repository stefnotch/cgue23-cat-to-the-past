use std::{collections::VecDeque, time::Instant};

pub struct TimeManager {
    /// To limit the size of this, we could either
    /// - have a countdown for every level
    /// - only save actual changes, so when the user is AFK, we don't save anything
    /// - have a max size and remove the oldest commands,
    ///   this is especially useful when it's always possible to restart the level simply by walking back to the beginning
    commands: VecDeque<GameChanges>,
    current_frame_commands: GameChanges,
}

/// All game changes in one frame
struct GameChanges {
    timestamp: Instant,
    commands: Vec<Box<dyn GameChange>>,
}

pub trait GameChange {}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
            current_frame_commands: GameChanges {
                timestamp: Instant::now(),
                commands: Vec::new(),
            },
        }
    }

    pub fn start_frame(&mut self) {
        self.current_frame_commands.timestamp = Instant::now();
    }

    pub fn end_frame(&mut self) {
        // Swap the current frame commands with an empty one
        let current_commands = std::mem::replace(
            &mut self.current_frame_commands,
            GameChanges {
                timestamp: Instant::now(),
                commands: Vec::new(),
            },
        );

        // Only if any commands were added, add them to the queue
        if current_commands.commands.len() > 0 {
            self.commands.push_back(current_commands);
        }
    }

    pub fn add_command(&mut self, command: Box<dyn GameChange>) {
        self.current_frame_commands.commands.push(command);
    }

    // TODO:
    // Pop
    // Peek
    // Apply game changes
    // - kinematic character controller
    // - ...
}
