use super::{*};

use std::fs::File;
use chrono::prelude::*;

use rocket::Data;
use rocket::http::ContentType;
use rocket_multipart_form_data::{mime, MultipartFormDataOptions, MultipartFormData, MultipartFormDataField};

use ODM::models::*;
use ODM::odm::*;

use log::{error,info};
/// 
///Toggles invites ( slide button )
///
///INPUT:  user verification, DB access
///OUTPUT: Ok / Err
///
#[post("/toggleInvites")]
async fn toggle_invites(db: &State<MongoState>, user: GoogleUserInfo, route: &Route) -> Status {
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Status::InternalServerError;
        },
        Ok(value) => value.unwrap()
    };


    match ODM::odm::toggle_user_invites(&db.db, uid.clone()).await.unwrap() {
                None => return Status::BadRequest,
                Some(value) => {
                    info!("{}", format!("Server>>Client:\tApproving action {} for {}", &route, &uid));
                    return Status::NoContent;
                },
            }
}

/// 
///Add car checker and uploader route 
///
///INPUT:  user verification, form data and DB access
///OUTPUT: Ok / Err
///
#[post("/add-car", data = "<data>")]
async fn post_add_car(user : GoogleUserInfo, content_type: &ContentType, data: Data<'_>, db: &State<MongoState>, route: &Route) -> Status {
    
    let uid = ODM::odm::get_id_by_username(&db.db, user.email.clone()).await;
    let uid = match uid {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Status::InternalServerError;
        },
        Ok(value) => value.unwrap()
    };

    let cars = ODM::odm::get_cars_for_user(&db.db,uid.clone()).await.unwrap().len();

    if cars >= 5 {
        return Status::Forbidden;
    }

    // Extract the parts from the form
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            
            MultipartFormDataField::raw("picture").content_type_by_string(Some(mime::IMAGE_STAR)).unwrap(),
            MultipartFormDataField::text("user1"),
            MultipartFormDataField::text("user2"),
            MultipartFormDataField::text("user3"),
            MultipartFormDataField::text("user4"),
            MultipartFormDataField::text("model"),
            MultipartFormDataField::text("year"),
            MultipartFormDataField::text("plate"),
            MultipartFormDataField::text("tank-cap"),
            MultipartFormDataField::text("cur-fule"),
            MultipartFormDataField::text("is-electric"),
            MultipartFormDataField::text("last-fule-date"),
            MultipartFormDataField::text("last-fule-description"),
            MultipartFormDataField::text("last-oil-date"),
            MultipartFormDataField::text("last-oil-description"),
            MultipartFormDataField::text("last-clean-date"),
            MultipartFormDataField::text("last-clean-description"),

        ]
    );

    let multipart_form_data = MultipartFormData::parse(content_type, data, options).await;
    let multipart_form_data = match multipart_form_data {
        Ok(value) => value,
        Err(_) => return Status::UnprocessableEntity,
    };
    // Divide into sections
    let photo = multipart_form_data.raw.get("picture"); // Use the get method to preserve file fields from moving out of the MultipartFormData instance in order to delete them automatically when the MultipartFormData instance is being dropped
    let pre_users = vec![
        multipart_form_data.texts.get("user1"),
        multipart_form_data.texts.get("user2"),
        multipart_form_data.texts.get("user3"),
        multipart_form_data.texts.get("user4"),
    ];
    let pre_specs = vec![
        multipart_form_data.texts.get("model"),
        multipart_form_data.texts.get("year"),
        multipart_form_data.texts.get("plate"),
        multipart_form_data.texts.get("tank-cap"),
        multipart_form_data.texts.get("cur-fule"),
        multipart_form_data.texts.get("is-electric"),
    ];
    let pre_checkups_dates = vec! [
        multipart_form_data.texts.get("last-fule-date"),
        multipart_form_data.texts.get("last-oil-date"),
        multipart_form_data.texts.get("last-clean-date"),
    ];
    let pre_checkups_descriptions = vec! [
        multipart_form_data.texts.get("last-fule-description"),
        multipart_form_data.texts.get("last-oil-description"),
        multipart_form_data.texts.get("last-clean-description"),
    ];

    // Check that everything's valid
    let mut users = Vec::<String>::new();
    for user in pre_users {
        if let Some(inner) = user {
            users.push(inner[0].text.to_string());
            continue;
        }
    }

    let mut specs = Vec::<String>::new();
    for spec in pre_specs {
        if let Some(inner) = spec {
            specs.push(inner[0].text.to_string());
            continue;
        }
        // specs has a length of 5 exactly (excluding description)
        if specs.len() != 5 {
            return Status::NotAcceptable;
        }
    }
    
    // Extract dates
    let mut checkups_dates = vec![];
    let length = pre_checkups_dates.len();
    for i in 0..length {
        if let Some(inner) = pre_checkups_dates[i] {
            let chrono_dt: chrono::DateTime<Utc> =
            match (inner[0].text.to_string()+"T00:00:00Z").parse() {
                Err(_) => return Status::UnprocessableEntity,
                Ok(val) => val,
            };
            let bson_dt: bson::DateTime = chrono_dt.into();
            let bson_dt = bson::DateTime::from_chrono(chrono_dt);
            checkups_dates.push(bson_dt);
            continue;
        }
        return Status::NotAcceptable;
    }

    // Set up decriptions (if exist) for checkups
    let mut checkups_descriptions: [String; 3] = [String::from(""),
                                                  String::from(""),
                                                  String::from("")];
    let length = pre_checkups_descriptions.len();
    for i in 0..length {
        if let Some(inner) = pre_checkups_descriptions[i] {
            checkups_descriptions[i] = inner[0].text.to_string();
            continue;
        }
    }

    // Continutes only if a photo has arrived
    if let Some(file_fields) = photo {
        let file_field = &file_fields[0]; 

        // Has to be JPEG type!
        if &file_field.content_type != &Some(mime::IMAGE_JPEG) {
            return Status::UnsupportedMediaType;
        }
        // Creates specs
        let owner_id = uid.clone();

        let specs_struct = Specs {
            model: specs[0].clone(),
            yearMade: specs[1].clone().parse::<i32>().unwrap(),
            licensePlateNumber: specs[2].clone(),
            tankCapacity: specs[3].clone().parse::<i32>().unwrap(),
            currentFuleAmount: specs[4].clone().parse::<i32>().unwrap(),
            isElectric: specs.len() != 5,
        };

        // Puts in correct users
        let mut final_users = Vec::<String>::new();
        for user in users {
            match ODM::odm::get_id_by_username(&db.db, user).await {
                Err(_) => return Status::InternalServerError,
                Ok(val) => {
                    match val {
                        None => {},
                        Some(value) => final_users.push(value),
                    }
                }
            }
        }

        // Car insertion to DB - free unwrap because a check has already been done
        let car_id = ODM::odm::get_max_car_id(&db.db).await.unwrap() + 1;
        let _ = ODM::odm::insert_car(&db.db, owner_id.clone(), final_users.clone(), vec![], specs_struct).await;
        let _ = ODM::odm::update_user_cars(&db.db, owner_id.clone(), Action::Append, car_id.to_string()).await;

        // Add a user access to DB
        for user in final_users {
            if ODM::odm::get_user_invites(&db.db, user.clone()).await.unwrap_or(Some(false)).unwrap_or(false) {
                let _ = ODM::odm::update_user_cars(&db.db, user, Action::Append, car_id.to_string()).await;
            }
        }
        // Creates events for checkups
        let refule = Event {
            name: "Refule".into(),
            date: checkups_dates[0],
            createdBy: owner_id.clone(),
            description: checkups_descriptions[0].clone(),
            cost: 0,
        };
        let _ = ODM::odm::add_car_event(&db.db, car_id.to_string(), refule).await;
        let oil = Event {
            name: "Oil Change".into(),
            date: checkups_dates[1],
            createdBy: owner_id.clone(),
            description: checkups_descriptions[1].clone(),
            cost: 0,
        };
        let _ = ODM::odm::add_car_event(&db.db, car_id.to_string(), oil).await;
        let cleaning = Event {
            name: "Cleaning".into(),
            date: checkups_dates[2],
            createdBy: owner_id.clone(),
            description: checkups_descriptions[2].clone(),
            cost: 0,
        };
        let _ = ODM::odm::add_car_event(&db.db, car_id.to_string(), cleaning).await;

        // Create the received JPEG
        let mut file = File::create(format!("./assets/{}.jpg", car_id)).unwrap();
        // Write a slice of bytes to the file
        let _ = file.write_all(&file_field.raw);
        info!("{}", format!("Server>>Client:\tApproving action {} for {}", &route, &uid));
        return Status::NoContent;
    }
    Status::UnsupportedMediaType
}



