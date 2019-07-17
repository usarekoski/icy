use rumqtt::{
    MqttClient,
    MqttOptions,
    QoS,
    mqttoptions::SecurityOptions,
    Notification
};
use serde::{Serialize, Deserialize};

use crate::schema::measurements;
use crate::models;
use crate::db::DbConn;
use crate::settings::SETTINGS;

const MQTT_PORT: u16 = 1883;
const CHANNEL: &str = "sensor";

#[derive(Deserialize, Debug)]
struct Measurement {
    sensor_id: i32,
    temperature: f64,
    pressure: f64,
}

pub fn run(conn: DbConn) {
    let mqtt = &SETTINGS.mqtt;

    let mqtt_options = MqttOptions::new(mqtt.user.clone(), mqtt.server.clone(), MQTT_PORT)
        .set_security_opts(
            SecurityOptions::UsernamePassword(mqtt.user.clone(), mqtt.password.clone())
        )
        .set_clean_session(false);
    let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();

    mqtt_client.subscribe(CHANNEL, QoS::AtLeastOnce).unwrap();

    for notification in notifications {
        println!("{:?}", notification);
        match notification {
            Notification::Publish(msg) => {
                let measurement: Measurement = serde_json::from_slice(&msg.payload).unwrap();
                println!("{:?}", measurement);
                models::create_measurement(&conn, measurement.sensor_id, measurement.temperature, measurement.pressure);
            }
            _ => ()
        }
    }
}
