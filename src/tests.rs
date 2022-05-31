
use log::{warn,error};
use rocket::{Rocket,Build};
// TALHKF10 login's
const DEMO_TOKEN_COOKIE: &str = "YaZtEo4rer8RM/YLD2Y8jIfNFD7vkLiYxX+OWRorHEYNVYtHMRgcKein14c2sfJCB2cuuOfyi+A8hIx/X/WM66F2UhykdfGfAIUe2YuTp1aDO23Kj2xZ2LoNC1ueDx0hGqXBbYfG01pDxKNRh790AxOCa076HHDWe3z0uBWGL6iUHTkV+UPX1ZvxtJzCZILmrBmSFwrR6CyBJMXtRXdXG2wtFfwd1rpYSsfuT9ZLCoK237k9K+LCC9QjGQgwBx3zHw==";
const DEMO_EMAIL_COOKIE: &str = "2n/Xk/BWjvW9mqPc2XAeElhtCAZcAWby9xHwDi7QtL05S/NUq2uMHN18B/nELw==";
const DEMO_UN_COOKIE: &str = "d9jmnNfo/gAhKfPZ40dGBwBlPI7dV8wKi3Kjpvg2xQ==";
const TEST_CAR : &str = "/2";
use super::*;

async fn redo_main() ->Rocket<Build> {
    let config = config::config::init().await;
    let config = match config {
        Err(e) => {
            panic!("error: {}", format!("CONFIG failed to launch {}", e));
        },
        Ok(value) => value,
    };
    let _ = config::config::setup_logger(&config).await;
    let db = match ODM::odm::init(&config).await {
        Err(val) =>  {
            panic!("error: {}", val);
        },
        Ok(value) => value,
    };
    warn!("TEST LAUNCH");

    let lift = super::rocket::build()
    .mount("/", routes::routes())
    .attach(OAuth2::<Google>::fairing("google"))
    .manage(MongoState { db })
    .manage(Config { config } );

    lift
}

use rocket::http::Status;
use rocket::local::asynchronous::Client;
use rocket::http::Cookie;
use rocket::http::ContentType;

#[rocket::async_test]
async fn test_basic() {
    let beta = redo_main().await;

    let client = Client::tracked(beta).await.unwrap();
    let req = client.get("/")
    .dispatch().await;
    

}

#[rocket::async_test]
async fn test_cookies() {
    let beta = redo_main().await;

    let client = Client::tracked(beta).await.unwrap();
    let req = client.get("/home")
    .cookies(vec![ Cookie::new("token", DEMO_TOKEN_COOKIE),
                   Cookie::new("email", DEMO_EMAIL_COOKIE),
                   Cookie::new("username", DEMO_UN_COOKIE)])
    .dispatch().await;
    
    let body = &req.into_string().await.unwrap();
    let success = body.contains("Home");
    match success {
        true => {},
        false => {
            panic!("Response did not go through");
        }
    }
}

#[rocket::async_test]
async fn test_get_route() {
    let beta = redo_main().await;

    let client = Client::tracked(beta).await.unwrap();
    let req = client.get(format!("/calendar{}", TEST_CAR))
    .cookies(vec![ Cookie::new("token", DEMO_TOKEN_COOKIE),
                   Cookie::new("email", DEMO_EMAIL_COOKIE),
                   Cookie::new("username", DEMO_UN_COOKIE)])
    .dispatch().await;
    
    let body = &req.into_string().await.unwrap();
    let success = body.contains("Calendar");
    match success {
        true => {},
        false => {
            panic!("Response did not go through");
        }
    }
}

#[rocket::async_test]
async fn test_post_route() {
    let beta = redo_main().await;

    let client = Client::tracked(beta).await.unwrap();
    let mut req = client.post(format!("/locations{}/delete", TEST_CAR))
    .cookies(vec![ Cookie::new("token", DEMO_TOKEN_COOKIE),
                   Cookie::new("email", DEMO_EMAIL_COOKIE),
                   Cookie::new("username", DEMO_UN_COOKIE)])
    .header(ContentType::JSON);
    req.set_body(r#"{ "location_del": [35.31280517578125,32.07854461669922] }"#);
    let response = req.dispatch().await;
    
    let stat = response.status();
    assert_eq!(stat, Status::NoContent);
}