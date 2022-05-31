//! 
//! Documentation of the odm module.
//! Used to connect to the TalMobil database.
//!




#![allow(non_snake_case)]
#![allow(unused_imports)]
use super::*;
use models::*;
use odm_utils::*;
use config::config::ConfyConfig;
use std::time::{Duration, Instant};

//use futures::stream::StreamExt;
use mongodb::bson::{doc, DateTime, Document, Array };
use mongodb::Database;

// The async ODM for the MongoDB database connection and queries

// use chrono::prelude::*;
use futures::stream::TryStreamExt;
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::options::FindOptions;
use mongodb::options::ReturnDocument;

// use mongodb::bson::{doc, Document};
use mongodb::options::ClientOptions;
use mongodb::{Client};
use mongodb::options::ListDatabasesOptions;
use log::{debug, error, info, trace, warn};

/// 
/// Initiate DB connection
///
///
/// # Arguments
///
/// * `config` - A config object containing 'database' and 'database_ip'
///
/// 
/// # Log
/// 
/// * `info` - "Database Connected!", indicating success
/// * `error` - "Could not connect to MongoDB {error}", indicating error
/// 
pub async fn init(config: &ConfyConfig) -> mongodb::error::Result<Database> {
    connect(&config).await
}

/// basic connection, isn't available out of the "odm.rs" module
async fn connect(config: &ConfyConfig) -> mongodb::error::Result<Database> {
    
    let mut client_options = ClientOptions::parse(&config.database_ip).await?;
    client_options.connect_timeout = Some(Duration::from_secs(config.timeout));
    client_options.heartbeat_freq = Some(Duration::from_secs(config.timeout));
    client_options.server_selection_timeout = Some(Duration::from_secs(config.timeout));
    let client = Client::with_options(client_options)?;

    match client.list_database_names(Document::new(), ListDatabasesOptions::builder().build()).await {
        Ok(value) => {
            info!("Database Connected!");
        },
        Err(value) => {
            error!("{}", format!("Could not connect to MongoDB {}", value));
            return Err(value);
        },
    }

    let database = client.database(&config.database[..]);

    Ok(database)
}

//
// USER ACTIONS
//


/// 
/// Get username by id
/// 
/// # Arguments
///
/// * `database` - Refrence to a database object
/// * `id` - User ID
///
/// # Output
/// 
/// * None - if the user could not be found
/// * Some(value) - if the user has been found
/// * Some("") - empty string, indicates database error
/// 
/// # Example
/// ```
/// let value = get_username_by_id(&db, id).await; //randomuser@gmail.com, id="6"
/// assert_eq!(value, Some("random"));
/// ```
/// 
pub async fn get_username_by_id(db : &Database, id: String) -> mongodb::error::Result<Option<String>>  {
    let user = get_user_by_id(&db, id).await?;

    match user {
        None => return Ok(None),
        Some(value) => return Ok(Some(value.username)),
    }
}

/// 
/// Get id by email
/// 
/// # Arguments
///
/// * `database` - Refrence to a database object
/// * `email` - User email
///
pub async fn get_id_by_username(db : &Database, email: String) -> mongodb::error::Result<Option<String>>  {
    let user = get_user_by_email(&db, email).await?;

    match user {
        None => return Ok(None),
        Some(value) => return Ok(Some(value.userID)),
    }
}

/// 
/// Get email by email
/// 
/// # Arguments
///
/// * `database` - Refrence to a database object
/// * `id` - User ID
///
pub async fn get_email_by_id(db : &Database, id: String) -> mongodb::error::Result<Option<String>>  {
    let user = get_user_by_id(&db, id).await?;

    match user {
        None => return Ok(None),
        Some(value) => return Ok(Some(value.email)),
    }
}

/// 
/// Get all users - EXP (unsafe to use - no wrapping)
/// 
/// # Arguments
///
/// * `database` - Refrence to a database object
///
/// # Output
/// 
/// * Err(_) - indicates DB error
/// * Ok(vec) - all users
/// 
pub async fn get_all_users(db: &Database) -> mongodb::error::Result<Vec<User>> {
    let collection = db.collection::<UserDocument>("users");
    let find_options = FindOptions::builder().build();

    let mut cursor = collection.find(None, find_options).await?;

    let mut users: Vec<User> = vec![];
    while let Some(result) = cursor.try_next().await? {
        users.push(doc_to_user(&result));
    }

    Ok(users)
}


