extern crate pretty_env_logger;
#[macro_use] extern crate log;
use log::{info, trace};

extern crate dotenv;
use dotenv::dotenv;
use std::env;

use serde::{Deserialize, Serialize};
use mysql_async::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct HotspotData {
    data: Vec<Hotspot>,
    cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Hotspot {
    address: Option<String>,
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
static SQL_TABLE: &str = r"
CREATE TABLE hotspot (
    address varchar(60) primary key,
    lng float,
    lat float,
    timestamp_added text,
    reward_scale float,
    payer text,
    owner text,
    nonce int not null,
    name text,
    mode text,
    location_hex text,
    location text,
    last_poc_challenge int,
    last_change_block int not null,
    gain int not null,
    elevation int not null,
    block_added int not null,
    block int not null)
";
static SQL_TABLE_GEO: &str = r"
CREATE TABLE hotspot_geo (
    address varchar(60) primary key,
    short_street text,
    short_state text,
    short_country text,
    short_city text,
    long_street text,
    long_state text,
    long_country text,
    long_city text,
    city_id text
)";
static SQL_TABLE_STATUS: &str = r"
CREATE TABLE hotspot_status (
    timestamp text,
    online text,
    listen_addrs text,
    height int
)";
static SQL_INSERT: &str = r"
INSERT IGNORE INTO hotspot (
    address,
    lng,
    lat,
    timestamp_added,
    reward_scale,
    payer,
    owner,
    nonce,
    name,
    mode,
    location_hex,
    location,
    last_poc_challenge,
    last_change_block,
    gain,
    elevation,
    block_added,
    block)
VALUES (
    :address,
    :lng,
    :lat,
    :timestamp_added,
    :reward_scale,
    :payer,
    :owner,
    :nonce,
    :name,
    :mode,
    :location_hex,
    :location,
    :last_poc_challenge,
    :last_change_block,
    :gain,
    :elevation,
    :block_added,
    :block)
";

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let mut new_cursor: Option<String> = Some(String::new());
    let mut hotspots: Vec<Hotspot> = Vec::new();

    trace!("Entering main loop...");
    while let Some(cursor) = new_cursor {
        trace!("Getting hotspot data from Helium servers");

        let helium_url = format!("{}{}", HELIUM_API_HOTSPOTS, cursor);
        let mut resp: HotspotData = reqwest::get(helium_url)
            .await
            .expect("Problem sending GET request")
            .json()
            .await
            .expect("Problem parsing JSON");

        new_cursor = resp.cursor.clone();
        hotspots.append(&mut resp.data);

        trace!("Added hotspot cursor to vec");
    }

    trace!("Done getting all hotspots from Helium chain");

    let database_url = format!("mysql://{}:{}@{}:{}/{}"
                               , dotenv::var("user").unwrap()
                               , dotenv::var("password").unwrap()
                               , dotenv::var("host").unwrap() 
                               , dotenv::var("port").unwrap()
                               , dotenv::var("database").unwrap());

    let pool = mysql_async::Pool::from_url(database_url)
        .expect("Unable to create db pool");
    let mut conn = pool
        .get_conn()
        .await
        .expect("Unable to connect to MySQL DB");

    // Create a temporary table
    //SQL_TABLE
    //    .ignore(&mut conn)
    //    .await
    //    .expect("Unable to create temp table");

    // Save hotspot data
    SQL_INSERT
        .with(hotspots.iter().map(|hotspot| params! {
            "address" => hotspot.address.as_ref(),
            "lng" => hotspot.lng,
            "lat" => hotspot.lat,
            "timestamp_added" => hotspot.timestamp_added.as_ref(),
            "reward_scale" => hotspot.reward_scale,
            "payer" => hotspot.payer.as_ref(),
            "owner" => hotspot.owner.as_ref(),
            "nonce" => hotspot.nonce,
            "name" => hotspot.name.as_ref(),
            "mode" => hotspot.mode.as_ref(),
            "location_hex" => hotspot.location_hex.as_ref(),
            "location" => hotspot.location.as_ref(),
            "last_poc_challenge" => hotspot.last_poc_challenge,
            "last_change_block" => hotspot.last_change_block,
            "gain" => hotspot.gain,
            "elevation" => hotspot.elevation,
            "block_added" => hotspot.block_added,
            "block" => hotspot.block
        }))
        .batch(&mut conn)
        .await
        .expect("Error inserting new rows");
}      
