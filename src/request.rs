use std::result::Result;
use scraper::{Html,Selector};

pub struct Request{
    pub url : String
}

impl Request{
    #[allow(non_snake_case)]
    pub async fn get_PageContent(&self) -> Result<String ,reqwest::Error>{
        let resp = reqwest::get(&self.url)
        .await?
        .text_with_charset("utf-8")
        .await?;
        Ok(resp)
    }

    pub async fn scape_html(&self) -> String{
        String::from("GG")
    }
    
}