/// 
/// Get user by id
/// 
/// # Arguments
///
/// * `database` - Refrence to a database object
/// * `id` - user ID
///
/// # Output
/// 
/// * Err(_) - indicates DB error
/// * Ok(None) - user has not been found
/// * Ok(Some(value)) - user has been found
/// 
pub async fn get_user_by_id(
    db: &Database,
    id: String,
) -> mongodb::error::Result<Option<User>> {
    let collection = db.collection::<UserDocument>("users");

    let customer_doc = collection.find_one(doc! {"userID":id }, None).await?;
    if customer_doc.is_none() {
        return Ok(None);
    }

    let unwrapped_doc = customer_doc.unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_user(&unwrapped_doc);

    Ok(Some(customer_json))
}

/// Get user by email
pub async fn get_user_by_email(
    db: &Database,
    email: String,
) -> mongodb::error::Result<Option<User>> {
    let collection = db.collection::<UserDocument>("users");

    let customer_doc = collection.find_one(doc! {"email":email }, None).await?;
    if customer_doc.is_none() {
        return Ok(None);
    }

    let unwrapped_doc = customer_doc.unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_user(&unwrapped_doc);

    Ok(Some(customer_json))
}

/// Get all the cars of a user using ID
pub async fn get_cars_for_user(db : &Database, id: String) -> mongodb::error::Result<Vec<String>> {

    let user = get_user_by_id(&db, id).await?;

    let user = match user {
        Some(value) => value,
        None => return Ok(vec![]),
    };

    Ok(user.cars)
}

/// Get user allowedInvites field (True/False)
pub async fn get_user_invites(db : &Database, id: String) -> mongodb::error::Result<Option<bool>> {
    let user = get_user_by_id(&db, id).await?;

    match user {
        None => return Ok(None),
        Some(value) => return Ok(Some(value.allow_invites)),
    }
}

/// Checks if a user exists (via email)
pub async fn user_exists(db: &Database, email: String) -> mongodb::error::Result<bool> {
    match get_user_by_email(&db, email).await {
        Err(value) => Err(value),
        Ok(value) =>{
            match value {
                None => return Ok(false),
                Some(value) => return Ok(true),
            }
        }
    }
}

/// Creates an inserts a new user
pub async fn insert_user(
    db: &Database,
    email: String,
    name: String,
) -> mongodb::error::Result<String> {
    let collection = db.collection::<Document>("users");

    let max_id = get_max_id(&db).await;

    let max_id = match max_id {
        Err(value) => return Err(value),
        Ok(value) => value,
    };

    let id = ( max_id + 1 ).to_string();

    let insert_one_result = collection
        .insert_one(
            doc! {
                  "email":      email.clone(),
                  "userID":     id,
                  "username":   name.clone(),
                  "cars":       Vec::<String>::new(),
                  "allowInvites":true
        },
            None,
        )
        .await?;

    Ok(insert_one_result.inserted_id.to_string())
}

/// Update the cars of a user, using the Action enum and car's id
/// 
/// # Example
/// ```
/// update_user_cars(&db, "5", Action::Delete, "1");
/// ```
/// ‎ 
pub async fn update_user_cars(
    db: &Database,
    id: String,
    action: Action,
    car_id: String
) -> mongodb::error::Result<Option<User>> {
    let collection = db.collection::<UserDocument>("users");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let cars = get_cars_for_user(&db, id.clone()).await;
    let mut cars = match cars {
        Ok(value) => value,
        Err(value) => return Err(value),
    };

    match action {
        Action::Delete => &cars.retain(|x| x != &car_id),
        Action::Append if &cars.len() < &5 => &cars.push(car_id.clone()),
        _ => &{ },
    };

    //let id = get_user_by_id(&db, id).await.unwrap().unwrap()._id;

    let customer_doc = collection
        .find_one_and_update(
            doc! {"userID":  id   },
            doc! {"$set": doc! { "cars": cars} },
            find_one_and_update_options,
        )
        .await;
    
    let customer_doc = match customer_doc {
        Ok(value) => value,
        Err(value) => return Err(value),
    };

    if customer_doc.is_none() {
        return Ok(None);
    }

    let unwrapped_doc = customer_doc.unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_user(&unwrapped_doc);

    Ok(Some(customer_json))
}

