pub(crate) use scraper::{Html, Selector};

pub struct Request {
    pub s_url: String,
        f_content: String
}

impl Request {
    #[allow(non_snake_case)]
    pub async fn get_PageContent( & mut self) -> Result<String, reqwest::Error>{
        let resp = reqwest::get( & self.s_url)
            .await ?
            .text_with_charset("utf-8")
            .await ? ;
        self.f_content = resp;
        Ok(String::from("Success"))
    }
    pub fn new(s_url: String) -> Self {
        Self {
            s_url,
            f_content: String::from("")
        }
    }
    #[warn(dead_code)]
    pub fn scrape_html(&self) -> Result<Vec<String>,&str>{
        let mut vec:Vec<String> = Vec::new();
        let fragment = Html::parse_document(&self.f_content);
        let selector = Selector::parse("div.col-lg-4.col-md-6.col-widen.text-center").unwrap();
        for element in fragment.select(&selector) {
            let selector2 = Selector::parse("div.well.result-box.nomargin > a").unwrap();
            let element2 = element.select(&selector2).next();
            match element2 {
                None => continue,
                Some(_t) =>{
                    let idk:String = element2.unwrap().html();
                    vec.push(idk);  
                    println!("{:#?}",element2.unwrap().value().attr("href"))
                }
            }
        };
        Ok(vec)
    }
}