#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::trace!("Starting schedule bot");

    let config = schedule_bot::config::get();
    schedule_bot::bot::run(config.token).await;
}
