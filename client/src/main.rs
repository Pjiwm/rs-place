use reqwest::blocking as reqwest;
use serde::Serialize;
use serde_json::Value;
#[derive(Serialize)]
struct Body {
    x: u32,
    y: u32,
    rgba: u32,
}
fn main() {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:8080/send")
        .json(&Body {
            x: 0,
            y: 0,
            rgba: 100,
        })
        .send();
    println!("{:?}", res);
    // reqwest::RequestBuilder::body(self, body)
    match reqwest::get("http://localhost:8080/state") {
        Ok(result) => {
            println!("{}", result.status());
            println!(
                "{:?}",
                result.json::<Value>().unwrap().as_array().unwrap().get(0)
            );
        }
        Err(e) => println!("{e}"),
    }
}
