use super::{*};
use routes_utils::*;
use anyhow::{Context,Error};
use hyper_sync_rustls;
use rocket::response::{Debug, Redirect};

use hyper::{
    header::{Authorization, UserAgent},
    net::HttpsConnector,
    Client,
};
use log::{debug, error, info, trace, warn};

// This route calls `get_redirect`, which sets up a token request and
// returns a `Redirect` to the authorization endpoint.
#[get("/login/google")]
async fn google_login(oauth2: OAuth2<Google>, cookies: &CookieJar<'_>) -> Redirect {
    // We want the "user:read" scope. For some providers, scopes may be
    // pre-selected or restricted during application registration. We could
    // use `&[]` instead to not request any scopes, but usually scopes
    // should be requested during registation, in the redirect, or both.
    oauth2.get_redirect(cookies, &["https://www.googleapis.com/auth/userinfo.profile", "https://www.googleapis.com/auth/userinfo.email"]).unwrap()
}

// This route, mounted at the application's Redirect URI, uses the
// `TokenResponse` request guard to complete the token exchange and obtain
// the token.
#[get("/auth/google")]
async fn google_callback(token: TokenResponse<Google>, cookies: &CookieJar<'_>) -> Result<Redirect, Debug<Error>> 
{
    let https = HttpsConnector::new(hyper_sync_rustls::TlsClient::new());
    let client = Client::with_connector(https);
    
    let response = client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .header(Authorization(format!("Bearer {}", token.access_token())))
        .header(UserAgent("TalMobil".into()))
        .send()
        .context("failed to send request to API")?;

    if !response.status.is_success() {
        return Err(anyhow::anyhow!(
            "got non-success status {}",
            response.status
        ))?;
    }
    

    let user_info : GoogleUserInfo = serde_json::from_reader(response)
        .context("failed to deserialize response")?;


    // Set a private cookie with the access token
    cookies.add_private(
        Cookie::build("token", token.access_token().to_string())
            .same_site(SameSite::Lax)
            .finish()
    );
    
    cookies.add_private(
        Cookie::build("email", user_info.email)
            .same_site(SameSite::Lax)
            .finish(),
    );

    cookies.add_private(
        Cookie::build("username", user_info.name)
            .same_site(SameSite::Lax)
            .finish(),
    );

    
    Ok(Redirect::to("/home"))
}


/// 
///Logout  
///
///INPUT:  user's cookies
///OUTPUT: redirect to home page
///
#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Redirect{
    
    if let Some(cookie) = cookies.get_private("token") {
        cookies.remove_private(Cookie::named("token"));
    }
    if let Some(cookie) = cookies.get_private("email") {
        cookies.remove_private(Cookie::named("email"));
    }
    if let Some(cookie) = cookies.get_private("username") {
        cookies.remove_private(Cookie::named("username"));
    }

    Redirect::to("/")
}



/// 
///External file delivery system, verifies the user id   
///
///INPUT:  the file needed + DB access
///OUTPUT: the requested file, if the user's access suits the file
///
#[get("/<file..>", rank = 1)]
async fn get_file_external_v(file: PathBuf, db: &State<MongoState>, user: GoogleUserInfo, route: &Route) -> Result<Option<NamedFile>, Status> {

    let extension_string = &file
    .extension()
    .unwrap_or_default()
    .to_str()
    .unwrap_or_default()[..];

    let mut allowed = true;
    match extension_string {
    "jpeg" | "png" | "jpg" => {
        let temp = has_access(user.email.clone(), file.file_stem().
                            unwrap_or_default().to_str()
                            .unwrap_or_default().into(), 
                            &db.db, false).await; 
        match temp {
            Ok(value) => {
                allowed = value;
            },
            Err(_) => {
                error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
                return Err(Status::InternalServerError);
            },
        }
    },
    _ => {},
    };
    
    match allowed {
        true => return Ok(get_file_internal(file).await),
        false => return Err(Status::Forbidden),
    }
}


/// 
///External file delivery system, doesn't verifies the user id   
///
///INPUT:  the requested file path
///OUTPUT: the file, if its from the login page
///
#[get("/<file..>", rank = 2)]
async fn get_file_external_uv(file: PathBuf) -> Result<Option<NamedFile>, Redirect> {
    if !ALLOWED_FILES.contains(&file.to_str().unwrap_or_default()) {
        return Err(Redirect::to(uri!(index)));
    }
    Ok(get_file_internal(file).await)
}

/// 
///Home screen route  
///
///INPUT:  user verification, internal config and DB access
///OUTPUT: the rendered page
///
#[get("/home")]
async fn home<'r>(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, route: &Route) -> Result<content::Html<String>, Status> {
    let exists = ODM::odm::user_exists(&db.db, user.email.clone()).await;
    let exists = match exists {
        Ok(value) => value,
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Err(Status::InternalServerError)
        },
    };

    if !exists {
        let mut name = user.name.clone();
        if name.len() > 6 {
            name = name[0..6].to_string();
        }
        let _ = ODM::odm::insert_user(&db.db, user.email.clone(), name).await;
    }
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone())
                                      .await.unwrap().unwrap();
    
    let format = html_format::html_format::format_home(&file.config, uid, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
}


