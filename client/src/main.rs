use std::io::{self, Write};

use once_cell::sync::Lazy;
use reqwest::blocking as reqwest;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

const CLIENT: Lazy<Client> = Lazy::new(|| Client::new());
const IP: &'static str = "http://localhost:8080";

fn main() {
    println!("Enter a command (or 'exit' to quit):");
    loop {
        print!("> ");
        if let Err(e) = io::stdout().flush() {
            println!("{e}");
            std::process::exit(0);
        }

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed_input = input.trim().to_string();

        match TryInto::<Command>::try_into(trimmed_input) {
            Ok(c) => c.run(),
            Err(e) => println!("{}", e),
        }
    }
}

#[derive(Serialize, Debug)]
struct JsonBody {
    x: u32,
    y: u32,
    rgba: u32,
}

#[derive(Debug)]
enum Command {
    SendPixel(JsonBody),
    Exit,
}

impl Command {
    fn run(&self) {
        match self {
            Command::SendPixel(json_body) => send_one_pixel(json_body),
            Command::Exit => std::process::exit(0),
        }
    }
}

impl TryFrom<String> for Command {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let args: Vec<&str> = value.split_whitespace().collect();
        match args.as_slice() {
            ["pixel", rgba, x, y] => {
                let rgba = u32::from_str_radix(rgba, 16)
                    .map_err(|_| "RGBA is not a number".to_string())?;
                let x = x
                    .parse::<u32>()
                    .map_err(|_| "X is not a number".to_string())?;
                let y = y
                    .parse::<u32>()
                    .map_err(|_| "X is not a number".to_string())?;
                Ok(Command::SendPixel(JsonBody { rgba, x, y }))
            }
            ["pixel", ..] => Err(format!(
                "Invalid arguments for send, expected: send <rgba> <x> <y>"
            )),
            ["Exit"] => Ok(Command::Exit),
            _ => Err(format!("Invalid command {:?}", args)),
        }
    }
}

fn send_one_pixel(body: &JsonBody) {
    let endpoint = format!("{IP}/send");
    let res = CLIENT.post(endpoint).json(body).send();
    println!("{:?}", res);
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
