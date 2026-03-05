mod adapters;
mod config;
mod db;
mod ports;
mod service;
mod utils;

use std::{error::Error, sync::Arc};

use adapters::db::{init_db, UserRepo};
use config::AppConfig;
use service::handle_msg::HandleMsgService;
use teloxide::{prelude::*, types::Message};
use tracing::{error, info};
use utils::logger;

#[tokio::main]
async fn main() {
    let config = AppConfig::load();
    logger::init(&config.log_level, config.log_disable_timestamp);
    info!("Starting bot");

    std::panic::set_hook(Box::new(|panic_info| {
        error!(%panic_info, "Unhandled panic");
    }));

    let pool = init_db(&config.db_location).expect("Failed to initialize database");
    let user_repo = Arc::new(UserRepo::new(pool));

    let handle_msg_service = Arc::new(HandleMsgService::new(user_repo, config.clone()));

    let bot = Bot::new(config.tg_bot_token);

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint({
            let service = handle_msg_service.clone();
            move |msg: Message| {
                let service = service.clone();
                async move {
                    if msg.from.as_ref().map(|u| u.is_bot).unwrap_or(true) {
                        return Ok(());
                    }
                    if let Err(e) = service.handle_msg(&msg).await {
                        error!(error = %e, "Error handling message");
                    }
                    Ok::<(), Box<dyn Error + Send + Sync>>(())
                }
            }
        }))
        .branch(Update::filter_callback_query().endpoint({
            let service = handle_msg_service.clone();
            move |query: teloxide::types::CallbackQuery| {
                let service = service.clone();
                async move {
                    if let Err(e) = service.handle_callback(&query).await {
                        error!(error = %e, "Error handling callback");
                    }
                    Ok::<(), Box<dyn Error + Send + Sync>>(())
                }
            }
        }));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
