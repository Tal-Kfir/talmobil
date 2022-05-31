//!
//! Documentation of the config module.
//! Sets up the 'config' and 'logger'.
//! 



extern crate confy;

use serde::{Serialize, Deserialize};
use std::default::Default;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfyConfig {
    pub print_log: bool,
    pub log_file: String,
    pub database: String,
    pub database_ip: String,
    pub timeout: u64,
    pub file_home: String,
    pub file_add_car: String,
    pub file_calendar: String,
    pub file_car_edit: String,
    pub file_car_services: String,
    pub file_car_overview: String,
    pub file_car_view: String,
    pub file_drive: String,
    pub file_locations: String,
}

///Config check
impl Default for ConfyConfig {
    fn default() -> Self {
        ConfyConfig {
            print_log: false,
            log_file: "output.log".to_string(),
            database: "talmobil".to_string(),
            database_ip: "mongodb://localhost:27017/".to_string(),
            timeout: 2,
            file_home: "home.html".to_string(),
            file_add_car: "add-car.html".to_string(),
            file_calendar: "calendar.html".to_string(),
            file_car_edit: "edit-car.html".to_string(),
            file_car_services: "car-services.html".to_string(),
            file_car_overview: "car-overview.html".to_string(),
            file_car_view: "car-view.html".to_string(),
            file_drive: "drive.html".to_string(),
            file_locations: "locations.html".to_string(),
        }
    }
}

/// Initialize config and load
pub async fn init() -> Result<ConfyConfig, confy::ConfyError> {
    let cfg: ConfyConfig = confy::load_path("talmobil.toml").unwrap_or_default();
    Ok(cfg)
}

/// Sets up logger
pub async fn setup_logger(file: &ConfyConfig) -> Result<(), fern::InitError> {
    if file.print_log {
        fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(&file.log_file)?)
        .apply()?;
    }

    else {
        fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(&file.log_file)?)
        .apply()?;
    }

    Ok(())
}