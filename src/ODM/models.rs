//! 
//! Documentation of the Models module.
//! Contains all the models needed for a TalMobil connection.
//!



use rocket::serde::{Serialize, Deserialize};
use bson::{oid::ObjectId, DateTime};

/// The Action enum, used to decide for updating functions
pub enum Action {
    Delete,
    Append
}
/* 
Models for the MongoDB operations
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDocument {
    /// The ID of the model.
    pub _id: Option<ObjectId>,
    /// The user's email address.
    pub email: String,
    /// The internal ID
    pub userID: String,
    /// The Car List
    pub cars: Vec<String>,
    /// Allow / Disallow to be invited to a car
    pub allowInvites: bool,
    /// The name of the user
    pub username: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    /// The ID of the model.
    pub _id: String,
    /// The user's email address.
    pub email: String,
    // The internal ID
    pub userID: String,
    // The Car List
    pub cars: Vec<String>,
    // Allow / Disallow to be invited to a car
    pub allow_invites: bool,
    // The name of the user
    pub username: String
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Drive {
    pub startTime: DateTime,
    pub endTime: DateTime,
    pub driverID: String,
    pub length: i32,
    pub fule: f32,
    pub startLocation: Vec<f32>,
    pub endLocation: Vec<f32>,
    pub passengers: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Specs {
    pub licensePlateNumber: String,
    pub yearMade: i32,
    pub tankCapacity: i32,
    pub isElectric: bool,
    pub currentFuleAmount: i32,
    pub model: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    location: Vec<f32>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Point {
    pub name: String,
    pub location: Vec<f32>,
    pub description: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub name: String,
    pub date: DateTime,
    pub description: String,
    pub createdBy: String,
    pub cost: i32,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Car {
    //// The ID of the model.
    pub _id: String,
    /// The internal ID
    pub carID: String,
    /// The internal ID of the owner
    pub owner: String,
    /// The users of the car
    pub users: Vec<String>,
    /// The drives vector
    pub drive: Vec<Drive>,
    /// The specifications of the car
    pub specs: Specs,
    /// The * locations of the car
    pub locations: Vec<Point>,
    /// The calendar ( Vec of events )
    pub calendar: Vec<Event>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CarDocument {
    /// The ID of the model.
    pub _id: Option<ObjectId>,
    /// The internal ID
    pub carID: String,
    /// The internal ID of the owner
    pub owner: String,
    /// The users of the car
    pub users: Vec<String>,
    /// The drives vector
    pub drive: Vec<Drive>,
    /// The specifications of the car
    pub specs: Specs,
    /// The * locations of the car
    pub locations: Vec<Point>,
    /// The calendar ( Vec of events )
    pub calendar: Vec<Event>
}
