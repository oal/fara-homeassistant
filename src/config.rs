use dotenvy::dotenv;

pub(crate) struct AppConfig {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) pta: String,
    pub(crate) client_id: String,

    pub(crate) mqtt_host: String,
    pub(crate) mqtt_port: u16,
}

impl AppConfig {
    pub(crate) fn from_env() -> Self {
        let _ = dotenv();
        use std::env::var;
        let port = var("MQTT_PORT").or::<()>(Ok("1883".to_string())).unwrap();
        let mqtt_port = port.parse::<u16>().expect("MQTT_PORT must be a valid port number.");
        Self {
            username: var("FARA_USERNAME").expect("FARA_USERNAME not set"),
            password: var("FARA_PASSWORD").expect("FARA_PASSWORD not set"),
            pta: var("FARA_PTA").expect("FARA_PTA not set"),
            client_id: var("FARA_CLIENT_ID").expect("FARA_CLIENT_ID not set"),

            mqtt_host: var("MQTT_HOST").expect("MQTT_HOST not set"),
            mqtt_port,
        }
    }
}
