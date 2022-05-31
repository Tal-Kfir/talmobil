#![allow(dead_code)]
extern crate serde_json;
extern crate rand;

use super::{*};
use rocket::response::content;
use std::collections::HashMap;
use std::result::Result;
use tera::{Context, Tera};
use config::config::ConfyConfig;
use mongodb::Database;
use chrono::prelude::{ * };
use chrono::{ Duration };
use rand::Rng;
use rocket::serde::{ Serialize, Deserialize };
use ODM::models::*;
use ODM::odm;

// The img item in home screen
const HOME_GRID_ITEM : &str =  "<a href=\"/view/{{carID}}\">\n \
                                    <div class=\"grid-item\">\n \
                                        {{name}}'s Car\n \
                                        <br>\n \
                                        <img class = \"grid-img\" src=\"{{carID}}.jpg\">\n \
                                    </div>\n \
                                </a>";

// The last item (+) in home screen
const HOME_GRID_LAST_ITEM : &str = "<a href=\"./add-car\">\n \
                                        <div class=\"grid-item\">\n \
                                            <img class = \"grid-last-img\" src=\"add-car.png\">\n \
                                        </div>\n \
                                    </a>";

// The EVENT display for the services screen                                    
const SERVICES_GRID_ITEM : &str = 	"<div class=\"inner-grid-item\">\n \
                                        <div class=\"inner-grid-text-display\">\n \
                                            {{eventName}}\n \
                                        </div>\n \
                                        <div class=\"inner-grid-date-display\">\n \
                                            <div class = \"text\">Last Event:</div>\n \
                                            <div class = \"date\">{{eventDate}}</div>\n \
                                        </div>\n \
                                    </div>";

// Calendar utils
const CALENDAR_OPTION_ITEM : &str = "<option>{{eventName}}</option>";

const CALENDAR_EVENT : &str = "{ eventName: '{{eventName}}', calendar: '{{calendar}}', color: '{{color}}', date: '{{date}}' },\n";

const CALENDAR_COLORS : [&str;8] = ["blue","orange","green","yellow","red","purple","aqua","white"];

/* 
Format files   

INPUT:  template to render + parameters to put
OUTPUT: the rendered template
*/
pub fn format_file(template : String, params : &mut HashMap<String, String>) -> Result<String, tera::Error> {
    let mut tera = Tera::new("resources/**/*").unwrap();

    //Get File name (e.g. memo.exe => memo)
    let template_stem : Vec<&str> = template.split(".").collect();
    let template_stem : String = template_stem[0].into();
    let template = template_stem + "/" + &template[..];

    // disable HTML autoescape
    tera.autoescape_on(vec![]);
    let mut ctx = Context::new();
    
    for (key, value) in params {
        ctx.insert(key, value);
    }

    tera.render(&template, &ctx)
}

/* 
Format files   

INPUT:  template to render + parameters to put
OUTPUT: the rendered template
*/
pub fn format_file_redo(template : String, params : Context) -> Result<String, tera::Error> {
    let mut tera = Tera::new("resources/**/*").unwrap();

    //Get File name (e.g. memo.exe => memo)
    let template_stem : Vec<&str> = template.split(".").collect();
    let template_stem : String = template_stem[0].into();
    let template = template_stem + "/" + &template[..];

    // disable HTML autoescape
    tera.autoescape_on(vec![]);

    tera.render(&template, &params)
}

/* 
Replace items   

INPUT:  the base text, the items to replace and the value of the result, wrapper param
OUTPUT: the rendered text
*/
fn replace_items(text: String, to_replace : Vec<&str>, values: Vec<&str>, with_wrapper: bool) -> String {
    let mut output = String::from(text);
    let len = to_replace.len();
    for i in 0..len {
        if with_wrapper {
            let pattern = &(String::from("{{") + to_replace[i] + "}}")[..];
            output = output.replace(pattern,values[i]);
        }
        else {
            output = output.replace(to_replace[i],values[i]);
        

        }

    }
    output
}

