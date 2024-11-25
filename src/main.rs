use std::collections::HashMap;
use std::process::{Command, Output};

use teloxide::types::ParseMode;
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    println!("Starting Erobren bot...");
    let bot = Bot::from_env();
    SerbenCommand::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Sono supportati questi comandi:"
)]
enum SerbenCommand {
    #[command(description = "Mostra il messaggio di aiuto.")]
    Help,
    #[command(description = "IP di Erobren.")]
    Ip,
    #[command(description = "Lista dei Serben.")]
    Lista,
    #[command(description = "Accendi il Serben.")]
    Accendi(String),
    #[command(description = "Spegni il Serben.")]
    Spegni(String),
    #[command(description = "Mostra gli ultimi log del Serben.")]
    Logs(String),
    #[command(description = "Spegne Erobren.")]
    Shutdown(i32),
}

async fn answer(bot: Bot, msg: Message, cmd: SerbenCommand) -> ResponseResult<()> {
    println!("Received message: {:?}", msg.text().unwrap());
    match cmd {
        SerbenCommand::Help => {
            bot.send_message(msg.chat.id, SerbenCommand::descriptions().to_string())
                .await?;
        }
        SerbenCommand::Ip => {
            bot.send_message(
                msg.chat.id,
                reqwest::get("https://api.ipify.org?format=json")
                    .await?
                    .json::<HashMap<String, String>>()
                    .await?
                    .get("ip")
                    .unwrap_or(&String::from("Impossibile reperire l'ip")),
            )
            .await?;
        }
        SerbenCommand::Lista => match serben_list() {
            Ok(output) => {
                bot.parse_mode(ParseMode::MarkdownV2)
                    .send_message(
                        msg.chat.id,
                        format!(
                            "Lista containers:\n```\n{}\n```",
                            String::from_utf8(output.stdout).unwrap()
                        ),
                    )
                    .await?;
            }
            Err(error) => {
                bot.parse_mode(ParseMode::MarkdownV2)
                    .send_message(
                        msg.chat.id,
                        format!("Impossibile ottenere la lista:\n```\n{error}\n```"),
                    )
                    .await?;
            }
        },
        SerbenCommand::Accendi(serben_name) => match serben_start(serben_name) {
            Ok(output) => {
                bot.parse_mode(ParseMode::MarkdownV2)
                    .send_message(
                        msg.chat.id,
                        format!(
                            "Accensione:\n```\n{}\n```",
                            String::from_utf8(output.stdout).unwrap()
                        ),
                    )
                    .await?;
            }
            Err(error) => {
                bot.parse_mode(ParseMode::MarkdownV2)
                    .send_message(
                        msg.chat.id,
                        format!("Impossibile avviare il Serben:\n```\n{error}\n```"),
                    )
                    .await?;
            }
        },
        SerbenCommand::Spegni(serben_name) => match serben_stop(serben_name) {
            Ok(output) => {
                bot.parse_mode(ParseMode::MarkdownV2)
                    .send_message(
                        msg.chat.id,
                        format!(
                            "Spegnimento:\n```\n{}\n```",
                            String::from_utf8(output.stdout).unwrap()
                        ),
                    )
                    .await?;
            }
            Err(error) => {
                bot.parse_mode(ParseMode::MarkdownV2)
                    .send_message(
                        msg.chat.id,
                        format!("Impossibile fermare il Serben:\n```\n{error}\n```"),
                    )
                    .await?;
            }
        },
        SerbenCommand::Logs(serben_name) => match serben_logs(serben_name.clone()) {
            Ok(logs) => {
                let logs_lines = String::from_utf8(logs.stdout).unwrap();
                bot.parse_mode(ParseMode::MarkdownV2)
                    .send_message(
                        msg.chat.id,
                        format!(
                            "Ultime {} linee di logs:\n```\n{}\n```",
                            serben_name, logs_lines
                        ),
                    )
                    .await?;
            }
            Err(error) => {
                bot.parse_mode(ParseMode::MarkdownV2)
                    .send_message(
                        msg.chat.id,
                        format!("Impossibile recuperare i log:\n```\n{error}\n```"),
                    )
                    .await?;
            }
        },
        SerbenCommand::Shutdown(minutes) => {
            if minutes < 0 {
                bot.send_message(
                    msg.chat.id,
                    "Inserire un numero valido di minuti".to_string(),
                )
                .await?;
            }
            let min_word = if minutes > 0 { "minuti" } else { "minuto" };
            match erobren_shutdown(minutes) {
                Ok(_) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("Spegnimento in {minutes} {min_word}🛑"),
                    )
                    .await?;
                }
                Err(error) => {
                    bot.parse_mode(ParseMode::MarkdownV2)
                        .send_message(
                            msg.chat.id,
                            format!("Impossibile spegnere il server:\n```\n{error}\n```"),
                        )
                        .await?;
                }
            }
        }
    };
    Ok(())
}

fn serben_list() -> std::io::Result<Output> {
    Command::new("docker").args(["ps", "-a"]).output()
}

fn serben_start(serben_name: String) -> std::io::Result<Output> {
    Command::new("docker")
        .args(["start", &*serben_name])
        .output()
}

fn serben_stop(serben_name: String) -> std::io::Result<Output> {
    Command::new("docker")
        .args(["stop", &*serben_name])
        .output()
}

fn serben_logs(serben_name: String) -> std::io::Result<Output> {
    Command::new("docker")
        .args(["logs", "-n", "50", &*serben_name])
        .output()
}

#[cfg(target_os = "windows")]
fn erobren_shutdown(timer_minutes: i32) -> std::io::Result<Output> {
    Command::new("shutdown")
        .args(if timer_minutes > 0 {
            [format!("-s -t {} -f", timer_minutes * 60)]
        } else {
            ["-s -p -f".to_string()]
        })
        .output()
}

#[cfg(not(target_os = "windows"))]
fn erobren_shutdown(timer_minutes: i32) -> std::io::Result<Output> {
    Command::new("shutdown")
        .args(if timer_minutes > 0 {
            [format!("-P +{timer_minutes}")]
        } else {
            ["-h now".to_string()]
        })
        .output()
}
