use deploy_scripts::{add_asset::add_asset, deploy::deployment::deploy};

#[tokio::main]
pub async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Please specify 'deploy' or 'add-assets'");
        return;
    }

    match args[1].as_str() {
        "deploy" => deploy().await,
        "add-asset" => add_asset().await,
        _ => println!("Invalid argument. Use 'deploy' or 'add-asset'"),
    }
}