/* 
Render home screen 

INPUT:  the config of the system, the user accessing and the database access
OUTPUT: the rendered home screen
*/
pub async fn format_home(file: &ConfyConfig, user: String, db: &Database) -> Result<content::Html<String>, String> {

    let mut hmap = HashMap::<String, String>::new();
    
    let to_replace = vec!["name", "carID"];

    // Get user's info from the DB
    let cars = odm::get_cars_for_user(&db, user.clone()).await.unwrap();
    let owned_cars = odm::get_all_owned(&db, user.clone()).await.unwrap();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();

    // Element format - Owned Cars
    let mut element_owned = String::from("");
    for car in &owned_cars {
        let values = vec![&name[..], &car[..]];
        element_owned = element_owned + &replace_items(HOME_GRID_ITEM.into(), to_replace.clone(), values, true) + "\n";
    }
    // Whether or not the "Add Car" button is presentable
    if cars.len() < 5 {
        element_owned = element_owned + HOME_GRID_LAST_ITEM + "\n";
    }

    // Insert the vars to the render context table
    hmap.insert("name".into(), name);
    hmap.insert("gridOwned".into(), element_owned);
    
    // Element format - Unowned Cars
    let unowned_cars = odm::get_all_accessible(&db, user.clone()).await.unwrap();
    let mut element_unowned = String::from("");
    for car in unowned_cars {
        let car_owner = odm::get_car_owner(&db, car.clone()).await.unwrap().unwrap();
        let car_owner_name = odm::get_username_by_id(&db, car_owner.clone()).await.unwrap().unwrap();
        let values = vec![&car_owner_name[..], &car[..]];
        element_unowned = element_unowned + &replace_items(HOME_GRID_ITEM.into(), to_replace.clone(), values, true) + "\n";
    }

    // Insert the vars to the render context table
    hmap.insert("gridAccessible".into(), element_unowned);

    // CheckBox in the dropdown menu display
    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox".into(), checked.into());

    // Final formatting
    let format_temp = &file.file_home;
    let formatted = format_file(format_temp.to_string(), &mut hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}


/* 
Render add car screen 

INPUT:  the config of the system, the user accessing and the database access
OUTPUT: the rendered add car screen
*/
pub async fn format_add_car(file: &ConfyConfig, user: String, db: &Database) -> Result<content::Html<String>, String> {
    // Similar to home, getting info for base display
    let mut hmap = HashMap::<String, String>::new();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();
    hmap.insert("name".into(), name);

    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox".into(), checked.into());

    // Limiting the date option for today <=> 4 weeks ago
    let today : DateTime<Utc> = Utc::now();       // e.g. `2014-11-28T12:45:59.324310806Z`
    let today = today.date();
    let min_day = today.checked_sub_signed(Duration::weeks(4)).unwrap();

    let today = &today.to_string()[..];
    let length = today.len();
    let final_str : Vec<_> = today.split("U").collect();
    let final_str = final_str[0];
    hmap.insert("today".into(), final_str.into());

    let min_day = &min_day.to_string()[..];
    let length = min_day.len();
    let final_str : Vec<_> = min_day.split("U").collect();
    let final_str = final_str[0];
    hmap.insert("minDay".into(), final_str.into());

    // Final formatting
    let format_temp = &file.file_add_car;
    let formatted = format_file(format_temp.to_string(), &mut hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}

/* 
Render car view screen 

INPUT:  the config of the system, the user accessing, the car's id and the database access
OUTPUT: the rendered car view screen
*/
pub async fn format_car_view(file: &ConfyConfig, user: String, car_id: String, db: &Database) -> Result<content::Html<String>, String> {
    // Similar to home, getting info for base display
    let mut hmap = HashMap::<String, String>::new();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();
    hmap.insert("name".into(), name);
    hmap.insert("carID".into(), car_id.clone());

    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox".into(), checked.into());

    // Get the fule bar element
    let fule_bar = odm::get_car_fule_bar(&db, car_id.clone()).await.unwrap();

    match fule_bar {
        None => { hmap.insert("fuleBar".into(), "0".to_string()); },
        Some(value) => { hmap.insert("fuleBar".into(), value.to_string()); },
    }

    // Final formatting
    let format_temp = &file.file_car_view;
    let formatted = format_file(format_temp.to_string(), &mut hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}

/* 
Render car edit screen 

INPUT:  the config of the system, the user accessing, the car's id and the database access
OUTPUT: the rendered car edit screen
*/
pub async fn format_car_edit(file: &ConfyConfig, user: String, car_id: String, db: &Database) -> Result<content::Html<String>, String> {
    // Similar to home, getting info for base display
    let mut hmap = HashMap::<String, String>::new();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();
    hmap.insert("name".into(), name);
    hmap.insert("carID".into(), car_id.clone());

    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox".into(), checked.into());

    // Get car specs
    let car = odm::get_car(&db, car_id.clone()).await.unwrap().unwrap();
    let specs = car.specs;

    hmap.insert("model".into(), specs.model);
    hmap.insert("year".into(), specs.yearMade.to_string());
    hmap.insert("plate".into(), specs.licensePlateNumber);

    let electric = match specs.isElectric {
        false => "",
        true => "checked",
    };

    hmap.insert("electric".into(), electric.into());

    let is_owner = odm::is_owner(&db, user.clone(), car_id.clone()).await.unwrap();
    let owner = match is_owner {
        true => "",
        false => "disabled",
    };
    hmap.insert("owner".into(), owner.into());

    // Format the users fields
    let users = car.users;
    for i in 1..5 {
        if (i-1)>= users.len() {
            hmap.insert(format!("user{}",i), "".into());
        }
        else {
            let temp_user = odm::get_email_by_id(&db, users[(i-1)].clone()).await.unwrap().unwrap();
            hmap.insert(format!("user{}",i), temp_user);
        }
        
    }
    
    // Final formatting
    let format_temp = &file.file_car_edit;
    let formatted = format_file(format_temp.to_string(), &mut hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}

/* 
Render car services screen 

INPUT:  the config of the system, the user accessing, the car's id and the database access
OUTPUT: the rendered car services screen
*/
pub async fn format_services(file: &ConfyConfig, user: String, car_id: String, db: &Database) -> Result<content::Html<String>, String> {
    // Adding usual display items
    let mut hmap = HashMap::<String, String>::new();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();
    hmap.insert("name".into(), name);
    hmap.insert("carID".into(), car_id.clone());

    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox".into(), checked.into());

    // Getting the last event of each event type (name)
    let events = odm::get_car_events(&db, car_id.clone()).await.unwrap();

    let mut events_top = HashMap::<String, bson::DateTime>::new();

    for event in events {
        let value = events_top.entry(event.name).or_insert(event.date);
        *value = event.date.max(*value);
    }

    // Setting the name and date to the event's details
    let to_replace = vec!["eventName", "eventDate"];
    let mut element_events = String::from("");
    for (key, value) in events_top {
        let temp = value.to_string();
        let values = vec![&key[..], &temp[..10]]; // Fix borrowship isue - DONE
        element_events = element_events + &replace_items(SERVICES_GRID_ITEM.into(), to_replace.clone(), values, true) + "\n";
    }
    hmap.insert("gridEvents".into(), element_events);

    // Final formatting
    let format_temp = &file.file_car_services;
    let formatted = format_file(format_temp.to_string(), &mut hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}

/* 
Render car overview screen 

INPUT:  the config of the system, the user accessing, the car's id and the database access
OUTPUT: the rendered car overview screen
*/
pub async fn format_overview(file: &ConfyConfig, user: String, car_id: String, db: &Database) -> Result<content::Html<String>, String> {

    // usual display details
    let mut hmap = HashMap::<String, String>::new();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();
    hmap.insert("name".into(), name);
    hmap.insert("carID".into(), car_id.clone());

    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox".into(), checked.into());

    // Get car events
    let car = odm::get_car(&db, car_id.clone()).await.unwrap().unwrap();

    // Getting all users in the car
    let mut car_users_cloned = car.users.clone();
    car_users_cloned.push(car.owner);
    let car_users = &car_users_cloned;
    let mut html_users = String::from("");
    let length = car_users.len();

    // Setting the car users for the chart (eg user1, user2...)
    for i in 0..length {
        html_users.push_str("\"");
        html_users.push_str(
            &odm::get_username_by_id(
                &db,
                car_users[i].clone()
            ).await.unwrap().unwrap()[..]);
        html_users.push_str("\"");

        if (i+1) != length {
            html_users.push_str(", ");
        }
    }
    hmap.insert("users".into(), html_users.into());

    // Limiting event frame, a month ago to today
    let today : DateTime<Utc> = Utc::now();       // e.g. `2014-11-28T12:45:59.324310806Z`
    let today = today.date();
    let min_day = today.checked_sub_signed(Duration::weeks(4)).unwrap().and_time(NaiveTime::from_hms(0,0,0)).unwrap();

    // Filtering events by creator id
    let events = odm::get_car_events(&db, car_id.clone()).await.unwrap();
    let mut users_payments = String::from("");
    for user in car_users {
        let mut sum = 0;
        for event in &events {
            if &event.date < &bson::DateTime::from_chrono(min_day) {
                continue;
            }
            if &event.createdBy == user {
                sum += &event.cost;
            }
        }
        users_payments.push_str("\"");
        users_payments.push_str(&sum.to_string()[..]);
        users_payments.push_str("\",");
    }
    hmap.insert("money".into(), users_payments.into());

    // Choosing a random Hex color for the chart (#ff12ff...)
    let mut colors = String::from("");
    let mut rng = rand::thread_rng();
    for _ in car_users {
        let number : i32 = rng.gen_range(1..16) * rng.gen_range(1..16) * rng.gen_range(8..16) * rng.gen_range(8..16);
        colors.push_str("\"");
        colors.push_str("#");
        colors.push_str(&format!("{:x}", number)[..]);
        colors.push_str("\",");
    }
    hmap.insert("colors".into(), colors.into());

    // Final formatting
    let format_temp = &file.file_car_overview;
    let formatted = format_file(format_temp.to_string(), &mut hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}

/* 
Render car calendar screen 

INPUT:  the config of the system, the user accessing, the car's id and the database access
OUTPUT: the rendered car calendar screen
*/
pub async fn format_calendar(file: &ConfyConfig, user: String, car_id: String, db: &Database) -> Result<content::Html<String>, String> {
    // Usual base components
    let mut hmap = HashMap::<String, String>::new();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();
    hmap.insert("name".into(), name);
    hmap.insert("carID".into(), car_id.clone());

    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox".into(), checked.into());

    // Limiting the "Add event" option frame to a month ago to today
    let today : DateTime<Utc> = Utc::now();       // e.g. `2014-11-28T12:45:59.324310806Z`
    let today = today.date();
    let min_day = today.checked_sub_signed(Duration::weeks(4)).unwrap();

    let today = &today.to_string()[..];
    let length = today.len();
    let final_str : Vec<_> = today.split("U").collect();
    let final_str = final_str[0];
    hmap.insert("today".into(), final_str.into());

    let min_day = &min_day.to_string()[..];
    let length = min_day.len();
    let final_str : Vec<_> = min_day.split("U").collect();
    let final_str = final_str[0];
    hmap.insert("minDay".into(), final_str.into());

    // Filtering events by types
    let mut event_type_options = String::from("");
    let mut events = odm::get_car_events(&db, car_id.clone()).await.unwrap();

    // Adding Drives to the events
    let drives = odm::get_car_drives(&db, car_id.clone()).await.unwrap();
    for drive in drives {
        events.push(
            Event {
                cost: drive.length,
                createdBy: drive.driverID,
                name: "Drive".into(),
                date: drive.endTime,
                description: format!("{} passengers ", drive.passengers),
            }
        );
    }

    // Displaying events in the correct type and color
    let mut event_types = vec![];
    let mut event_element = String::from("");
    for event in events {
        // Replacing the Calendar option element
        let to_replace = vec!["eventName"];
        let event_type = event.name;
        if !event_types.contains(&event_type) {
            event_types.push(event_type.clone());
            let values = vec![&event_type[..]];
            let replaced = replace_items(CALENDAR_OPTION_ITEM.to_string(), to_replace, values, true);
            event_type_options += &(replaced + "\n");
        }

        // Assigning a color to each type if not assigned already
        let index = event_types.iter().position(|r| &r == &&event_type).unwrap();
        let to_replace = vec!["eventName", "calendar", "color", "date"];

        // Adding the description addon if there is one
        let mut name = odm::get_username_by_id(&db, event.createdBy).await.unwrap().unwrap();
        if &event.description != "" {
            name = name.to_string() + ", " + &event.description;
        }
        name = name + " - " + &event.cost.to_string();

        // Cutting the date to date only (removing time)
        let date = &(event.date.to_string())[..];
        let date = &date[0..11];

        let values = vec![&(name)[..], &event_type[..], CALENDAR_COLORS[index%8], date];

        let replaced = replace_items(CALENDAR_EVENT.into(), to_replace, values, true);

        event_element += &replaced[..];
    }
    hmap.insert("options".into(), event_type_options);
    hmap.insert("events".into(), event_element);

    // Example: { eventName: 'Lunch Meeting w/ Mark', calendar: 'Work', color: 'orange', date: '2022-02-08' },

    // Final formatting
    let format_temp = &file.file_calendar;
    let formatted = format_file(format_temp.to_string(), &mut hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}


/* 
Render car drive screen 

INPUT:  the config of the system, the user accessing, the car's id and the database access
OUTPUT: the rendered car drive screen
*/
pub async fn format_drive(file: &ConfyConfig, user: String, car_id: String, db: &Database) -> Result<content::Html<String>, String> {
    // Usual base screen items
    let mut hmap = HashMap::<String, String>::new();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();
    hmap.insert("name".into(), name);
    hmap.insert("carID".into(), car_id.clone());

    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox".into(), checked.into());

    // Final formatting
    let format_temp = &file.file_drive;
    let formatted = format_file(format_temp.to_string(), &mut hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Marker {
    name: String,
    locationLat: f32,
    locationLon: f32,
    description: String,
    owner: String,
}

/* 
Render car locations screen 

INPUT:  the config of the system, the user accessing, the car's id and the database access
OUTPUT: the rendered car locations screen
*/
pub async fn format_locations(file: &ConfyConfig, user: String, car_id: String, db: &Database) -> Result<content::Html<String>, String> {
    // Usual base screen items
    let mut hmap = Context::new();
    let name = odm::get_username_by_id(&db, user.clone()).await.unwrap().unwrap();
    hmap.insert("name", &name);
    hmap.insert("carID", &car_id);

    let checked = match odm::get_user_invites(&db, user.clone()).await.unwrap_or_default().unwrap() {
        false => "",
        true => "checked",
    };
    hmap.insert("checkBox", &checked);

    let locations = odm::get_car_locations(&db, car_id.clone()).await.unwrap();
    let is_owner = odm::is_owner(&db, user.clone(), car_id.clone()).await.unwrap();

    let mut owner = String::from("");
    if !is_owner {
        owner += "disabled";
    }

    let mut markers = vec![];

    let last_drive = odm::get_car_last_drive(&db, car_id.clone()).await.unwrap();
    match last_drive {
        None => {
            markers.push(
                Marker {
                    name: "Current Car Location".into(),
                    description: "No Location Provided - 0,0 default".into(),
                    locationLat: 0.0,
                    locationLon: 0.0,
                    owner: owner.clone()
                }
            );
        },
        Some(value) => {
            markers.push(
                Marker {
                    name: "Current Car Location".into(),
                    description: "".into(),
                    locationLat: value.endLocation[1],
                    locationLon: value.endLocation[0],
                    owner: owner.clone()
                }
            );
        },
    }
    for location in locations {
        
        let marker = Marker {
            name: location.name,
            description: location.description,
            locationLat: location.location[1],
            locationLon: location.location[0],
            owner: owner.clone(),
        };
        markers.push(marker);
    }
    hmap.insert("markers", &markers);

    // Final formatting
    let format_temp = &file.file_locations;
    let formatted = format_file_redo(format_temp.to_string(), hmap);
    match formatted {
        Ok(value) => {
            return Ok(content::Html(value)) },
        Err(err) => return Err(format!("Failed to parse {}, error: {}", &format_temp, err.to_string())),
    }
}