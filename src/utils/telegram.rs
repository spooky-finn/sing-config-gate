//! Telegram UI utilities.

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Creates an inline keyboard with a single row of buttons.
pub fn inline_keyboard_row(
    buttons: Vec<(&str, &str)>,
) -> InlineKeyboardMarkup {
    let row: Vec<InlineKeyboardButton> = buttons
        .into_iter()
        .map(|(text, callback_data)| {
            InlineKeyboardButton::callback(text, callback_data.to_string())
        })
        .collect();

    InlineKeyboardMarkup::new(vec![row])
}

/// Creates an inline keyboard with multiple rows of buttons.
pub fn inline_keyboard(rows: Vec<Vec<(&str, &str)>>) -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|(text, callback_data)| {
                    InlineKeyboardButton::callback(text, callback_data.to_string())
                })
                .collect()
        })
        .collect();

    InlineKeyboardMarkup::new(keyboard)
}
