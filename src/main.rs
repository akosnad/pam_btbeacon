#[tokio::main]
async fn main() {
    pam_btbeacon::run(None).await.unwrap();
}
