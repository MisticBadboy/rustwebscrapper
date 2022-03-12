mod request;

use request::Request;

#[tokio::main]
async fn main() -> Result<(),reqwest::Error>{
    let resp = Request{url : String::from("https://csgostash.com/containers/skin-cases")}
    .get_PageContent()
    .await?;
    println!("text : {:?}", resp);
    Ok(())
}