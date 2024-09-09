mod add_assets;
mod deploy;

use add_assets::add_assets;
use deploy::deployment::deploy;

#[tokio::main]
pub async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Please specify 'deploy' or 'add-assets'");
        return;
    }

    match args[1].as_str() {
        "deploy" => deploy().await,
        "add-assets" => add_assets().await,
        _ => println!("Invalid argument. Use 'deploy' or 'add-assets'"),
    }
}
