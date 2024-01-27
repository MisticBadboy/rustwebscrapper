use std::{vec, error::Error, thread::JoinHandle, ops::{Deref, Add, Index}, future::{Future, self}, os::windows::thread, borrow::Borrow, sync::{Arc, RwLock, Mutex}, rc::Rc, clone};

use reqwest;
use scraper::{element_ref::Select, ElementRef};
use structs::{CaseElement, Items};
use tokio::{self};
mod structs;

#[tokio::main]
async fn main() {
    if let Err(_) = run().await {
        // Handle the error here
        println!("An error occurred.");
    }
}

async fn run() -> Result<(), reqwest::Error> {
    let res = reqwest::get("https://csgostash.com/containers/skin-cases").await?;
    let body = res.text().await?;
    let scraper_doc = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("body > div.container.main-content > div:nth-child(7) > div").unwrap();
    let html_products = scraper_doc.select(&selector);
    let mut CaseElements : Vec<structs::CaseElement> = Vec::new();
    for (index, element) in html_products.enumerate()
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

        CaseElements.push(CaseElement::new(url, image, name, price));
    }

    start_case_parser(&mut CaseElements).await?;

    let mut CaseKnifeElements : Vec<CaseElement> = Vec::new();

    // for case in CaseElements
    // {
    //     CaseKnifeElements.push(case.knifes)
    // }

    Ok(())
}

async fn start_case_parser<'a>(CaseElements: &'a mut Vec<CaseElement>) -> Result<(), reqwest::Error> {
    let mut threads = Vec::new();

    let arc_elements: Arc<futures::lock::Mutex<Vec<CaseElement>>> = Arc::new(futures::lock::Mutex::new(CaseElements.clone()));

    // for (index, element) in CaseElements.iter().enumerate()
    // {
    //     let arc_elements_clone = Arc::clone(&arc_elements.);
    //     let spawned = tokio::spawn(async move {
    //         CaseParser(index, arc_elements, false).await.unwrap()
    //     }).await;
    //     threads.push(spawned);
    // }

    threads = CaseElements.into_iter().enumerate()
    .map(|f| {
        let index = f.0;
        let mut arc_elements_clone = Arc::clone(&arc_elements);
        tokio::spawn(async move {
            CaseParser(index,arc_elements_clone, false).await
        });
    })
    .collect();

    Ok(())
}

async fn CaseParser<'b>(index : usize, collection : Arc<futures::lock::Mutex<Vec<CaseElement>>>, knife : bool) ->Result<(), reqwest::Error>
{
    let res = reqwest::get(collection.lock().await[index].url.clone().unwrap().as_str()).await?;
    let body = res.text().await?;
    let scraper_doc = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("body > div.container.main-content > div:nth-child(7) > div").unwrap();
    let html_products = scraper_doc.select(&selector);
    let url = scraper_doc.select(&scraper::Selector::parse("body > div.container.main-content > div:nth-child(7) > div:nth-child(1) > div > h3 > a").unwrap())
    .next()
    .and_then(|x| x.value().attr("href"))
    .map(str::to_owned);
    if index == 0 && !knife
    {
        collection.lock().await[index].knifes = Some(Box::new(CaseElement::new(url, Some(String::new()), Some(String::new()),Some(String::new()))));
    }
    for (index, element) in html_products.enumerate()
    {
        let aselector = &scraper::Selector::parse("div > h3 > a").unwrap();
        let mut name : Option<String> = Some("".to_string());
        let mut name_header = element
        .select(aselector);
        for (index, i) in name_header .enumerate()
        {
            name = Some(name.unwrap() + &i.text().collect::<String>() + " ");
        }
        let rarity : Option<structs::Rarity> = Some(
            structs::Rarity::convert(element.select(&scraper::Selector::parse("div > a:nth-child(2) > div > p").unwrap())
            .next()
            .map(|x| x.text().collect::<String>())));
        let nonstatprice : Option<String> = element.select(&scraper::Selector::parse("div > div:nth-child(5) > p > a").unwrap())
        .next()
        .map(|x| x.text().collect::<String>());
        let statprice : Option<String> = element.select(&scraper::Selector::parse("div > div:nth-child(6) > p > a").unwrap())
        .next()
        .map(|x| x.text().collect::<String>()); 
        let image : Option<String> = element.select(&scraper::Selector::parse("div > a:nth-child(4) > img").unwrap())
        .next()
        .and_then(|x| x.value().attr("src"))
        .map(str::to_owned);
        let mut knifes : Option<CaseElement> = None;
        collection.lock().await[index].items.as_mut().unwrap().push(Items::new(name, rarity, nonstatprice, statprice, image));
    }
    println!("Thread {:#?} has finished", index);
    Ok(())
}