/// Toggles the user's "allowInvites" field
pub async fn toggle_user_invites(
    db: &Database,
    id: String,
    ) -> mongodb::error::Result<Option<User>> {

    let collection = db.collection::<UserDocument>("users");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();
    
    let is_on = get_user_invites(&db, id.clone()).await?;
    let is_on = match is_on {
        Some(value) => value,
        None => return Ok(None),
    };

    let customer_doc = collection
        .find_one_and_update(
            doc! {"userID":  id   },
            doc! {"$set": doc! { "allowInvites": !is_on} },
            find_one_and_update_options,
        )
        .await;

    let customer_doc = match customer_doc {
        Ok(value) => value,
        Err(value) => return Err(value),
    };

    if customer_doc.is_none() {
        return Ok(None);
    }
    
    let unwrapped_doc = customer_doc.unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_user(&unwrapped_doc);

    Ok(Some(customer_json))
}

/// Gets all the cars owned by a given user
pub async fn get_all_owned(    
    db: &Database,
    id: String
    ) -> mongodb::error::Result<Vec<String>> {

    let cars = get_cars_for_user(&db, id.clone()).await?;

    let mut owned = Vec::<String>::new();
    for car in cars {
        if is_owner(&db, id.clone(), car.clone()).await? {
            owned.push(car);
        }
    }
    Ok(owned)
}

/// Gets all the cars that are not owned by a given user (but still are in his inventory)
pub async fn get_all_accessible(
    db: &Database,
    id: String
    ) -> mongodb::error::Result<Vec<String>> {
        
    let cars = get_cars_for_user(&db, id.clone()).await?;

    let mut unowned = Vec::<String>::new();
    for car in cars {
        if !is_owner(&db, id.clone(), car.clone()).await.unwrap_or(false) {
            let _ = &unowned.push(car);
        }
    }
    Ok(unowned)

}

//
// CAR actions
//

/// Gets all the cars
pub async fn get_all_cars(db: &Database) -> mongodb::error::Result<Vec<Car>> {
    let collection = db.collection::<CarDocument>("cars");
    let find_options = FindOptions::builder().build();
    
    let mut cursor = collection.find(None, find_options).await?;

    let mut cars: Vec<Car> = vec![];
    while let Some(result) = cursor.try_next().await? {
        cars.push(doc_to_car(&result));
    }
    Ok(cars)
}

/// Get the top car ID currently
pub async fn get_max_car_id(db: &Database) -> mongodb::error::Result<i32> {
    
    let cars_vector = get_all_cars(&db).await?;

    if cars_vector.len() < 1 {
        return Ok(0);
    }

    let mut id_vector = vec![];

    for user in cars_vector {
        id_vector.push(user.carID.parse::<i32>().unwrap())
    }

    Ok(*id_vector.iter().max().unwrap())
}

/// Get car by it's ID
pub async fn get_car(db : &Database, id: String) -> mongodb::error::Result<Option<Car>> {
    let collection = db.collection::<CarDocument>("cars");

    let car_doc = collection.find_one(doc! {"carID":id }, None).await?;
    if car_doc.is_none() {
        return Ok(None);
    }

    let unwrapped_doc = car_doc.unwrap();
    // transform ObjectId to String
    let car_json = doc_to_car(&unwrapped_doc);

    Ok(Some(car_json))
}

/// Get the car's fule bar value
pub async fn get_car_fule_bar(db : &Database, id: String) -> mongodb::error::Result<Option<i32>> {
    let car = get_car(&db, id).await?;

    match car {
        None => return Ok(None),
        Some(value) => {
            return Ok(Some(value.specs.currentFuleAmount));
        }
    }
}

/// Gets all of the car's marked locations
pub async fn get_car_locations(    
    db: &Database,
    car_id: String,
) -> mongodb::error::Result<Vec::<Point>> {
    
    let locations = get_car(&db, car_id.clone()).await?;

    let locations = match locations {
        None => vec![],
        Some(value) => value.locations,
    };

    return Ok(locations);
}

