extern crate linux_embedded_hal as hal;

use bme280::BME280;
use hal::{Delay, I2cdev};
use rumqtt::{mqttoptions::SecurityOptions, MqttClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::{thread, time};
use config::{Config, ConfigError, File};

const MQTT_PORT: u16 = 1883;

#[derive(Debug, Deserialize)]
struct Settings {
    mqtt: MqttSettings,
}

#[derive(Debug, Deserialize)]
struct MqttSettings {
    server: String,
    user: String,
    password: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut c = Config::new();

        #[cfg(debug_assertions)]
        c.merge(File::with_name("config.toml"))?;

        #[cfg(not(debug_assertions))]
        c.merge(File::with_name("/usr/local/etc/icy-pi-client/icy-pi-client.toml"))?;

        c.try_into()
    }
}

#[derive(Serialize, Debug)]
pub struct Measurement {
    sensor_id: i32,
    temperature: f64,
    pressure: f64,
}

fn main() {
    let settings: Settings = Settings::new().unwrap();
    println!("{:?}", settings);

    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();
    let mut bme280 = BME280::new_primary(i2c_bus, Delay);
    bme280.init().unwrap();

    let mqtt_options = MqttOptions::new(&settings.mqtt.user, settings.mqtt.server, MQTT_PORT)
        .set_security_opts(SecurityOptions::UsernamePassword(
            settings.mqtt.user.into(),
            settings.mqtt.password.into(),
        ))
        .set_clean_session(false);
    let (mut mqtt_client, _notifications) = MqttClient::start(mqtt_options).unwrap();

    loop {
        let measurements = bme280.measure().unwrap();

        println!("Temperature = {} deg C", measurements.temperature);
        println!("Pressure = {} pascals", measurements.pressure);

        let measurement = Measurement {
            temperature: measurements.temperature.into(),
            pressure: measurements.pressure.into(),
            sensor_id: 2,
        };
        let payload = serde_json::to_vec(&measurement).unwrap();
        mqtt_client
            .publish("sensor", QoS::AtLeastOnce, false, payload)
            .unwrap();
        thread::sleep(time::Duration::from_secs(20));
    }
}
