use crate::manifest::{CommandSpec, command_specs};

#[derive(Clone)]
pub struct CommandRegistry {
    commands: Vec<CommandSpec>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: command_specs(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&CommandSpec> {
        self.commands.iter().find(|command| command.name == name)
    }
}
