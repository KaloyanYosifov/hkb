use clap::{ArgMatches, Command};

pub trait CommandPluggable {
    fn init(&self) -> Command;

    fn handle_subcommand(&self, subcommand: (&str, &ArgMatches)) -> Result<(), String>;
}
