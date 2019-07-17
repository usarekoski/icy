#include <Wire.h>
#include <SPI.h>
#include <Adafruit_BMP280.h>
#include <ESP8266WiFi.h>
#include <PubSubClient.h>
#include <ArduinoJson.h>
#include <FS.h>

const int WIFI_TIMEOUT = 15000;
const int SLEEP_MICROSECONDS = 60000000;
const char* CONFIG_FILENAME = "/config.json";

// From https://arduino-esp8266.readthedocs.io/en/latest/Troubleshooting/debugging.html
#ifdef DEBUG_ESP_PORT
  #define DEBUG_PRINTF(...) DEBUG_ESP_PORT.printf(__VA_ARGS__)
  #define DEBUG_PRINT(...) DEBUG_ESP_PORT.print(__VA_ARGS__)
#else
  #define DEBUG_PRINTF(...)
  #define DEBUG_PRINT(...)
#endif

enum class Error: int {
  success,
  wifi,
  mqtt,
  bmp,
  config,
};

struct Measurements {
  float temperature;
  float pressure;
};

struct Config {
  char wifi_ssid[64];
  char wifi_password[64];
  char mqtt_server[64];
  char mqtt_username[64];
  char mqtt_password[64];
  int sensor_id;
};

// Don't forget to change the capacity to match your requirements.
// Use arduinojson.org/v6/assistant to compute the capacity.
typedef StaticJsonDocument<512> JsonConfig;

Adafruit_BMP280 bmp;
WiFiClient client;
PubSubClient mqtt_client(client);
Config config;

// All errors from mqtt lib, error codes map to array index.
const char* MQTT_ERRORS[] = {
  "CONNECTION_TIMEOUT",
  "CONNECTION_LOST",
  "CONNECT_FAILED",
  "DISCONNECTED",
  "CONNECTED",
  "CONNECT_BAD_PROTOCOL",
  "CONNECT_BAD_CLIENT_ID",
  "CONNECT_UNAVAILABLE",
  "CONNECT_BAD_CREDENTIALS",
  "CONNECT_UNAUTHORIZED"
};

Error copy_config_string(char* dest, JsonConfig* doc, const char* property, size_t dest_capacity) {
  auto obj = doc->as<JsonObject>();
  auto copied = strlcpy(dest, obj[property], dest_capacity);
  if (copied >= dest_capacity) {
    DEBUG_PRINTF("Config property %s is too long!\n", property);
    return Error::config;
  }
  return Error::success;
}

Error loadConfig() {
  File file = SPIFFS.open(CONFIG_FILENAME, "r");
  if (!file) {
    DEBUG_PRINT("Failed to open config file\n");
    return Error::config;
  }

  // Allocate a temporary JsonDocument
  JsonConfig doc;

  DeserializationError error = deserializeJson(doc, file);
  if (error) {
    DEBUG_PRINT("Failed to deserialize config file\n");
    return Error::config;
  }

  auto e = copy_config_string(config.wifi_ssid, &doc, "wifi_ssid", sizeof(config.wifi_ssid));
  if (e != Error::success) goto err;
  e = copy_config_string(config.wifi_password, &doc, "wifi_password", sizeof(config.wifi_password));
  if (e != Error::success) goto err;
  e = copy_config_string(config.mqtt_server, &doc, "mqtt_server", sizeof(config.mqtt_server));
  if (e != Error::success) goto err;
  e = copy_config_string(config.mqtt_username, &doc, "mqtt_username", sizeof(config.mqtt_username));
  if (e != Error::success) goto err;
  e = copy_config_string(config.mqtt_password, &doc, "mqtt_password", sizeof(config.mqtt_password));
  if (e != Error::success) goto err;
  config.sensor_id = doc["sensor_id"];

err:
  file.close();
  return e;
}

