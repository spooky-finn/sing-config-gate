use std::{error::Error, sync::Arc};

use teloxide::{
    prelude::*,
    types::{ChatId, Message, User as TgUser},
    Bot,
};
use tracing::{error, info};

use crate::{
    config::AppConfig,
    db::{enums::UserStatus, models::User},
    errors::RepoError,
    ports::{user::UserRepoTrait, vless_identity::VlessIdentityRepoTrait},
    service::admin::InvitationCmd,
    utils::telegram,
};

pub struct HandleMsgService {
    bot: Bot,
    user_repo: Arc<dyn UserRepoTrait>,
    vless_identity_repo: Arc<dyn VlessIdentityRepoTrait>,
    config: AppConfig,
}

impl HandleMsgService {
    pub fn new(
        bot: Bot,
        user_repo: Arc<dyn UserRepoTrait>,
        vless_identity_repo: Arc<dyn VlessIdentityRepoTrait>,
        config: AppConfig,
    ) -> Self {
        Self {
            bot,
            vless_identity_repo,
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
        info!(user_id, "Handling callback");

        if let Some(cmd) = self.is_admin_invation_command(query) {
            self.handle_admin_invation_command(&cmd, query.message.as_ref())
                .await?;
        }

        // Acknowledge the callback query
        self.bot.answer_callback_query(query.id.clone()).await?;
        Ok(())
    }

    pub async fn handle_msg(&self, msg: &Message) -> Result<(), Box<dyn Error + Send + Sync>> {
        let from = msg.from.as_ref();
        if from.is_none() || from.unwrap().is_bot {
            return Ok(());
        }

        let from = from.unwrap();
        let user_id = from.id.0 as i64;
        info!(user_id, "Handling message");

        let existing_user = self.user_repo.get(user_id)?;
        match existing_user {
            Some(user) => {
                self.send_status_message(&user).await?;
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
        self.send_invation_request_to_admin(user).await?;
        self.send_status_message(&new_user).await?;

        Ok(())
    }

    async fn send_status_message(&self, user: &User) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chat_id = ChatId(user.id);
        let status = user.status_enum();

        match status {
            UserStatus::New => {
                self.bot
                    .send_message(chat_id, "Администратор скоро рассмотрит вашу заявку")
                    .await?;
            }
            UserStatus::Accepted => {
                let config_url = self.get_config_link(user)?;
                self.bot
                    .send_message(
                        chat_id,
                        format!("Вам одобрен доступ. Ссылка для подключения: {}", config_url),
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

    fn get_config_link(&self, user: &User) -> Result<String, RepoError> {
        let base = self.config.client_config_endpoint.trim_end_matches('/');
        let vless_identity = self.vless_identity_repo.get_by_user_id(user.id)?;
        Ok(format!("{}/{}", base, vless_identity.uuid))
    }

    async fn send_invation_request_to_admin(
        &self,
        user: &TgUser,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = format!(
            "Новая заявка от {}",
            user.username
                .clone()
                .unwrap_or_else(|| user.id.0.to_string()),
        );

        let accept_cmd = InvitationCmd::new(user.id.0 as i64, UserStatus::Accepted);
        let reject_cmd = InvitationCmd::new(user.id.0 as i64, UserStatus::Rejected);

        let keyboard = telegram::inline_keyboard_row(vec![
            ("Accept", &accept_cmd.to_callback_data()?),
            ("Reject", &reject_cmd.to_callback_data()?),
        ]);

        self.bot
            .send_message(ChatId(self.config.tg_admin_id), msg)
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }

    fn is_admin_invation_command(&self, msg: &CallbackQuery) -> Option<InvitationCmd> {
        if msg.from.id.0 as i64 != self.config.tg_admin_id {
            return None;
        }

        InvitationCmd::parse(msg.data.as_ref()?)
            .inspect_err(|e| error!(error = %e, "Failed to parse admin command"))
            .ok()
    }

    async fn handle_admin_invation_command(
        &self,
        cmd: &InvitationCmd,
        message: Option<&teloxide::types::MaybeInaccessibleMessage>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.user_repo.set_status(cmd.user_id, cmd.status)?;
        // Assign VLESS identity only when admin accepts
        if cmd.status == UserStatus::Accepted {
            self.vless_identity_repo.assign(cmd.user_id)?;
        }

        let user = self.user_repo.get(cmd.user_id)?;
        match user {
            Some(user) => {
                self.send_status_message(&user).await?;
            }
            None => {
                error!("user not found {}", cmd.user_id)
            }
        }

        info!(user_id = cmd.user_id, status = ?cmd.status, "Admin callback handled");
        // Remove the inline keyboard from the admin message
        if let Some(msg) = message {
            self.bot
                .edit_message_reply_markup(ChatId(self.config.tg_admin_id), msg.id())
                .await
                .ok();
        }
        Ok(())
    }
}
