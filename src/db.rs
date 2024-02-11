use std::{fs::File, io::Read, sync::Mutex};

use mongodb::{bson::doc, options::{ClientOptions, ServerApi, ServerApiVersion}, Client};
use toml::Table;
use lazy_static::lazy_static;
use crate::structs::CaseElement;

lazy_static!{
    static ref CONNECTION_STRING : Mutex<String> = Mutex::new(String::new());
}

pub fn init()
{
    let mut file = File::open("mongodb.toml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let main_table = contents.parse::<Table>().unwrap();
    if let Some(connection) = main_table.get("connection") {
        let env_table = connection.as_table();
        if let Some(table) = env_table {
            if table.clone().get("connectionstring").is_some() {
                CONNECTION_STRING.lock().unwrap().push_str(table["connectionstring"].as_str().unwrap());
            } 
        } 
    }
}

pub async fn push_data(elements : Vec<CaseElement>)
{
    let client_options = match ClientOptions::parse(CONNECTION_STRING.lock().unwrap().as_str()).await {
        Ok(mut options) => {
            let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
            options.server_api = Some(server_api);
            options
        },
        Err(_) => {
            panic!("Failed to parse client options");
        }
    };
    let client = Client::with_options(client_options);
    client.unwrap()
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await;
    println!("Pinged your deployment. You successfully connected to MongoDB!");
}