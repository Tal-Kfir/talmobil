use super::*;
use config::config::ConfyConfig;
use bson::DateTime;
use rocket::request::{Request, FromRequest, Outcome};
use ODM::models::*;

// Basic allowed files ( no need for user auth )
pub const ALLOWED_FILES: [&'static str; 6] = ["index.html", "index.css", "talmobil.png", "favicon.ico", "index_styles.css", "index_styles.css.map"];

// Models for Input Check and Login Check
#[derive(Deserialize, Debug)]
pub struct DriveDoc {
    pub startTime: i64,
    pub endTime: i64,
    pub length: i32,
    pub fule: f32,
    pub startLocation: (f32,f32),
    pub endLocation: (f32,f32),
    pub passengers: i32,
}

// Models for Input Check and Login Check
#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct LocationStruct {
    pub location_del: (f32,f32),
}

use log::info;

// Checking that a user is connected
#[rocket::async_trait]
impl<'r> FromRequest<'r> for GoogleUserInfo {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<GoogleUserInfo, ()> {
        let cookies = request.guard::<&'r CookieJar<'r>>().await.unwrap();

        let route = match request.route() {
            None => format!("Unknown Route"),
            Some(value) => format!("{}", value),
        };
        let method = request.method();
        if let Some(value) = cookies.get_private("token") {

            if let Some(name_cookie) = cookies.get_private("username") {
                
                if let Some(email_cookie) = cookies.get_private("email") {
                    info!("{}", format!("Client>>Server:\t{} is trying to access route: {} as {}", &email_cookie, route, method));
                    
                    return Outcome::Success(GoogleUserInfo {
                        name: name_cookie.value().to_string(),
                        email: email_cookie.value().to_string(),
                    });

                }
            }
        }

        Outcome::Forward(())
    }
}


fn is_numeric<T: Into<String>>(value: T) -> bool {
    let as_string : String = value.into();
    match as_string.parse::<i32>() {
        Ok(_) => return true,
        Err(_) => return false,
    }
}



/// 
///Internal file system export  
///
///INPUT:  the path to the file
///OUTPUT: the actual file
///
pub async fn get_file_internal(file: PathBuf)-> Option<NamedFile> {
    let extension_string = &file
    .extension()
    .unwrap_or_default()
    .to_str()
    .unwrap_or_default()[..];

    let adder = match extension_string {
    "css" | "scss" | "map" => String::from("resources/") + file.file_stem().unwrap().to_str().unwrap(),
    "ico" => String::from("assets/"),
    "jpeg" | "png" | "jpg" => String::from("assets/"),
    "js" => String::from("scripts/"),
    "html" | "htm" | _ => return None,
    };
    NamedFile::open(Path::new(&adder).join(file)).await.ok()
}

/// 
///Html file deliver, needed so that users wont be able to switch pages as they want   
///
///INPUT:  the file path
///OUTPUT: the raw html file
///
pub async fn get_html_file(file: PathBuf) -> Option<NamedFile> {
    let path = Path::new("resources").join(&file.file_stem().unwrap());
    let path = path.join(&file);
    NamedFile::open(path).await.ok()
}



/// 
///Has access - checks the user access to the file 
///
///INPUT:  user verification, file path and DB access
///OUTPUT: boolean, suitable or not
///
pub async fn has_access(user_email : String, file: String, db: &Database, check_numeric: bool) -> mongodb::error::Result<bool> {
    if check_numeric {
        if !is_numeric(&file) {
            return Ok(false);
        }
    }
    
    let value = file.parse::<i64>();
    if let Ok(inner) = value {
        let cars = ODM::odm::get_cars_for_user(&db, ODM::odm::get_id_by_username(&db, user_email.clone()).await?.unwrap_or("-1".to_string())).await?; //Check if NONE
        return Ok(cars.iter().any(|i| i==&file));
    }
    Ok(true)
}


/// 
///Util  
///
///INPUT:  ride and userID
///OUTPUT: the Drive struct obj
///
pub fn drive_to_doc(drive: Json<DriveDoc>, userID: String) -> Drive {
    return Drive {
        driverID:       userID,
        startTime:      DateTime::from_millis(drive.startTime),
        endTime:        DateTime::from_millis(drive.endTime),
        endLocation:    vec![drive.endLocation.0, drive.endLocation.1],
        startLocation:  vec![drive.startLocation.0, drive.startLocation.1],
        fule:           drive.fule,
        length:         drive.length,
        passengers:     drive.passengers,
    };
}

// Utils struct for rocket::manage
pub struct MongoState {
    pub db: mongodb::Database,
}

pub struct Config {
    pub config: ConfyConfig,
}

pub struct Google;

#[derive(Deserialize, Debug)]
pub struct GoogleUserInfo {
    #[serde(default)]
    pub name: String,
    pub email: String,
}