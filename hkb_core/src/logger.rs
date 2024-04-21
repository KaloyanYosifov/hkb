use log::LevelFilter;
pub use log::{debug, error, info, log, trace, warn};

use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;

pub fn init() {
    let log_line_pattern = "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}";
    let file_appender = {
        #[cfg(debug_assertions)]
        {
            FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(log_line_pattern)))
                .build("__log/main.log")
                .unwrap()
        }

        #[cfg(not(debug_assertions))]
        {
            let cache_dir = dirs::cache_dir().unwrap();
            // TODO: support rotation logs
            FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(log_line_pattern)))
                .build(cache_dir.join("hkb/main.log"))
                .unwrap()
        }
    };
    let filter_level = {
        #[cfg(debug_assertions)]
        {
            LevelFilter::Debug
        }

        #[cfg(not(debug_assertions))]
        {
            LevelFilter::Info
        }
    };

    let config = Config::builder()
        .appender(Appender::builder().build("main", Box::new(file_appender)))
        .build(Root::builder().appender("main").build(filter_level))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