/// 
///Add event route 
///
///INPUT:  user verification, internal config, car IDm the added data and DB access
///OUTPUT: Ok / Err
///
#[post("/calendar/<car>", data = "<data>")]
async fn post_car_calendar(user : GoogleUserInfo, content_type: &ContentType, data: Data<'_>, db: &State<MongoState>, car: String, route: &Route) -> Status {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Status::InternalServerError;
        },
        Ok(value) => value,
    };

    if !access  {
        return Status::Forbidden;
    }

    let uid = ODM::odm::get_id_by_username(&db.db, user.email).await;
    let uid = match uid {
        Err(_) => return Status::InternalServerError,
        Ok(value) => value.unwrap()
    };

    // Similar to add-car
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            
            MultipartFormDataField::text("date"),
            MultipartFormDataField::text("type"),
            MultipartFormDataField::text("cost"),
            MultipartFormDataField::text("description"),
        ]
    );

    let multipart_form_data = MultipartFormData::parse(content_type, data, options).await;
    let multipart_form_data = match multipart_form_data {
        Ok(value) => value,
        Err(_) => return Status::UnprocessableEntity,
    };
    let pre_data = vec![
        multipart_form_data.texts.get("date"),
        multipart_form_data.texts.get("type"),
        multipart_form_data.texts.get("cost"),
        multipart_form_data.texts.get("description"),
    ];

    let mut event_data = vec![];
    for detail in pre_data {
        if let Some(value) = detail {
            event_data.push(value[0].text.to_string());
            continue;
        }
        return Status::NotAcceptable;
    }
    let date = event_data[0].clone();
    let chrono_dt: chrono::DateTime<Utc> = (date+"T00:00:00Z").parse().unwrap();
    let bson_dt: bson::DateTime = chrono_dt.into();
    let bson_dt = bson::DateTime::from_chrono(chrono_dt);

    let event = Event {
        createdBy:   uid.clone(),
        date:        bson_dt,
        name:        event_data[1].clone(),
        cost:        event_data[2].parse::<i32>().unwrap(),
        description: event_data[3].clone(),
    };

    let _ = ODM::odm::add_car_event(&db.db, car.clone(), event).await.unwrap();

    info!("{}", format!("Server>>Client:\tApproving action {} for {}", &route, &uid));
    Status::NoContent
}


