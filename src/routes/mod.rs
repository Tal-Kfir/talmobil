#![allow(unused_imports)]
pub mod get_routes;
pub mod post_routes;
pub mod routes_utils;

use super::ODM;
use super::config;
use super::html_format;

use rocket::serde::{ Deserialize, json::{ Json, Error as JsonError } };
use std::path::{Path, PathBuf};
use rocket::http::{SameSite, CookieJar, Cookie};

use rocket::Route;
pub use rocket_oauth2::{OAuth2, TokenResponse};
pub use routes_utils::*;
use rocket::http::Status;

use rocket::response::{content};
use rocket::State;
use rocket::fs::NamedFile;

use mongodb::Database;

use std::io::prelude::*;

use log::{debug, error, info, trace, warn};

use get_routes::get_routes;
use post_routes::post_routes;

pub fn routes() -> Vec<Route> {
    let mut routes = get_routes();
    routes.append(&mut post_routes());
    routes
}