use std::collections::HashMap;

use reqwest::Response;
use scraper::{Html, Selector};

pub struct Scraping {
    pub mainpage : String,
}

impl Scraping {
    async fn scrape(_site : &String) -> Result<Response,reqwest::Error> {
        let resp = reqwest::get(_site).await?;
        Ok(resp)
    }
    
    async fn get_page(page : &String) -> Result<HashMap<&str,Vec<&str>>,reqwest::Error> {
        let resp = Scraping::scrape(&page).await?.text().await.unwrap();
        let html_d = Html::parse_document(resp.as_str());
        let mut items = HashMap::new();
        let mut price  = Vec::new();
        let mut item = Vec::new();
        let mut ext  = Vec::new();
        let frag = html_d.select(&Selector::parse(".col-lg-4.col-md-6.col-widen.text-center").unwrap()).next();
        for element in frag{
            let res = match element.select(&Selector::parse("a").unwrap()).next(){
                Some(o) => o,
                None => continue
            };
            let _case = element.text().collect::<Vec<_>>();
            let _link = match res.value().attr("href"){
                Some(x) => x,
                None => continue
            };
            ext.push(_link);
            item.push(_case[3]);
            price.push(_case[7]);
        }  
        items.insert("links", ext);
        items.insert("items", item);
        items.insert("prices", price);
        Ok(items)     
    }

    pub async fn start(&self) -> Result<bool,reqwest::Error>{
        let resp = Scraping::scrape(&self.mainpage).await?.text().await.unwrap();
        let fragment = Html::parse_document(resp.as_str());
        let mut items = HashMap::new();
        let mut price  = Vec::new();
        let mut item = Vec::new();
        let mut ext  = Vec::new();
        for element in fragment.select(&Selector::parse(".col-lg-4.col-md-6.col-widen.text-center").unwrap()){
            let res = match element.select(&Selector::parse("a").unwrap()).next(){
                Some(o) => o,
                None => continue
            };
            let _case = element.text().collect::<Vec<_>>();
            let _link = match res.value().attr("href"){
                Some(x) => x,
                None => continue
            };
            ext.push(_link);
            item.push(_case[3]);
            price.push(_case[7]);
        }  
        items.insert("links", ext);
        items.insert("items", item);
        items.insert("prices", price);
        println!("{:?}",&items); 
        Ok(true)
    }
}