use std::collections::HashSet;
use std::fmt::Display;
use std::str::FromStr;

use log::LevelFilter;
pub use log::{debug, error, info, log, trace, warn};

use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::append::Append;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;

#[derive(PartialEq, Eq, Hash)]
pub enum AppenderType {
    FILE,
    STDOUT,
}

impl Display for AppenderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            AppenderType::FILE => "file",
            AppenderType::STDOUT => "stdout",
        }
        .to_owned();

        write!(f, "{}", value)
    }
}

fn init_file_appender(pattern: &str) -> FileAppender {
    #[cfg(debug_assertions)]
    {
        FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(pattern)))
            .build("__log/main.log")
            .unwrap()
    }

    #[cfg(not(debug_assertions))]
    {
        let cache_dir = dirs::cache_dir().unwrap();
        // TODO: support rotation logs
        FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(pattern)))
            .build(cache_dir.join("hkb/main.log"))
            .unwrap()
    }
}

fn init_stdout_appender(pattern: &str) -> ConsoleAppender {
    ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build()
}

pub fn init(appenders: Option<Vec<AppenderType>>) {
    let appenders = appenders
        .unwrap_or_else(|| vec![AppenderType::FILE])
        .into_iter()
        .collect::<HashSet<_>>();

    let log_line_pattern = {
        #[cfg(debug_assertions)]
        {
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — [{t}]: {m}{n}"
        }

        #[cfg(not(debug_assertions))]
        {
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | [{t}]: {m}{n}"
        }
    };
    let default_level = {
        #[cfg(debug_assertions)]
        {
            LevelFilter::Debug
        }

        #[cfg(not(debug_assertions))]
        {
            LevelFilter::Info
        }
    };
    let filter_level: LevelFilter = match std::env::var("HKB_LOG_LEVEL") {
        Ok(level) => LevelFilter::from_str(&level).unwrap_or(default_level),
        Err(_) => default_level,
    };
    let mut config = Config::builder();
    let mut root = Root::builder();

    for appender in appenders {
        let appender_to_build: Box<dyn Append> = match appender {
            AppenderType::FILE => Box::new(init_file_appender(log_line_pattern)),
            AppenderType::STDOUT => Box::new(init_stdout_appender(log_line_pattern)),
        };

        config =
            config.appender(Appender::builder().build(appender.to_string(), appender_to_build));
        root = root.appender(appender.to_string());
    }

    log4rs::init_config(config.build(root.build(filter_level)).unwrap()).unwrap();

    info!("Logger Initialized");
}
