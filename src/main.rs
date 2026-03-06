use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use sing_box_config_bot::{
    adapters::{UserRepo, VlessIdentityRepo},
    config::AppConfig,
    domain::RoutingConfig,
    generate_config,
    service::handle_msg::HandleMsgService,
    utils::logger,
};
use std::sync::Arc;
use teloxide::{macros::BotCommands, prelude::*, types::Message};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(BotCommands, Debug, Clone)]
#[command()]
enum BotCommands {
    Start,
}

pub struct AppState {
    app_config: AppConfig,
    routing_config: RoutingConfig,
}

#[tokio::main]
async fn main() {
    let config = AppConfig::load();
    logger::init(&config.log_level, config.log_disable_timestamp);
    info!("Starting bot and config server");

    std::panic::set_hook(Box::new(|panic_info| {
        error!(%panic_info, "Unhandled panic");
    }));

    let pool = sing_box_config_bot::db::connect(&config.db_location)
        .expect("Failed to initialize database");
    let user_repo = Arc::new(UserRepo::new(pool.clone()));
    let vless_identity_repo = Arc::new(VlessIdentityRepo::new(pool.clone()));
    let routing_config =
        RoutingConfig::load("config/domains.json").expect("Failed to load routing config");

    let bot = Bot::new(config.tg_bot_token.clone());

    let handle_msg_service = Arc::new(HandleMsgService::new(
        bot.clone(),
        user_repo.clone(),
        vless_identity_repo.clone(),
        config.clone(),
    ));

    let app_state = Arc::new(AppState {
        app_config: config.clone(),
        routing_config,
    });

    // Build HTTP server for config delivery
    let http_app = Router::new()
        .route("/health", get(health))
        .route("/{uuid}", get(get_config))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let server_addr = format!("0.0.0.0:{}", config.sing_box_server_port);
    let listener = tokio::net::TcpListener::bind(&server_addr)
        .await
        .expect("Failed to bind address");

    info!("HTTP server listening on {}", server_addr);

    // Run both bot and HTTP server concurrently
    let bot_task = tokio::spawn(async move {
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
    });

    let server_task = tokio::spawn(async move {
        axum::serve(listener, http_app)
            .await
            .expect("HTTP server failed");
    });

    tokio::select! {
        _ = bot_task => {},
        _ = server_task => {},
    }
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

async fn health() -> &'static str {
    "OK"
}

async fn get_config(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    match generate_config(&state.app_config, &state.routing_config, &uuid) {
        Ok(config_json) => Json(config_json).into_response(),
        Err(e) => {
            error!("Failed to generate config: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