/// 
///Add location route 
///
///INPUT:  user verification, internal config, car IDm the added data and DB access
///OUTPUT: Ok / Err
///
#[post("/locations/<car>", data = "<data>")]
async fn post_car_locations(user : GoogleUserInfo, content_type: &ContentType, data: Data<'_>, db: &State<MongoState>, car: String, route: &Route) -> Status {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Status::InternalServerError;
        },
        Ok(value) => value,
    };

    if !access  {
        return Status::Forbidden;
    }

    let uid = ODM::odm::get_id_by_username(&db.db, user.email).await;
    let uid = match uid {
        Err(_) => return Status::InternalServerError,
        Ok(value) => value.unwrap()
    };

    if !ODM::odm::is_owner(&db.db, uid.clone(), car.clone()).await.unwrap_or(false) {
        return Status::Forbidden;
    }

    let locations = ODM::odm::get_car_locations(&db.db, car.clone()).await.unwrap();
    if locations.len() > 9 {
        return Status::BadRequest;
    }
    

    // Similar to add-car
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            MultipartFormDataField::text("type"),
            MultipartFormDataField::text("description"),
            MultipartFormDataField::text("lat"),
            MultipartFormDataField::text("lon"),
        ]
    );

    let multipart_form_data = MultipartFormData::parse(content_type, data, options).await;
    let multipart_form_data = match multipart_form_data {
        Ok(value) => value,
        Err(_) => return Status::UnprocessableEntity,
    };
    let pre_data = vec![
        multipart_form_data.texts.get("type"),
        multipart_form_data.texts.get("description"),
        multipart_form_data.texts.get("lat"),
        multipart_form_data.texts.get("lon"),
    ];

    let mut location_data = vec![];
    for detail in pre_data {
        if let Some(value) = detail {
            location_data.push(value[0].text.to_string());
            continue;
        }
        return Status::NotAcceptable;
    }

    let lat = location_data[2].parse::<f32>();
    let lat = match lat {
        Err(_) => return Status::UnprocessableEntity,
        Ok(value) => value,
    };

    let lon = location_data[3].parse::<f32>();
    let lon = match lon {
        Err(_) => return Status::UnprocessableEntity,
        Ok(value) => value,
    };

    if !(lat <= 90.0 && lat >= -90.0) {
        return Status::UnprocessableEntity;
    }
    if !(lon <= 180.0 && lon >= -180.0) {
        return Status::UnprocessableEntity;
    }

    let point = Point {
        name: location_data[0].clone(),
        description: location_data[1].clone(),
        location: vec![lon,lat],
    };

    let _ = add_car_location(&db.db, car.clone(), point).await.unwrap();

    info!("{}", format!("Server>>Client:\tApproving action {} for {}", &route, &uid));

    Status::NoContent
}