/// Gets all if the car's events
pub async fn get_car_events(    
    db: &Database,
    car_id: String,
) -> mongodb::error::Result<Vec::<Event>> {
    
    let events = get_car(&db, car_id.clone()).await?;

    let events = match events {
        None => vec![],
        Some(value) => value.calendar,
    };

    return Ok(events);
}

/// Adds a given event to a specified car
pub async fn add_car_event(
    db: &Database,
    car_id: String,
    event: Event
) -> mongodb::error::Result<Option<Car>> {
    let collection = db.collection::<CarDocument>("cars");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let mut events = get_car_events(&db, car_id.clone()).await?;

    events.push(event.clone());
    let cloned = event.clone();

    let mut doc : Array = Array::new();
    for event in events {
        doc.push(bson::Bson::Document(doc! {
            "name":        event.name.clone(),
            "date":        event.date.clone(),
            "createdBy":   event.createdBy.clone(),
            "description": event.description.clone(),
            "cost":        event.cost.clone(),
    }));
    }

    let car_doc = collection
        .find_one_and_update(
            doc! {"carID":  car_id.clone()   },
            doc! {"$set": doc! { 
                  "calendar":  doc } },
            find_one_and_update_options,
        )
        .await;
    
    if cloned.name == "Refule" {
        let _ = update_car_fule_check(&db, car_id.clone(), cloned).await;
    }


    match car_doc.clone() {
        Ok(value) => {
                if value.is_none() {
                    return Ok(None);
    }},
        Err(value) => return Err(value),
    }


    let unwrapped_doc = car_doc.unwrap().unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_car(&unwrapped_doc);

    Ok(Some(customer_json))

}

/// Add a custom location to a specified car
pub async fn add_car_location(
    db: &Database,
    car_id: String,
    point: Point
) -> mongodb::error::Result<Option<Car>> {
    let collection = db.collection::<CarDocument>("cars");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let mut locations = get_car_locations(&db, car_id.clone()).await?;

    locations.push(point.clone());

    let mut doc : Array = Array::new();
    for location in locations {
        doc.push(bson::Bson::Document(doc! {
            "name":            location.name.clone(),
            "location":        location.location.clone(),
            "description":     location.description.clone(),
    }));
    }

    let car_doc = collection
        .find_one_and_update(
            doc! {"carID":  car_id.clone()   },
            doc! {"$set": doc! { 
                  "locations":  doc } },
            find_one_and_update_options,
        )
        .await;


    match car_doc.clone() {
        Ok(value) => {
                if value.is_none() {
                    return Ok(None);
    }},
        Err(value) => return Err(value),
    }


    let unwrapped_doc = car_doc.unwrap().unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_car(&unwrapped_doc);

    Ok(Some(customer_json))

}

/// Delete a car location (Point) by its XY
pub async fn delete_car_location(
    db: &Database,
    car_id: String,
    point: (f32, f32)
) -> mongodb::error::Result<Option<Car>> {
    let collection = db.collection::<CarDocument>("cars");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let mut locations = get_car_locations(&db, car_id.clone()).await?;

    locations.retain(|x| (&x.location[0] != &point.0 && &x.location[1] != &point.1) );

    let mut doc : Array = Array::new();
    for location in locations {
        doc.push(bson::Bson::Document(doc! {
            "name":            location.name.clone(),
            "location":        location.location.clone(),
            "description":     location.description.clone(),
    }));
    }

    let car_doc = collection
        .find_one_and_update(
            doc! {"carID":  car_id.clone()   },
            doc! {"$set": doc! { 
                  "locations":  doc } },
            find_one_and_update_options,
        )
        .await;

    match car_doc.clone() {
        Ok(value) => {
                if value.is_none() {
                    return Ok(None);
    }},
        Err(value) => return Err(value),
    }


    let unwrapped_doc = car_doc.unwrap().unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_car(&unwrapped_doc);

    Ok(Some(customer_json))

}

/// Gets all of the car's drives
pub async fn get_car_drives(    
    db: &Database,
    car_id: String,
) -> mongodb::error::Result<Vec::<Drive>> {
    
    let drives = get_car(&db, car_id.clone()).await?;

    let drives = match drives {
        None => vec![],
        Some(value) => value.drive,
    };

    return Ok(drives);
}

