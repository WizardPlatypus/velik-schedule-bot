use crate::data::{Day, Group, Repeat, Slot};
use chrono::Datelike;
use futures::future::BoxFuture;
use teloxide::{
    prelude::*,
    types::ParseMode,
    utils::command::{BotCommands, ParseError},
};

use lazy_static::lazy_static;

use crate::db::Database;
use async_once::AsyncOnce;
lazy_static! {
    static ref DB: AsyncOnce<Database> = AsyncOnce::new(async {
        let pool = sqlx::SqlitePool::connect("sqlite:///tmp/test.db")
            .await
            .unwrap();

        Database::new(pool)
    });
}

pub async fn run(token: String) {
    let bot = Bot::new(token);

    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("Failed to set bot commands");

    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(command_handler),
    );

    let error_handler = std::sync::Arc::new(ErrorHandler { bot: bot.clone() });

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .error_handler(error_handler)
        .build()
        .dispatch()
        .await;
}

#[derive(BotCommands, Clone)]
#[command(description = "Commands:", rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "<group>")]
    Config(String),
    #[command(description = "<date> <slot> <group>", parse_with = parse_command_subject)]
    Subject {
        slot: Option<String>,
        date: Option<String>,
    },
}

fn parse_command_subject(s: String) -> Result<(Option<String>, Option<String>), ParseError> {
    let args: Vec<&str> = s.split(" ").collect();
    match &args[..] {
        [] => Ok((None, None)),
        [slot] => Ok((Some(slot.to_string()), None)),
        ["_", date] => Ok((None, Some(date.to_string()))),
        [slot, date] => Ok((Some(slot.to_string()), Some(date.to_string()))),
        _ => Err(ParseError::Custom("Only 2 arguments allowed!".into())),
    }
}

pub struct ErrorHandler {
    bot: Bot,
}

impl teloxide::error_handlers::ErrorHandler<Error> for ErrorHandler {
    fn handle_error(self: std::sync::Arc<Self>, error: Error) -> BoxFuture<'static, ()> {
        use Error::*;

        let (chat_id, message) = match error {
            Some(x) => (x, "Something went wrong".into()),
            NoGroupConfigured(x) => (
                x,
                "Please configure your group with `/config <group>`".into(),
            ),
            InvalidGroup(x, value) => (x, format!("Invalid group: {}.", &value)),
            InvalidDate(x, value) => (x, format!("Invalid date: {}.", &value)),
            InvalidSlot(x, value) => (x, format!("Invalid slot: {}.", &value)),
            InvalidWeekday(x, value) => (x, format!("Invalid weekday: {}.", &value)),
        };

        let fut = async move {
            let _ = self.bot.send_message(chat_id, message).await;
        };

        Box::pin(fut)
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidGroup(ChatId, String),
    InvalidDate(ChatId, String),
    InvalidSlot(ChatId, String),
    InvalidWeekday(ChatId, String),
    NoGroupConfigured(ChatId),
    Some(ChatId),
}

async fn command_handler(msg: Message, bot: Bot, cmd: Command) -> Result<(), Error> {
    use Command::*;
    match cmd {
        Config(gang) => {
            log::trace!("/config {}", &gang);
            let group = Group::try_from(gang.as_str())
                .map_err(|_| Error::InvalidGroup(msg.chat.id, gang))?;

            match DB.get().await.get_group(&msg.chat.id).await {
                Ok(_) => DB
                    .get()
                    .await
                    .update_user(&msg.chat.id, &group)
                    .await
                    .map_err(|err| {
                        log::error!("Failed to update user group: {:?}", err);
                        Error::Some(msg.chat.id)
                    })?,
                Err(sqlx::Error::RowNotFound) => DB
                    .get()
                    .await
                    .add_user(&msg.chat.id, &group)
                    .await
                    .map_err(|err| {
                        log::error!("Failed to add new user: {:?}", err);
                        Error::Some(msg.chat.id)
                    })?,
                Err(err) => {
                    log::error!("Failed to add new user: {:?}", err);
                    return Err(Error::Some(msg.chat.id));
                }
            }
        }
        Subject { slot, date } => {
            log::debug!("/subject {:?} {:?}", &slot, &date);

            let slot = if let Some(value) = slot {
                Slot::try_from(value.as_str())
                    .map_err(|_| Error::InvalidSlot(msg.chat.id, value))?
            } else {
                Slot::from(&msg.date)
            };

            let dt = if let Some(value) = date {
                let format = "%d.%m.%Y";
                chrono::NaiveDate::parse_from_str(value.as_str(), format)
                    .map_err(|err| {
                        log::trace!("Failed to parse {} as '{}': {:?}", &value, format, &err);
                        Error::InvalidDate(msg.chat.id, value)
                    })?
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
                    .and_utc()
            } else {
                msg.date.clone()
            };

            let day = Day::try_from(&dt).map_err(|err| {
                log::trace!(
                    "Failed to convert weekday '{}' to `Day`: {:?}",
                    dt.weekday(),
                    &err
                );
                Error::InvalidWeekday(msg.chat.id, format!("{}", dt.weekday()))
            })?;

            let repeat = Repeat::from(&dt);

            let group = match DB.get().await.get_group(&msg.chat.id).await {
                Ok(ok) => ok,
                Err(sqlx::Error::RowNotFound) => return Err(Error::NoGroupConfigured(msg.chat.id)),
                Err(err) => {
                    log::error!("Failed to get user group: {:?}", &err);
                    return Err(Error::Some(msg.chat.id));
                }
            };

            let subjects = DB
                .get()
                .await
                .get_subjects(day, repeat, slot, group)
                .await
                .map_err(|err| {
                    log::error!("Failed to get subjects: {:?}", &err);
                    Error::Some(msg.chat.id)
                })?;

            if subjects.is_empty() {
                let _ = bot
                    .send_message(msg.chat.id, "No such subject is found.")
                    .await;
            } else {
                let mut message = String::new();
                for s in subjects {
                    let d = crate::display::Subject::new(slot, s.title, vec![]);
                    message.push_str(format!("{}\n", &d).as_str());
                }

                let _ = bot
                    .send_message(msg.chat.id, message)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await;
            }
        }
    }
    Ok(())
}
