mod adapters;
mod db;
mod ports;
mod service;
mod utils;

use adapters::db::{init_db, DieselUserRepo};
use service::{admin::AdminService, handle_msg::HandleMsgService};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::Message;
use tracing::{error, info};
use utils::env::AppConfig;
use utils::log::init_logger;

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env().expect("Failed to load environment configuration");

    init_logger(&config.log_level, config.log_disable_timestamp);

    info!("Starting bot");

    std::panic::set_hook(Box::new(|panic_info| {
        error!(%panic_info, "Unhandled panic");
    }));

    let pool = init_db(&config.db_location).expect("Failed to initialize database");
    let user_repo = Arc::new(DieselUserRepo::new(pool));

    let admin_service = AdminService::new(user_repo.clone(), config.tg_admin_id);
    let handle_msg_service = Arc::new(HandleMsgService::new(
        user_repo,
        admin_service,
        config.client_config_endpoint,
    ));

    let bot = Bot::new(config.tg_bot_token);

    info!("Bot started successfully");

    let handler = Update::filter_message()
        .branch(dptree::endpoint({
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
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
            }
        }))
        .branch(
            Update::filter_callback_query().branch(dptree::endpoint({
                let service = handle_msg_service.clone();
                move |query: teloxide::types::CallbackQuery| {
                    let service = service.clone();
                    async move {
                        if query.from.is_bot {
                            return Ok(());
                        }
                        if let Err(e) = service.handle_callback(&query).await {
                            error!(error = %e, "Error handling callback");
                        }
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    }
                }
            })),
        );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
