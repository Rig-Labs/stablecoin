use deploy_scripts::deploy::deployment::deploy;

#[tokio::main]
pub async fn main() {
    deploy().await;
}
