//!
//! #  TalMobil - Tal Kfir's final high school project
//! 
//! We did it!
//! 
//! TalMobil is a web application, built to help co-owning a shared vehicle.
//! 
//! It is comfortable, easy on the eyes and contains multiple features that
//! can help any individual who wishes to co-own a car, such as:
//! 
//! * GPS tracking
//! * Calendar for past car events
//! * Specific detail editing
//! * Location save and modify
//! * Payment services and overview
//! 


#![doc(html_logo_url = "https://img001.prntscr.com/file/img001/sKs1RnNHSY6hPUZKcIRrlA.png",
       html_favicon_url = "https://img001.prntscr.com/file/img001/sKs1RnNHSY6hPUZKcIRrlA.png")]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#[macro_use] extern crate rocket;
extern crate rocket_multipart_form_data;

mod routes;
mod html_format;
mod ODM;
mod config;

use routes::{OAuth2, Google, MongoState, Config};
use log::{debug, error, info, trace, warn};

#[cfg(test)] mod tests;

/// The main functions, runs w/ cargo run
#[rocket::main]
async fn main() -> Result<(), ()> {
    let config = config::config::init().await;
    let config = match config {
        Err(e) => {
            println!("{}", format!("CONFIG failed to launch {}", e));
            return Ok(());
        },
        Ok(value) => value,
    };
    let _ = config::config::setup_logger(&config).await;
    let db = match ODM::odm::init(&config).await {
        Err(val) =>  {
            return Ok(());
        },
        Ok(value) => value,
    };
    warn!("TALMOBIL IS LAUNCHING");

    let lift = rocket::build()
    .mount("/", routes::routes())
    .attach(OAuth2::<Google>::fairing("google"))
    .manage(MongoState { db })
    .manage(Config { config } )
    .launch()
    .await;

    warn!("TALMOBIL OVER");
    match lift {
        Ok(value) => return Ok(()),
        Err(value) => {
            error!("Rocket could not run, error {}", value);
            return Ok(());
        }
    }
}