Error connectWifi() {
  unsigned long wifiConnectStart = millis();

  WiFi.mode(WIFI_STA);
  WiFi.begin(config.wifi_ssid, config.wifi_password);

  while (WiFi.status() != WL_CONNECTED) {
    if (WiFi.status() == WL_CONNECT_FAILED) {
      DEBUG_PRINTF("Failed to connect to WiFi. Please verify credentials.\n");
      delay(10000);
    }

    delay(500);
    DEBUG_PRINTF("...");
    // Try a few times.
    if (millis() - wifiConnectStart > WIFI_TIMEOUT) {
      DEBUG_PRINTF("Failed to connect to WiFi\n");
      return Error::wifi;
    }
  }

  return Error::success;
}

void checkMQTTError() {
  int state = mqtt_client.state();
  unsigned int err_idx = state + 4;
  if (err_idx >= 0 && err_idx < (sizeof(MQTT_ERRORS) / sizeof(char*))) {
    DEBUG_PRINTF("MQTT error: %s\n", MQTT_ERRORS[err_idx]);
  } else {
    DEBUG_PRINTF("Unknown MQTT error: %i\n", state);
  }
}

Error connectMQTT() {
  mqtt_client.setServer(config.mqtt_server, 1883);

  if (mqtt_client.connect(config.mqtt_username, config.mqtt_username, config.mqtt_password)) {
    return Error::success;
  } else {
    DEBUG_PRINT("Connecting to mqtt broker failed.\n");
    checkMQTTError();
    return Error::mqtt;
  }
}

Error sendMQTT(Measurements measurements) {
  const int capacity = JSON_OBJECT_SIZE(3);
  StaticJsonDocument<capacity> doc;
  doc["temperature"] = measurements.temperature;
  doc["pressure"] = measurements.pressure;
  doc["sensor_id"] = config.sensor_id;

  char output[128];
  serializeJson(doc, output);

  DEBUG_PRINTF("message = %s", output);
  if (!mqtt_client.publish("sensor", output)) {
    checkMQTTError();
    return Error::mqtt;
  }
  return Error::success;
}

Error setupBMP() {
  if (!bmp.begin()) {
    DEBUG_PRINTF("Could not find a valid BMP280 sensor, check wiring!\n");
    return Error::bmp;
  }

  // Default settings from datasheet.
  bmp.setSampling(Adafruit_BMP280::MODE_NORMAL,     // Operating Mode.
                  Adafruit_BMP280::SAMPLING_X2,     // Temp. oversampling
                  Adafruit_BMP280::SAMPLING_X16,    // Pressure oversampling
                  Adafruit_BMP280::FILTER_X16,      // Filtering.
                  Adafruit_BMP280::STANDBY_MS_500); // Standby time.

  return Error::success;
}

Measurements readMeasurements() {
  return Measurements {
    .temperature = bmp.readTemperature(),
    .pressure = bmp.readPressure(),
  };
}

void run() {
  if (!SPIFFS.begin()) {
    DEBUG_PRINT("Mounting SPIFFS failed\n");
    return;
  }

  auto e = loadConfig();
  if (e != Error::success) return;

  e = setupBMP();
  if (e != Error::success) return;

  e = connectWifi();
  if (e != Error::success) return;

  e = connectMQTT();
  if (e != Error::success) return;

  auto measurements = readMeasurements();
  DEBUG_PRINTF("Temperature = %f *C\n", measurements.temperature);
  DEBUG_PRINTF("Pressure = %f Pa\n", measurements.pressure);

  e = sendMQTT(measurements);
  if (e != Error::success) return;

  mqtt_client.disconnect();
}

void setup() {
  #ifdef DEBUG_ESP_PORT
    DEBUG_ESP_PORT.begin(9600);  
  #endif

  run();

  DEBUG_PRINTF("Going into deep sleep for %i seconds.\n", SLEEP_MICROSECONDS / 1000000);
  ESP.deepSleep(SLEEP_MICROSECONDS);
}

void loop() {}