/// Get the last drive of a car
pub async fn get_car_last_drive(    
    db: &Database,
    car_id: String,
) -> mongodb::error::Result<Option<Drive>> {
    
    let drives = get_car_drives(&db, car_id).await?;

    if drives.len() > 0 {
        let mut max = &drives[0];

        for elem in &drives {
            if elem.endTime > max.endTime {
                max = elem;
            }
        }

        return Ok(Some(max.clone()));
    }

    return Ok(None);
}

/// Add a drive to a car
pub async fn add_car_drive(
    db: &Database,
    car_id: String,
    drive: Drive
) -> mongodb::error::Result<Option<Car>> {
    let collection = db.collection::<CarDocument>("cars");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let mut drives = get_car_drives(&db, car_id.clone()).await?;

    drives.push(drive.clone());

    let mut doc : Array = Array::new();
    for drive in drives {
        doc.push(bson::Bson::Document(doc! {
            "startTime":     drive.startTime.clone(),
            "endTime":       drive.endTime.clone(),
            "driverID":      drive.driverID.clone(),
            "length":        drive.length.clone(),
            "fule":          drive.fule.clone(),
            "startLocation": drive.startLocation.clone(),
            "endLocation":   drive.endLocation.clone(),
            "passengers":    drive.passengers.clone(),
    }));
    }

    let car_doc = collection
        .find_one_and_update(
            doc! {"carID":  car_id   },
            doc! {"$set": doc! { 
                  "drive":  doc } },
            find_one_and_update_options,
        )
        .await;

    match car_doc.clone() {
        Ok(value) => {
                if value.is_none() {
                    return Ok(None);
    }},
        Err(_) => return Ok(None),
    }


    let unwrapped_doc = car_doc.unwrap().unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_car(&unwrapped_doc);

    Ok(Some(customer_json))

}

/// Update a car's users list
pub async fn update_car_users(
    db: &Database,
    car_id: String,
    users: Vec<String>
) -> mongodb::error::Result<Option<Car>> {
    let collection = db.collection::<CarDocument>("cars");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();


    let car_doc = collection
        .find_one_and_update(
            doc! {"carID":  car_id   },
            doc! {"$set": doc! { 
                  "users":  users } },
            find_one_and_update_options,
        )
        .await?;

    if car_doc.is_none() {
        return Ok(None);
    }

    let unwrapped_doc = car_doc.unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_car(&unwrapped_doc);

    Ok(Some(customer_json))

}

/// Update a car, checking it's fule status
pub async fn update_car_fule_check(    
    db: &Database,
    car_id: String,
    event: Event) -> mongodb::error::Result<()> {
    
    let car = get_car(&db, car_id.clone()).await?;
    let car = match car {
        Some(value) => value,
        None => return Ok(()),
    };

    let events = get_car_events(&db, car_id.clone()).await?;

    let mut max_ev : DateTime = bson::DateTime::from_millis(0);
    let mut cnt = 0;
    for ev in events {
        if ev.name != "Refule" {
            continue;
        }
        cnt+= 1;
        if ev.date > event.date {
            if max_ev < ev.date {
                max_ev = ev.date;
            }
        }
    }

    let drives = get_car_drives(&db, car_id.clone()).await?;
    let mut fule = 0.0;
    for drive in drives {
        if drive.endTime > max_ev {
            fule += drive.fule;
        }
    }
    if !(bson::DateTime::from_millis(100) > max_ev) || cnt > 1 {
        let _ = update_car_fule(&db, car_id.clone(), -100.0).await?;
    }
    
    let _ = update_car_fule(&db, car_id.clone(), fule).await?;
    Ok(())
}

/// Get the 'specs' value of a car
pub async fn get_car_specs(    
    db: &Database,
    car_id: String,
) -> mongodb::error::Result<Option<Specs>> {
    
    let car = get_car(&db, car_id.clone()).await?;
    let car = match car {
        None => return Ok(None),
        Some(value) => value,
    };

    return Ok(Some(car.specs));
}

