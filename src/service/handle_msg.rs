use std::{error::Error, sync::Arc};

use teloxide::{
    prelude::*,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, User as TgUser},
    Bot,
};
use tracing::{error, info};

use crate::{
    config::AppConfig,
    db::{enums::UserStatus, models::User},
    ports::user::IUserRepo,
    service::admin::InvitationCmd,
};

pub struct HandleMsgService {
    user_repo: Arc<dyn IUserRepo>,
    config: AppConfig,
    bot: Bot,
}

impl HandleMsgService {
    pub fn new(user_repo: Arc<dyn IUserRepo>, config: AppConfig) -> Self {
        Self {
            bot: Bot::from_env(),
            user_repo,
            config,
        }
    }

    pub async fn handle_callback(
        &self,
        query: &CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if query.from.is_bot {
            return Ok(());
        }

        let user_id = query.from.id.0 as i64;
        info!(user_id = user_id, "Handling callback");

        if let Some(cmd) = self.is_admin_invation_command(query) {
            self.handle_invation_command(&cmd).await?;
        }

        // Acknowledge the callback query to remove the loading icon from the client
        teloxide::Bot::from_env()
            .answer_callback_query(query.id.clone())
            .await?;
        Ok(())
    }

    pub async fn handle_msg(&self, msg: &Message) -> Result<(), Box<dyn Error + Send + Sync>> {
        let from = msg.from.as_ref();
        if from.is_none() || from.unwrap().is_bot {
            return Ok(());
        }

        let from = from.unwrap();
        let user_id = from.id.0 as i64;
        info!(user_id = user_id, "Handling message");

        let existing_user = self.user_repo.select(user_id)?;

        match existing_user {
            Some(user) => {
                self.send_status(user_id, &user).await?;
            }
            None => {
                self.register(from).await?;
            }
        }

        Ok(())
    }

    async fn register(&self, user: &TgUser) -> Result<(), Box<dyn Error + Send + Sync>> {
        let new_user = User {
            id: user.id.0 as i64,
            username: user.username.clone().unwrap_or_default(),
            status: UserStatus::New as i32,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.user_repo.insert(&new_user)?;
        self.send_message_to_admin(user, "Новая заявка").await?;

        Ok(())
    }

    async fn send_status(
        &self,
        user_id: i64,
        user: &User,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chat_id = ChatId(user_id);

        let status = user.status_enum();

        match status {
            UserStatus::New => {
                self.bot
                    .send_message(chat_id, "Администратор скоро рассмотрит вашу заявку")
                    .await?;
            }
            UserStatus::Accepted => {
                let config_url = self.get_config_link(user);
                self.bot
                    .send_message(
                        chat_id,
                        format!(
                            "Вам одобрен доступ. Ваш файл конфигурации доступен по ссылке: {}",
                            config_url
                        ),
                    )
                    .await?;
            }
            UserStatus::Rejected => {
                self.bot
                    .send_message(chat_id, "Ваша заявка отклонена")
                    .await?;
            }
        }

        Ok(())
    }

    fn get_config_link(&self, user: &User) -> String {
        let base = self.config.client_config_endpoint.trim_end_matches('/');
        format!("{}/{}", base, user.id)
    }

    async fn send_message_to_admin(
        &self,
        user: &TgUser,
        message: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let bot = Bot::from_env();
        let msg = format!(
            "Новая заявка от {}: {}",
            user.username
                .clone()
                .unwrap_or_else(|| user.id.0.to_string()),
            message
        );

        let accept_cmd = InvitationCmd::new(user.id.0 as i64, UserStatus::Accepted);
        let reject_cmd = InvitationCmd::new(user.id.0 as i64, UserStatus::Rejected);

        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("Accept", accept_cmd.to_callback_data()?),
            InlineKeyboardButton::callback("Reject", reject_cmd.to_callback_data()?),
        ]]);

        bot.send_message(ChatId(self.config.tg_admin_id), msg)
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }

    pub fn is_admin_invation_command(&self, msg: &CallbackQuery) -> Option<InvitationCmd> {
        if msg.from.id.0 as i64 != self.config.tg_admin_id {
            return None;
        }

        InvitationCmd::parse(msg.data.as_ref()?)
            .inspect_err(|e| error!(error = %e, "Failed to parse admin command"))
            .ok()
    }

    pub async fn handle_invation_command(
        &self,
        cmd: &InvitationCmd,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.user_repo.set_status(cmd.user_id, cmd.status)?;

        match cmd.status {
            UserStatus::Accepted => todo!(),
            UserStatus::Rejected => todo!(),
            UserStatus::New => {}
        }

        info!(user_id = cmd.user_id, status = ?cmd.status, "Admin callback handled");
        Ok(())
    }
}
