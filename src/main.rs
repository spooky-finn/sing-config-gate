mod adapters;
mod config;
mod db;
mod ports;
mod service;
mod utils;

use std::sync::Arc;

use adapters::db::UserRepo;
use config::AppConfig;
use service::handle_msg::HandleMsgService;
use teloxide::{macros::BotCommands, prelude::*, types::Message};
use tracing::{error, info};
use utils::logger;

use crate::adapters::db::VlessIdentityRepo;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(BotCommands, Debug, Clone)]
#[command()]
enum BotCommands {
    Start,
}

#[tokio::main]
async fn main() {
    let config = AppConfig::load();
    logger::init(&config.log_level, config.log_disable_timestamp);
    info!("Starting bot");

    std::panic::set_hook(Box::new(|panic_info| {
        error!(%panic_info, "Unhandled panic");
    }));

    let pool = db::connect(&config.db_location).expect("Failed to initialize database");
    let user_repo = Arc::new(UserRepo::new(pool.clone()));
    let vless_identity_repo = Arc::new(VlessIdentityRepo::new(pool.clone()));
    let handle_msg_service = Arc::new(HandleMsgService::new(
        user_repo,
        vless_identity_repo,
        config.clone(),
    ));
    let bot = Bot::new(config.tg_bot_token);

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<BotCommands>()
                .endpoint(handle_command),
        )
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![handle_msg_service])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn handle_command(
    _cmd: BotCommands,
    msg: Message,
    service: Arc<HandleMsgService>,
) -> Result<(), teloxide::RequestError> {
    if let Err(e) = service.handle_msg(&msg).await {
        error!(error = %e, "Error handling /start command");
    }
    Ok(())
}

async fn handle_message(
    msg: Message,
    service: Arc<HandleMsgService>,
) -> Result<(), teloxide::RequestError> {
    if let Err(e) = service.handle_msg(&msg).await {
        error!(error = %e, "Error handling message");
    }
    Ok(())
}

async fn handle_callback(
    query: CallbackQuery,
    service: Arc<HandleMsgService>,
) -> Result<(), teloxide::RequestError> {
    if let Err(e) = service.handle_callback(&query).await {
        error!(error = %e, "Error handling callback");
    }
    Ok(())
}