/// Update a car's fulebar value
pub async fn update_car_fule(
    db: &Database,
    car_id: String,
    fule: f32,
) -> mongodb::error::Result<Option<Car>> {
    let collection = db.collection::<CarDocument>("cars");
    let find_one_and_update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();


    let specs = get_car_specs(&db, car_id.clone()).await?;
    let specs = match specs {
        Some(value) => value,
        None => return Ok(None),
    };
    let max_fule = specs.tankCapacity.into();
    let fule_percentage : f64 = specs.currentFuleAmount.into();
    let fule : f64 = fule.into();

    let mut cur_fule : f64 = (fule_percentage * max_fule) / 100.0;
    cur_fule -= fule;

    if cur_fule < 0.0 {
        cur_fule = 0.0;
    }
    if cur_fule > max_fule {
        cur_fule = max_fule;
    }
    cur_fule = cur_fule / max_fule * 100.0;
    let fule_percentage : i32 = cur_fule as i32;

    let car_doc = collection
        .find_one_and_update(
            doc! {"carID":  car_id   },
            doc! { "$set": doc! { 
                        "specs":      doc! {
                        "licensePlateNumber": specs.licensePlateNumber,
                        "yearMade": specs.yearMade,
                        "tankCapacity": specs.tankCapacity,
                        "isElectric": specs.isElectric,
                        "currentFuleAmount": fule_percentage,
                        "model": specs.model,
                        }, }, },
            find_one_and_update_options,
        )
        .await?;

    if car_doc.is_none() {
        return Ok(None);
    }

    let unwrapped_doc = car_doc.unwrap();
    // transform ObjectId to String
    let customer_json = doc_to_car(&unwrapped_doc);

    Ok(Some(customer_json))

}

/// Deletes a car
pub async fn delete_car_by_id(
    db: &Database,
    id: String,
) -> mongodb::error::Result<Option<Car>> {
    let collection = db.collection::<CarDocument>("cars");

    // if you just unwrap,, when there is no document it results in 500 error.
    let car_doc = collection
        .find_one_and_delete(doc! {"carID": id.clone() }, None)
        .await?;
    if car_doc.is_none() {
        return Ok(None);
    }

    let unwrapped_doc = car_doc.unwrap();
    // transform ObjectId to String
    let car_json = doc_to_car(&unwrapped_doc);
    
    for user in &car_json.users {
        dbg!(&user, &id);
        let _ = update_user_cars(&db, user.clone(), Action::Delete, id.clone()).await;
    }
    let _ = update_user_cars(&db, car_json.owner.clone(), Action::Delete, id.clone()).await;

    Ok(Some(car_json))
}

/// Insert a given car to the database
pub async fn insert_car(
    db: &Database,
    owner_id: String,
    users: Vec<String>,
    locations: Vec<Point>,
    specs: Specs,
) -> mongodb::error::Result<String> {
    let collection = db.collection::<Document>("cars");

    let id = ( get_max_car_id(&db).await? + 1 ).to_string();

    let insert_one_result = collection
        .insert_one(
            doc! {
                  "owner":      owner_id,
                  "carID":      id,
                  "users":      users.clone(),
                  "drive":      Vec::<String>::new(),
                  "locations":  get_location_doc(locations),
                  "specs":      doc! {
                    "licensePlateNumber": specs.licensePlateNumber,
                    "yearMade": specs.yearMade,
                    "tankCapacity": specs.tankCapacity,
                    "isElectric": specs.isElectric,
                    "currentFuleAmount": specs.currentFuleAmount,
                    "model": specs.model,
                    },
                  "calendar":   Vec::<String>::new(),
        },
            None,
        )
        .await?;

    Ok(insert_one_result.inserted_id.to_string())
}

/// Get the owner field of a car
pub async fn get_car_owner(
    db: &Database,
    carID: String
) ->  mongodb::error::Result<Option<String>> {
    let car = get_car(&db, carID).await?;
    match car {
        Some(result) => return Ok(Some(String::from(result.owner))),
        None => return Ok(None)
    }
}

/// Checks whether a given user ID matches the car's owner ID
pub async fn is_owner(
    db: &Database,
    userID: String,
    carID: String
    ) -> mongodb::error::Result<bool> {
    let car_owner = get_car_owner(&db, carID.clone()).await?;

    match car_owner {
        Some(owner) => {
            return Ok(owner.eq(&userID))
        },
        None => return Ok(false),
    }
}