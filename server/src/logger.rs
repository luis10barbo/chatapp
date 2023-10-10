use std::error::Error;
use std::io::Write;

use chrono::Local;
use env_logger::Builder;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};

pub fn setup_env_logger() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-d%T%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Info)
        .init();
}

pub fn setup_logger() -> Result<(), Box<dyn Error>> {
    // setup_env_logger();
    let log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S %Z)} [{l}] - {m} {n}",
        )))
        .build("log/output.log")?;

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%H:%M:%S)} [{l}] {t} - {m} {n}",
        )))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("log_file", Box::new(log_file)))
        .appender(Appender::builder().build("stdout", Box::new(console_appender)))
        .logger(Logger::builder().build("stdout", log::LevelFilter::Info))
        .build(
            Root::builder()
                .appender("log_file")
                .appender("stdout")
                .build(log::LevelFilter::Info),
        )?;

    log4rs::init_config(config)?;
    log::info!("Hello World!");

    Ok(())
}
