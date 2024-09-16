use deploy_scripts::{
    add_asset::add_asset,
    deploy::deployment::deploy,
    pause::{pause_protocol, unpause_protocol},
    sanity_check::sanity_check,
};

#[tokio::main]
pub async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Please specify 'deploy', 'add-asset', 'pause', 'unpause', or 'sanity-check'");
        return;
    }

    match args[1].as_str() {
        "deploy" => deploy().await,
        "add-asset" => add_asset().await,
        "pause" => pause_protocol().await,
        "unpause" => unpause_protocol().await,
        "sanity-check" => sanity_check().await,
        _ => println!(
            "Invalid argument. Use 'deploy', 'add-asset', 'pause', 'unpause', or 'sanity-check'"
        ),
    }
}