/// 
///Delete location route 
///
///INPUT:  user verification, internal config, car ID, the added data and DB access
///OUTPUT: Ok / Err
///
#[post("/locations/<car>/delete", data = "<data>")]
async fn delete_car_location(user : GoogleUserInfo, data: Json<LocationStruct>, db: &State<MongoState>, car: String, route: &Route) -> Status {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Status::InternalServerError;
        },
        Ok(value) => value,
    };

    if !access  {
        return Status::Forbidden;
    }

    let uid = ODM::odm::get_id_by_username(&db.db, user.email).await;
    let uid = match uid {
        Err(_) => return Status::InternalServerError,
        Ok(value) => value.unwrap()
    };

    if !ODM::odm::is_owner(&db.db, uid.clone(), car.clone()).await.unwrap_or(false) {
        return Status::Forbidden;
    }

    let location = data.location_del;

    if !(location.0 <= 90.0 && location.0 >= -90.0) {
        return Status::UnprocessableEntity;
    }
    if !(location.1 <= 180.0 && location.1 >= -180.0) {
        return Status::UnprocessableEntity;
    }

    let _ = ODM::odm::delete_car_location(&db.db, car.clone(), data.location_del).await;

    info!("{}", format!("Server>>Client:\tApproving action {} for {}", &route, &uid));
    Status::NoContent
}

/// 
///Add Drive screen route  
///
///INPUT:  user verification, internal config, car ID, the ride and DB access
///OUTPUT: Ok / Err
///
#[post("/drive/<car>", data = "<drive>")]
async fn post_drive(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, car: String, drive: Json<DriveDoc>, route: &Route) -> Status {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Status::InternalServerError;
        },
        Ok(value) => value,
    };

    if !access  {
        return Status::Forbidden;
    }

    let uid = ODM::odm::get_id_by_username(&db.db, user.email).await;
    let uid = match uid {
        Err(_) => return Status::InternalServerError,
        Ok(value) => value.unwrap()
    };

    let drive = drive_to_doc(drive, uid.clone());
    let _ = ODM::odm::update_car_fule(&db.db, car.clone(), drive.fule.clone()).await;
    let _ = ODM::odm::add_car_drive(&db.db, car.clone(), drive).await;

    info!("{}", format!("Server>>Client:\tApproving action {} for {}", &route, &uid));
    Status::NoContent
}


