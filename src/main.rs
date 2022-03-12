mod request;

use request::Request;

#[tokio::main]
async fn main() -> Result<(),reqwest::Error>{
    let mut resp = Request::new(String::from("https://csgostash.com/containers/skin-cases"));
    let _page_content: String = resp.get_PageContent().await?;
    let scraped = resp.scrape_html();
    // println!("text : {:#?}", scraped);
    Ok(())
}