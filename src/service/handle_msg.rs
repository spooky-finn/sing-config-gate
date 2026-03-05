use crate::db::{enums::UserStatus, NewUser, User};
use crate::ports::user::IUserRepo;
use crate::service::admin::{AdminService, InvitationCmd};
use rand::Rng;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, User as TgUser};
use teloxide::Bot;
use tracing::info;

pub struct HandleMsgService {
    user_repo: Arc<dyn IUserRepo>,
    admin_service: AdminService,
    client_config_endpoint: String,
}

impl HandleMsgService {
    pub fn new(
        user_repo: Arc<dyn IUserRepo>,
        admin_service: AdminService,
        client_config_endpoint: String,
    ) -> Self {
        Self {
            user_repo,
            admin_service,
            client_config_endpoint,
        }
    }

    pub async fn handle_callback(&self, query: &CallbackQuery) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if query.from.is_bot {
            return Ok(());
        }

        if let Some(cmd) = self.admin_service.is_admin_callback(query) {
            self.admin_service.handle_admin_callback(&cmd).await?;
        }

        Ok(())
    }

    pub async fn handle_msg(&self, msg: &Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    async fn register(&self, user: &TgUser) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let auth_key = self.generate_auth_key();
        let new_user = NewUser {
            id: user.id.0 as i64,
            username: user.username.clone().unwrap_or_default(),
            status: UserStatus::New as i32,
            auth_key,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.user_repo.insert(&new_user)?;
        self.send_message_to_admin(user, "Новая заявка").await?;

        Ok(())
    }

    async fn send_status(&self, user_id: i64, user: &User) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bot = Bot::from_env();
        let chat_id = ChatId(user_id);

        let status = user.status_enum();

        match status {
            UserStatus::New => {
                bot.send_message(chat_id, "Администратор скоро рассмотрит вашу заявку")
                    .await?;
            }
            UserStatus::Accepted => {
                let config_url = self.get_config_link(user);
                bot.send_message(
                    chat_id,
                    format!(
                        "Вам одобрен доступ. Ваш файл конфигурации доступен по ссылке: {}",
                        config_url
                    ),
                )
                .await?;
            }
            UserStatus::Rejected => {
                bot.send_message(chat_id, "Ваша заявка отклонена")
                    .await?;
            }
        }

        Ok(())
    }

    fn get_config_link(&self, user: &User) -> String {
        let base = self.client_config_endpoint.trim_end_matches('/');
        format!("{}/{}", base, user.id)
    }

    fn generate_auth_key(&self) -> String {
        let mut rng = rand::rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    async fn send_message_to_admin(
        &self,
        user: &TgUser,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        bot.send_message(ChatId(self.admin_service.admin_id), msg)
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }
}
