use cli::args::ArgsError;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "parity/cli/config"]
struct Config;

pub fn get_config(config_name: &str) -> Result<String, ArgsError> {
    Config::get(config_name)
        .ok_or_else(|| "does not exist".to_owned())
        // .and_then(|x| srt(x).map_err(|e| e.to_owned()))
        .and_then(|x| String::from_utf8(x.to_vec()).map_err(|e| e.to_string()))
        .map_err(|e| ArgsError::ConfigReadError(format!(
                    "Failure to read config file {}: {}",
                    config_name, e
        )))
}
