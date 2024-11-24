use clap::{Arg, ArgMatches, Command};
use hkb_core::command::CommandPluggable;

pub struct CommandPlugin;

impl CommandPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl CommandPluggable for CommandPlugin {
    fn init(&self) -> Command {
        Command::new("habit").about("Manage habits").subcommand(
            Command::new("add")
                .about("Add a new habit")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the habit"),
                )
                .arg(
                    Arg::new("frequency")
                        .required(true)
                        .help("Frequency of the habit (e.g., daily, weekly)"),
                ),
        )
    }

    fn handle_subcommand(&self, subcommand: (&str, &ArgMatches)) -> Result<(), String> {
        let (name, matches) = subcommand;

        if name != "habit" {
            return Err("nope".to_string());
        }

        match matches.subcommand() {
            Some(("add", sub_matches)) => add_habit(sub_matches),
            _ => panic!("habit does not have this command ...."),
        }

        Ok(())
    }
}

fn add_habit(matches: &ArgMatches) {
    println!("nice! {:?}", matches);
}
