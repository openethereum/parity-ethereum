use fern::colors::{Color, ColoredLevelConfig};

use log::{LevelFilter};

use std::io;

fn setup_logger(verbosity: u32, log_to_file: bool) -> Result<(), fern::InitError> {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let colors_level = colors_line.clone()
        .info(Color::Green);

    let mut base_config = fern::Dispatch::new();

    base_config = match verbosity {
        0 => {
            base_config
            .level(LevelFilter::Error)
            .level_for("pretty_colored", LevelFilter::Error)
        }
        1 => {
            base_config
            .level(LevelFilter::Warn)
            .level_for("pretty_colored", LevelFilter::Warn)
        },
        2 => {
            base_config
            .level(LevelFilter::Info)
            .level_for("pretty_colored", LevelFilter::Info)
        },
        3 => {
            base_config
            .level(LevelFilter::Debug)
            .level_for("pretty_colored", LevelFilter::Debug)
        },
        _ => {
            base_config
            .level(LevelFilter::Trace)
            .level_for("pretty_colored", LevelFilter::Trace)
        },
    };

    let format_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            // In development users may use the `debug!` or `trace!` log levels and output date and target file name
            // for further information.
            if record.level() == LevelFilter::Debug || record.level() == LevelFilter::Trace {
                out.finish(format_args!(
                    "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                    color_line = format_args!("\x1B[{}m", colors_line.get_color(&record.level()).to_fg_str()),
                    date = chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    level = colors_level.color(record.level()),
                    target = record.target(),
                    message = message
                ))
            // In production (i.e. cargo build -p evmbin --release`, we do not want extra formatting shown in any of
            // the log levels up to the max log level that is `info!`, since we MUST print raw JSON as it is a
            // format that is cross-client compatible. It is used by the Ethereum Foundation to run tests (fuzzers)
            // to check for correctness by parsing and comparing the output.
            } else {
                out.finish(format_args!(
                    "{color_line}{message}\x1B[0m",
                    color_line = format_args!("\x1B[{}m", colors_line.get_color(&record.level()).to_fg_str()),
                    message = message
                ))
            }
        })
        // We must only print to `stdout` (not `stderr`) by default.
        .chain(io::stdout());

    if log_to_file {
        let file_config = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(fern::log_file("output.log")?);

        base_config.chain(file_config).chain(format_config).apply()?;
    } else {
        base_config.chain(format_config).apply()?;
    }

    Ok(())
}

/// Initialisation of the [Log Crate](https://crates.io/crates/log) and [Fern Crate](https://docs.rs/fern/0.5.5/fern/)
///
/// - Choice of log level verbosity from evmbin CLI: error (0), warn (1), info (2), debug (3), or trace (4).
/// - Fallback to default log level that is defined in evmbin/src/main.rs.
/// - Use of logging level macros from highest priority to lowest: `error!`, `warn!`, `info!`, `debug!` and `trace!`.
/// - [Compile time filters](https://docs.rs/log/0.4.1/log/#compile-time-filters) that override the evmbin CLI log levels
/// are configured in evmbin/Cargo.toml. In production max log level is `info!`, whereas in development max is `trace!`.
/// - Output to output.log when log_to_file is true.
pub fn init_logger(pattern: &str, log_to_file: bool) -> () {
    let verbosity: u32 = pattern.parse::<u32>().expect("parsing cannot fail; qed");

    match setup_logger(verbosity, log_to_file) {
        Ok(_) => {
            println!("Success initializing logger. Verbosity: {:?}. Log to file: {}", verbosity, &log_to_file); ()
        }
        Err(e) => { println!("Error initializing logger: {}", e); }
    }
}
