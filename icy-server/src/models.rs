use chrono::NaiveDateTime;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::Serialize;

use crate::schema::*;

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name="sensors"]
pub struct Sensor {
    pub id: i32,
    pub description: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Sensor)]
#[table_name="measurements"]
pub struct Measurement {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub sensor_id: i32,
    pub temperature: Option<f64>,
    pub pressure: Option<f64>,
}

#[derive(Insertable)]
#[table_name="measurements"]
pub struct NewMeasurement<> {
    pub timestamp: NaiveDateTime,
    pub sensor_id: i32,
    pub temperature: f64,
    pub pressure: f64,
}

pub fn create_measurement(conn: &PgConnection, sensor_id: i32, temperature: f64, pressure: f64) -> Measurement {
    let new_measurement = NewMeasurement {
        timestamp: Utc::now().naive_utc(),
        temperature,
        pressure,
        sensor_id,
    };

    diesel::insert_into(measurements::table)
        .values(&new_measurement)
        .get_result(conn)
        .expect("error saving measurement")
}
