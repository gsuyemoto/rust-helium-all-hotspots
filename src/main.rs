extern crate pretty_env_logger;
#[macro_use] extern crate log;

use serde::{Deserialize, Serialize};
use log::{info, trace};

#[derive(Debug, Serialize, Deserialize)]
struct HotspotData {
    data: Vec<Hotspot>,
    cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Hotspot {
    lng: Option<f64>,
    lat: Option<f64>,
    timestamp_added: Option<String>,
    status: Option<HotspotStatus>,
    reward_scale: Option<f32>,
    payer: Option<String>,
    owner: Option<String>,
    nonce: i32,
    name: Option<String>,
    mode: Option<String>,
    location_hex: Option<String>,
    location: Option<String>,
    last_poc_challenge: Option<i32>,
    last_change_block: i32,
    geocode: HotspotGeocode,
    gain: i32,
    elevation: i32,
    block_added: i32,
    block: i32,
    address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HotspotStatus {
    timestamp: Option<String>,
    online: Option<String>,
    listen_addrs: Option<Vec<String>>,
    height: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HotspotGeocode {
    short_street: Option<String>,
    short_state: Option<String>,
    short_country: Option<String>,
    short_city: Option<String>,
    long_street: Option<String>,
    long_state: Option<String>,
    long_country: Option<String>,
    long_city: Option<String>,
    city_id: Option<String>,
}

static HELIUM_API_HOTSPOTS: &str = "https://api.helium.io/v1/hotspots?cursor=";
static MONGO_REST: &str = "http://user:pass@some.ip:port/collection";

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut c: Option<String> = Some(String::new());

    trace!("Entering main loop...");
    while let Some(cursor) = c {
        trace!("Get hotspot data from Helium servers");

        let helium_url = format!("{}{}", HELIUM_API_HOTSPOTS, cursor);
        let resp: HotspotData = reqwest::get(helium_url)
            .await
            .expect("Problem sending GET request")
            .json()
            .await
            .expect("Problem parsing JSON");

        info!("cursor: {:?}", &resp.cursor);
        c = resp.cursor.clone();

        // insert hotspot data one cursor at a time into Rest based MongoDB
        let client = reqwest::Client::new();
        let resp = client.post(MONGO_REST)
            .json(&resp)
            .send()
            .await
            .expect("Problem POSTing to Mongo REST DB");

        trace!("Successfully POSTed hotspot data");
    }
}      
