// NOTE: Could be also just a usize which represent flags
pub struct Command {
    pub ls: bool,
}

impl Command {
    fn new() -> Command {
        return Command { ls: false };
    }

    pub fn parse(args: Vec<String>) -> Result<Command, String> {
        let mut command: Command = Command::new();

        if args.len() == 0 {
            return Err("No arguments provided".to_string());
        }

        for arg in args {
            if &arg == "ls" {
                command.ls = true;
            }
        }

        return Ok(command);
    }
}
