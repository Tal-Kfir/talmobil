//! 
//! Documentation of the Database Utilities module.
//! Contains all the utilities needed for a TalMobil connection.
//!

use super::*;
use models::*;
use mongodb::Database;
use bson::{Document, doc};
use odm::get_all_users;

/// Converts a UserDocument to User
pub fn doc_to_user(document: &UserDocument) -> User {
    let _id =      &document._id;
    let email =    &document.email;
    let userID =   &document.userID;
    let cars =     &document.cars;
    let invites =  &document.allowInvites;
    let username = &document.username;
    
    // transform ObjectId to String
    let user_json = User {
        _id:     _id.unwrap().to_string(),
        email:   email.to_string(),
        userID:  userID.to_string(),
        cars:    cars.clone(),
        allow_invites: invites.to_owned(),
        username:      username.to_string()
    };
    user_json
}

/// Converts a CarDocument to Car
pub fn doc_to_car(document: &CarDocument) -> Car {
    // The ID of the model.
    let _id   = document._id;
    // The internal ID
    let carID = &document.carID;
    // The internal ID of the owner
    let owner = &document.owner;
    // The users of the car
    let users = &document.users;
    // The drives tuple
    let drive = &document.drive;
    // The specifications of the car
    let specs = &document.specs;
    // The * locations of the car
    let locations = &document.locations;
    // The calendar ( Vec of events )
    let calendar = &document.calendar;
    
    // transform ObjectId to String
    let car_json = Car {
        _id: _id.unwrap_or_default().to_string(),
        carID: carID.to_string(),
        owner: owner.to_string(),

        users:     users.clone(),
        drive:     drive.clone(),
        specs:     specs.clone(),
        locations: locations.clone(),
        calendar:  calendar.clone(),
    };
    car_json
}

/// Converts a vector of location (size of 2)
pub fn location_to_array(location: Vec::<f32>) -> Vec::<String> {
    let mut vector = vec![];

    for number in location {
        vector.push(number.to_string());
    }

    vector
}

/// Gets the document form of a location (Point)
pub fn get_location_doc(locations: Vec<Point>) -> std::vec::Vec<mongodb::bson::Document> {
    let mut locations_bson = Vec::<Document>::new();
    for location in locations {
        let _ = &locations_bson.push(doc!{
          "name": location.name,
          "location": location_to_array(location.location),
          "description": location.description,
      });
    }
  locations_bson
}

/// Get the max ID of users
pub async fn get_max_id(db: &Database) -> Result<i32, mongodb::error::Error> {
    
    let users_vector = get_all_users(&db).await;
    let users_vector = match users_vector {
        Err(value) => return Err(value),
        Ok(value) => value,
    };

    if users_vector.len() < 1 {
        return Ok(0);
    }

    let mut id_vector = vec![];

    for user in users_vector {
        id_vector.push(user.userID.parse::<i32>().unwrap())
    }

    Ok(*id_vector.iter().max().unwrap())

}