/// 
///Index route 
///
///INPUT:  None
///OUTPUT: the index html file
///
#[get("/")]
async fn index() -> Option<NamedFile> {
    get_html_file(Path::new("index.html").to_path_buf()).await
}


/// 
///Add screen route  
///
///INPUT:  user verification, internal config and DB access
///OUTPUT: the rendered page
///
#[get("/add-car")]
async fn add_car(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, route: &Route) -> Result<content::Html<String>, Status> {

    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Err(Status::InternalServerError);
        },
        Ok(value) => value.unwrap()
    };

    let cars = ODM::odm::get_cars_for_user(&db.db,uid.clone()).await.unwrap().len();

    if cars >= 5 {
        return Err(Status::Forbidden);
    }

    let format = html_format::html_format::format_add_car(&file.config, uid, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
}


///
///Services screen route  
///
///INPUT:  user verification, internal config, car ID and DB access
///OUTPUT: the rendered page
///

#[get("/services/<car>")]
async fn car_services(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, car: String, route: &Route) -> Result<content::Html<String>, Status> {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Err(Status::InternalServerError)
        },
        Ok(value) => value,
    };

    if !access  {
        return Err(Status::Forbidden);
    }
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => value.unwrap()
    };

    let format = html_format::html_format::format_services(&file.config, uid, car, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
}

/// 
///Overview screen route  
///
///INPUT:  user verification, internal config, car ID and DB access
///OUTPUT: the rendered page
///
#[get("/overview/<car>")]
async fn car_overview(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, car: String, route: &Route) -> Result<content::Html<String>, Status> {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Err(Status::InternalServerError);
        },
        Ok(value) => value,
    };

    if !access  {
        return Err(Status::Forbidden);
    }


    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => value.unwrap()
    };

    let format = html_format::html_format::format_overview(&file.config, uid, car, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
}

/// 
///Calendar screen route  
///
///INPUT:  user verification, internal config, car ID and DB access
///OUTPUT: the rendered page
///
#[get("/calendar/<car>")]
async fn car_calendar(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, car: String, route: &Route) -> Result<content::Html<String>, Status> {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", &route, &user.email));
            return Err(Status::InternalServerError);
        },
        Ok(value) => value,
    };

    if !access  {
        return Err(Status::Forbidden);
    }
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => value.unwrap()
    };

    let format = html_format::html_format::format_calendar(&file.config, uid, car, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
}


/// 
///Calendar screen route  
///
///INPUT:  user verification, internal config, car ID and DB access
///OUTPUT: the rendered page
///
#[get("/locations/<car>")]
async fn car_locations(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, car: String, route: &Route) -> Result<content::Html<String>, Status> {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Err(Status::InternalServerError);
        },
        Ok(value) => value,
    };

    if !access  {
        return Err(Status::Forbidden);
    }
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => value.unwrap()
    };

    let format = html_format::html_format::format_locations(&file.config, uid, car, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
}


/// 
///Edit car screen route  
///
///INPUT:  user verification, internal config, car ID and DB access
///OUTPUT: the rendered page
///
#[get("/edit/<car>")]
async fn edit_car(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, car: String, route: &Route) -> Result<content::Html<String>, Status> {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", &route, &user.email));
            return Err(Status::InternalServerError);
        },
        Ok(value) => value,
    };

    if !access  {
        return Err(Status::Forbidden);
    }
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => value.unwrap()
    };

    let format = html_format::html_format::format_car_edit(&file.config, uid, car, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
}



/// 
///Drive GPS screen route  
///
///INPUT:  user verification, internal config, car ID and DB access
///OUTPUT: the rendered page
///
#[get("/drive/<car>")]
async fn drive(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, car: String, route: &Route) -> Result<content::Html<String>, Status> {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Err(Status::InternalServerError);
        },
        Ok(value) => value,
    };

    if !access  {
        return Err(Status::Forbidden);
    }
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => value.unwrap()
    };

    let format = html_format::html_format::format_drive(&file.config, uid, car, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
}

/// 
///car view screen route  
///
///INPUT:  user verification, internal config, car ID and DB access
///OUTPUT: the rendered page
///
#[get("/view/<car>")]
async fn car_view(car: String, user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, route: &Route) -> Result<content::Html<String>, Status> {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Err(Status::InternalServerError);
        },
        Ok(value) => value,
    };

    if !access  {
        return Err(Status::Forbidden);
    }
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => value.unwrap()
    };

    let format = html_format::html_format::format_car_view(&file.config, uid, car, &db.db).await;
    let format = match format {
        Err(_) => return Err(Status::InternalServerError),
        Ok(value) => {
            info!("{}", format!("Server>>Client:\tSending formatted {} for {}\ninfo: {}", &route, &user.email, &value.0[..][0..100]));
            value
        },
    };

    Ok(format)
    
}

#[get("/<temp..>", rank = 3)]
async fn panel_redirect(temp: PathBuf) -> Redirect {
    Redirect::to(uri!(index))
}

pub fn get_routes() -> Vec<Route> {
    return routes![
        google_login, google_callback, logout, get_file_external_v, 
        get_file_external_uv, home, index, add_car, car_services,
        car_overview, car_calendar, car_locations, edit_car,
        drive,car_view, panel_redirect];
}