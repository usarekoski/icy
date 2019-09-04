#![feature(proc_macro_hygiene, decl_macro)]

extern crate openssl;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;
use diesel::prelude::*;
use rocket::Rocket;
use rocket::fairing::AdHoc;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::{json::Json, templates::Template};
use serde::Serialize;
use std::collections::HashMap;
use std::{thread, time::Duration};

use self::db::DbConn;
use self::models::*;
use self::schema::*;
use self::settings::{SETTINGS, ROCKET_CONFIG};

mod db;
mod models;
mod mqtt_listener;
mod schema;
mod settings;

#[derive(Serialize, Debug)]
struct SensorWithLatestData {
    description: String,
    temperature: Option<f64>,
    pressure: Option<f64>,
    last_updated: String,
}

#[get("/")]
fn index(conn: DbConn) -> Template {
    let sensors: Vec<Sensor> = sensors::table.load(&*conn).unwrap();
    let sensors_with_data: Vec<SensorWithLatestData> = sensors
        .iter()
        .map(|sensor| {
            let res = Measurement::belonging_to(sensor)
                .order(measurements::timestamp.desc())
                .first::<Measurement>(&*conn);

            match res {
                Ok(measurement) => Some(SensorWithLatestData {
                    description: sensor.description.to_string(),
                    temperature: measurement.temperature,
                    pressure: measurement.pressure,
                    last_updated: measurement
                        .timestamp
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                }),
                Err(diesel::result::Error::NotFound) => None,
                e => panic!(e),
            }
        })
        .flatten()
        .collect();
    let mut context: HashMap<&str, &Vec<_>> = HashMap::new();
    context.insert("sensors", &sensors_with_data);
    Template::render("index", &context)
}

// #[get("/api")]
// fn measurements(conn: MeasurementsDbConn) -> Json<String> {
//     let res = measurements
//         .limit(5)
//         .load::<Measurement>(&*conn)
//         .expect("error in db query");
//     "test"
// }


fn start_mqtt_listener(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = DbConn::get_one(&rocket).expect("database connection");

    thread::spawn(move || {
        mqtt_listener::run(conn);
    });

    Ok(rocket)
}

fn mount_static_files(rocket: Rocket) -> Result<Rocket, Rocket> {
    let path = rocket.config().get_str("static_dir").unwrap().to_string();
    Ok(rocket.mount("/", StaticFiles::from(path)))
}

fn main() {
    let rocket = rocket::custom((*ROCKET_CONFIG).clone())
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Mqtt Listener", start_mqtt_listener))
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("Static Files", mount_static_files))
        .mount("/", routes![index]);

    rocket.launch();
}
