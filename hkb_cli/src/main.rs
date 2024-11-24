use clap::{Arg, ArgMatches, Command};
use hkb_cli_habit::CommandPlugin as HabitCommandPlugin;
use hkb_core::command::CommandPluggable;

fn main() {
    let commands: Vec<Box<dyn CommandPluggable>> = vec![Box::new(HabitCommandPlugin::new())];
    let mut command = Command::new("hkb")
        .version("0.1.0")
        .author("HKB <no_email>")
        .about("HKB CLI");

    for plugin_command in &commands {
        command = command.subcommand(plugin_command.init());
    }

    let matches = command.get_matches();
    let subcommand = matches.subcommand();

    if let Some(sub_match) = subcommand {
        for plugin_command in &commands {
            if plugin_command.handle_subcommand(sub_match).is_ok() {
                break;
            }
        }
    }
}