/// 
///Post edit car route  - see add-car for similar doc
///
///
///INPUT:  user verification, data, car ID and DB access
///OUTPUT: Ok / Err
///
#[post("/edit-car/<car>", data = "<data>")]
async fn post_edit_car(user : GoogleUserInfo, content_type: &ContentType, data: Data<'_>, db: &State<MongoState>, car: String, route: &Route) -> Status {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Status::InternalServerError;
        },
        Ok(value) => value,
    };

    if !access  {
        return Status::Forbidden;
    }

    let uid = ODM::odm::get_id_by_username(&db.db, user.email).await;
    let uid = match uid {
        Err(_) => return Status::InternalServerError,
        Ok(value) => value.unwrap()
    };

    if !ODM::odm::is_owner(&db.db, uid.clone(), car.clone()).await.unwrap_or(false) {
        return Status::Forbidden;
    }
    

    let options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            
            MultipartFormDataField::text("user1"),
            MultipartFormDataField::text("user2"),
            MultipartFormDataField::text("user3"),
            MultipartFormDataField::text("user4"),
        ]
    );

    let multipart_form_data = MultipartFormData::parse(content_type, data, options).await;
    let multipart_form_data = match multipart_form_data {
        Ok(value) => value,
        Err(_) => return Status::UnprocessableEntity,
    };

    let pre_users = vec![
        multipart_form_data.texts.get("user1"),
        multipart_form_data.texts.get("user2"),
        multipart_form_data.texts.get("user3"),
        multipart_form_data.texts.get("user4"),
    ];

    let mut users = Vec::<String>::new();
    for user in pre_users {
        if let Some(inner) = user {
            users.push(inner[0].text.to_string());
            continue;
        }
    }

    let car_obj = ODM::odm::get_car(&db.db, car.clone()).await;
    let car_obj = match car_obj {
        Err(val) => return Status::InternalServerError,
        Ok(val) => {
            match val {
                Some(value) => value,
                None => return Status::UnprocessableEntity,
            }
        }
    };
    let past_users = car_obj.users;

    let mut final_users = Vec::<String>::new();
    for user in users {
        match ODM::odm::get_id_by_username(&db.db, user).await.unwrap() {
            None => {},
            Some(value) => final_users.push(value),
        }
    }
    // Deletes access to past users and adds to new
    for user in past_users {
        if ODM::odm::get_user_invites(&db.db, user.clone()).await.unwrap_or(Some(false)).unwrap_or(false) {
            let _ = ODM::odm::update_user_cars(&db.db, user, Action::Delete, car.clone()).await;
        }
    }

    for user in &final_users {
        if ODM::odm::get_user_invites(&db.db, user.clone()).await.unwrap_or(Some(false)).unwrap_or(false) {
            let _ = ODM::odm::update_user_cars(&db.db, user.clone(), Action::Append, car.clone()).await;
        }
    }

    let _ = ODM::odm::update_car_users(&db.db, car, final_users).await;
    info!("{}", format!("Server>>Client:\tApproving action {} for {}", &route, &uid));
    Status::NoContent
}

/// 
///Delete car route  
///
///INPUT:  user verification, internal config, car ID and DB access
///OUTPUT: Ok / Err
///
#[post("/deleteCar/<car>")]
async fn delete_car(user : GoogleUserInfo, db: &State<MongoState>, file: &State<Config>, car: String, route: &Route) -> Status {
    let access = has_access(user.email.clone(), car.clone(), &db.db, true).await;

    let access = match access {
        Err(_) => {
            error!("{}", format!("Database failed while getting {} for {}", route, &user.email));
            return Status::InternalServerError;
        },
        Ok(value) => value,
    };

    if !access  {
        return Status::Forbidden;
    }
    let uid = ODM::odm::get_id_by_username(&db.db, user.email).await;
    let uid = match uid {
        Err(_) => return Status::InternalServerError,
        Ok(value) => value.unwrap()
    };

    let is_owner = ODM::odm::is_owner(&db.db, uid.clone(), car.clone()).await.unwrap();
    // If it's the owner, deletes car from every account, else - just leaves
    match is_owner {
        true => {
            let _ = ODM::odm::delete_car_by_id(&db.db, car).await;
        },
        false => {
            let mut users = ODM::odm::get_car(&db.db, car.clone()).await.unwrap().unwrap().users;
            let _ = &users.retain(|x| x != &uid);
            // Delete and update
            let _ = ODM::odm::update_user_cars(&db.db, uid.clone(), Action::Delete, car.clone()).await;
            let _ = ODM::odm::update_car_users(&db.db, car, users).await;
        },
    }
    info!("{}", format!("Server>>Client:\tApproving action {} for {}", &route, &uid));
    Status::Ok
}


pub fn post_routes() -> Vec<Route> {
    return routes![
        toggle_invites, post_add_car, post_car_calendar, post_car_locations, 
        delete_car_location, post_drive, post_edit_car, delete_car];
}