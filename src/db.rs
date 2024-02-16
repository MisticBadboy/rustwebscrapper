use std::{fs::File, io::{Read, Stdout, Write}, io, ops::Deref, sync::{Arc, Mutex}, thread, time::Duration};

use chrono::format;
use mongodb::{bson::doc, options::{ClientOptions, Compressor, DropDatabaseOptions, ServerApi, ServerApiVersion}, Client};
use tokio::io::{stdout, AsyncWriteExt};
use toml::Table;
use lazy_static::lazy_static;
use crate::structs::CaseElement;

lazy_static!{
    static ref CONNECTION_STRING : Mutex<String> = Mutex::new(String::new());
    static ref CLIENT : Mutex<Option<Client>> = Mutex::new(None);
}

pub async fn init() -> Option<()>
{
    let running_flag = Arc::new(Mutex::new(true));
    let running_flag_clone = Arc::clone(&running_flag);
    let display_thread = thread::spawn(move || 
        { 
            let mut dots = 0;
            let stdout = io::stdout();
            while *running_flag_clone.lock().unwrap() {
                if dots < 3 {
                    print!("\rConnecting {}", ".".repeat(dots + 1));
                } else {
                    print!("\rConnecting");
                }
                stdout.lock().flush().unwrap();
                thread::sleep(std::time::Duration::from_millis(500));
                dots = (dots + 1) % 3;
            }
            print!("\r{}{}", " ".repeat(12), "\r");
            stdout.lock().flush().unwrap();
        }
    );
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
    let client_options = match ClientOptions::parse(CONNECTION_STRING.lock().unwrap().as_str()).await {
        Ok(mut options) => {
            let server_api = ServerApi::builder().version(ServerApiVersion::V1).strict(true).deprecation_errors(true).build();
            let compressors = vec![
                Compressor::Snappy
            ];
            options.server_api = Some(server_api);
            options.compressors = Some(compressors);
            options
        },
        Err(_) => {
            panic!("Failed to parse client options");
        }
    };
    let client = Client::with_options(client_options).unwrap();
    CLIENT.lock().unwrap().clone_from(&Some(client));
    *running_flag.lock().unwrap() = false;
    display_thread.join().unwrap();
    println!("Client initialized");
    Some(())
}

pub async fn push_data(elements : Vec<CaseElement>)
{
    println!("Getting client ready!");
    let client_guard = CLIENT.lock().unwrap();
    let client = match &*client_guard {
        Some(c) => c,
        None => panic!("Client not initialized"),
    };
    println!("Client ready");
    println!("Creating tables and collections");
    // let _ = client.database("cases").drop(Some(DropDatabaseOptions::));
    // let _ = client.database("knifes").collection("knifes");
    // let _ = client.database("skins").collection("skins");
    let casesTable = client.database("cases").collection("cases");
    let knifesTable = client.database("knifes").collection("knifes");
    let skinTable = client.database("skins").collection("skins");
    // let mut Cases = Vec::new();
    // let mut Knifes = Vec::new();
    // let mut Skins = Vec::new();
    
    // println!("Adding cases, knifes and skins");
    // for i in elements
    // {
    //     let clone = i.clone();
    //     Cases.push(clone);
    //     for knife in i.clone().knifes.unwrap().items{
    //         Knifes.extend(knife)
    //     };
    //     Skins.push(i.clone().items);
    // }

    println!("Pushing cases");
    for case in elements {
        println!("Pushing case : {:?}", case.clone().name.unwrap());
        let cases_doc = doc! {
            "value": case.clone()
        };
        let result = match casesTable.insert_one(cases_doc, None).await
        {
            Ok(res) => res,
            Err(e) => panic!("Paniced with {:#?}", e)
        };
        let cases_id = result.inserted_id.as_object_id().unwrap();
        
        println!("      Pushing case : {:?} knifes", case.clone().name.unwrap());
        for knife in case.clone().knifes.unwrap().items
        {
            let knife_doc = doc! {
                "value": knife,
                "caseid" : cases_id
            };
            let result = match knifesTable.insert_one(knife_doc, None).await
            {
                Ok(res) => res,
                Err(e) => panic!("Paniced with {:#?}", e)
            };
        }
        
        println!("      Pushing case : {:?} skins", case.clone().name.unwrap());
        for skin in case.clone().items
        {
            let skin_doc = doc! {
                "value": skin,
                "caseid" : cases_id
            };
            let result = match skinTable.insert_one(skin_doc, None).await
            {
                Ok(res) => res,
                Err(e) => panic!("Paniced with {:#?}", e)
            };
        }
    }
}