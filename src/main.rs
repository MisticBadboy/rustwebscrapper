use std::{env, fs};
use std::io::Write;
use chrono::{DateTime, Local};
use lazy_static::lazy_static;
use reqwest;

use structs::{CaseElement, Items};
use tokio::{self, task::JoinError};
mod structs;
mod db;

#[tokio::main]
async fn main() {
    if let Err(_) = run().await {
        println!("An error occurred.");
    }
    db::init();
    db::push_data(Vec::new()).await;
}

lazy_static!
{
    static ref EXEC_TIME : std::sync::RwLock<Vec<DateTime<Local>>> = std::sync::RwLock::new(Vec::new());
}

async fn run() -> Result<(), reqwest::Error> {
    let res = reqwest::get("https://csgostash.com/containers/skin-cases").await?;
    let body = res.text().await?;
    let scraper_doc = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("body > div.container.main-content > div:nth-child(7) > div").unwrap();
    let html_products = scraper_doc.select(&selector);
    let mut case_elements : Vec<structs::CaseElement> = Vec::new();
    for element in html_products
    {
        let url = element
        .select(&scraper::Selector::parse("a").unwrap())
        .next()
        .and_then(|url| url.value().attr("href"))
        .map(str::to_owned);
        
        let name = element
        .select(&scraper::Selector::parse("a > h4").unwrap())
        .next()
        .map(|name| name.text().collect::<String>());

        let image = element
        .select(&scraper::Selector::parse("a > img").unwrap())
        .next()
        .and_then(|img| img.value().attr("src"))
        .map(str::to_owned);

        let price = element
        .select(&scraper::Selector::parse("a > div > p").unwrap())
        .next()
        .map(|p| p.text().collect::<String>());

        if (price.is_none() && image.is_none() && name.is_none() && url.is_none()) || name.clone().unwrap().to_lowercase().contains("collection")
        {
            continue;
        }

        case_elements.push(CaseElement::new(url, image, name, price));
    }

    case_elements = start_case_parser(case_elements, false).await.unwrap();

    let mut case_knife_elements : Vec<CaseElement> = case_elements.clone().into_iter().map(|x| x.knifes).filter_map(|x| match x{
        Some(x) => Some(*x),
        None => None

    }).collect();

    case_knife_elements = start_case_parser(case_knife_elements, true).await.unwrap();

    let mut case_elements_with_knifes = Vec::new();
    for i in case_elements.clone().into_iter().enumerate()
    {
        case_elements_with_knifes.push(case_elements[i.0].clone() + case_knife_elements[i.0].clone());
    }

    Ok(())
}

async fn start_case_parser(case_elements: Vec<CaseElement>, knife: bool) -> Result<Vec<CaseElement>, JoinError> {
    let mut threads = Vec::new();
    let mut return_case_elements = Vec::new();
    for (index, item) in case_elements.into_iter().enumerate() {
        println!("Thread {index} started!");
        EXEC_TIME.write().unwrap().push(Local::now());
        let handle = tokio::spawn(async move {
            case_parser(index, item, knife).await
        });
        threads.push(handle);
    }
    let mut count = 0; 
    let mut missed_items = String::new();
    for (index, handle) in threads.into_iter().enumerate() 
    {
        return_case_elements.push(handle.await?.unwrap());
        if return_case_elements[index].clone().items.unwrap().len() == 0
        {
            count += 1;
            let mut formatted_string = Vec::new();
            write!(&mut formatted_string, "{:#?}", return_case_elements).expect("Failed to format");
            missed_items.push_str(std::str::from_utf8(&formatted_string).unwrap());
        }
    }
    if let Err(e) = fs::create_dir_all("./log") {
        panic!("Failed to create directory: {}", e);
    }

    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            panic!("Failed to get current executable path: {}", e);
        }
    };

    // Get the directory of the executable
    let exe_dir = match exe_path.parent() {
        Some(dir) => dir,
        None => {
            panic!("Failed to get parent directory of executable");
        }
    };

    // Create the log directory if it doesn't exist in the same directory as the executable
    let log_dir = exe_dir.join("log");
    if let Err(e) = fs::create_dir_all(&log_dir) {
        panic!("Failed to create log directory: {}", e);
    }

    // Open or create the file for appending in the log directory
    let log_file = log_dir.join("cases.txt");
    let mut file = match fs::OpenOptions::new().append(true).create(true).open(&log_file) {
        Ok(file) => file,
        Err(e) => {
            panic!("Failed to open or create log file: {}", e);
        }
    };

    file.write_all(missed_items.as_bytes()).unwrap();
    println!("Items/Knifes missed : {count}");
    Ok(return_case_elements)
}

async fn case_parser(index: usize, mut collection: CaseElement, knife: bool) -> Result<CaseElement, reqwest::Error> {
    let res = reqwest::get(collection.url.take().unwrap()).await?;
    let body = res.text().await?;
    let scraper_doc = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("body > div.container.main-content > div:nth-child(7) > div").unwrap();
    let html_products = scraper_doc.select(&selector);
    let url = scraper_doc.select(&scraper::Selector::parse("body > div.container.main-content > div:nth-child(7) > div:nth-child(1) > div > h3 > a").unwrap())
        .next()
        .and_then(|x| x.value().attr("href"))
        .map(str::to_owned);

    if !knife {
        collection.knifes = Some(Box::new(CaseElement::new(url, Some(String::new()), Some(String::new()), Some(String::new()))));
    }

    for element in html_products {
        let aselector = &scraper::Selector::parse("div > h3 > a").unwrap();
        let mut name: Option<String> = Some("".to_string());
        let name_header = element.select(aselector);
        for i in name_header {
            name = Some(name.unwrap() + &i.text().collect::<String>() + " ");
        }
        let rarity: Option<structs::Rarity> = Some(
            structs::Rarity::convert(element.select(&scraper::Selector::parse("div > a:nth-child(2) > div > p").unwrap())
                .next()
                .map(|x| x.text().collect::<String>())));
        let nonstatprice: Option<String> = element.select(&scraper::Selector::parse("div > div:nth-child(5) > p > a").unwrap())
            .next()
            .map(|x| x.text().collect::<String>());
        let statprice: Option<String> = element.select(&scraper::Selector::parse("div > div:nth-child(6) > p > a").unwrap())
            .next()
            .map(|x| x.text().collect::<String>());
        let image: Option<String> = element.select(&scraper::Selector::parse("div > a:nth-child(4) > img").unwrap())
            .next()
            .and_then(|x| x.value().attr("src"))
            .map(str::to_owned);
        collection.items.as_mut().unwrap().push(Items::new(name, rarity, nonstatprice, statprice, image));
    }
    println!("Thread {:#?} has finished", index);
    println!("Thread {index} took : {:#?} secs", (Local::now() - EXEC_TIME.read().unwrap()[index]).num_seconds());
    Ok(collection)
}