mod bot;

use bot::*;
use serenity::{http::Http, model::prelude::MessageId, prelude::GatewayIntents, Client};
use std::{fs, path::Path};

fn make_corpus() {
    // let paths: Vec<Path, u16> = fs::read_dir("./chunks")
    // 	.unwrap()
    // 	.map(|p| {
    // 	    (
    // 		p.unwrap(),
    // 		p.unwrap()
    // 		    .file_name()
    // 		    .into_string()
    // 		    .unwrap()
    // 		    .parse::<u16>()
    // 		    .unwrap(),
    // 	    )
    // 	})
    // 	.collect();

    // for path in paths.sort_by() {
    // 	println!("{:?}", path);
    // }
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(_) => {
            make_corpus();
        }
        None => {
            let mut token = fs::read_to_string("token").expect("Error reading token file");
            token = token.trim().to_string();
            let handler = Handler {};

            println!("Creating client...");
            let mut client = Client::builder(&token, GatewayIntents::MESSAGE_CONTENT)
                .event_handler(handler)
                .await
                .expect("Err creating client");

            println!("Starting client...");
            if let Err(why) = client.start().await {
                println!("Client error: {:?}", why);
            }
            println!("Continuing...");
        }
    };
}
