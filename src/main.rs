mod request;

use request::Scraping;

#[tokio::main]
pub async fn main(){
    let idk = Scraping{
        mainpage : String::from("https://csgostash.com/containers/skin-cases"),
    };
    let _resp = idk.start().await;
}