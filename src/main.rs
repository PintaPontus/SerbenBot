use std::collections::HashMap;
use std::process::{Command, Output};

use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    log::info!("Starting command bot...");
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
    #[command(description = "Accende il Serben.")]
    Accendi,
    #[command(description = "Spegne il Serben.")]
    Spegni,
    #[command(description = "Mostra i log del Serben.")]
    Logs(i32),
    #[command(description = "Spegne Erobren.")]
    Shutdown(i32),
}

async fn answer(bot: Bot, msg: Message, cmd: SerbenCommand) -> ResponseResult<()> {
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
        SerbenCommand::Accendi => match serben_start() {
            Ok(output) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Accensione: {}", String::from_utf8(output.stdout).unwrap()),
                )
                .await?;
            }
            Err(error) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Impossibile avviare il Serben:\n{error}"),
                )
                .await?;
            }
        },
        SerbenCommand::Spegni => match serben_stop() {
            Ok(output) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Spegnimento: {}", String::from_utf8(output.stdout).unwrap()),
                )
                .await?;
            }
            Err(error) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Impossibile fermare il Serben:\n{error}"),
                )
                .await?;
            }
        },
        SerbenCommand::Logs(lines) => {
            if lines < 1 {
                bot.send_message(
                    msg.chat.id,
                    "Inserire un numero valido di linee".to_string(),
                )
                .await?;
            } else if lines > 30 {
                bot.send_message(
                    msg.chat.id,
                    "Impossibile inviare più di 30 linee".to_string(),
                )
                .await?;
            } else {
                match serben_logs(lines) {
                    Ok(logs) => {
                        let logs_lines = String::from_utf8(logs.stdout).unwrap();
                        bot.send_message(
                            msg.chat.id,
                            format!("Ultime {} linee di logs:\n```{}```", lines, logs_lines),
                        )
                        .await?;
                    }
                    Err(error) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("Impossibile recuperare i log:\n{error}"),
                        )
                        .await?;
                    }
                }
            }
        }
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
                    bot.send_message(
                        msg.chat.id,
                        format!("Impossibile spegnere il server:\n{error}"),
                    )
                    .await?;
                }
            }
        }
    };
    Ok(())
}

fn serben_start() -> std::io::Result<Output> {
    Command::new("docker")
        .args(["start", "project-ozone-3"])
        .output()
}

fn serben_stop() -> std::io::Result<Output> {
    Command::new("docker")
        .args(["stop", "project-ozone-3"])
        .output()
}

fn serben_logs(lines: i32) -> std::io::Result<Output> {
    Command::new("docker")
        .args(["logs", "-n", &lines.to_string(), "project-ozone-3"])
        .output()
}

#[cfg(target_os = "windows")]
fn erobren_shutdown(minutes: i32) -> std::io::Result<Output> {
    Command::new("shutdown")
        .args(if minutes > 0 {
            [format!("-s -t {} -f", minutes * 60)]
        } else {
            ["-s -p -f".to_string()]
        })
        .output()
}

#[cfg(not(target_os = "windows"))]
fn erobren_shutdown(minutes: i32) -> std::io::Result<Output> {
    Command::new("shutdown")
        .args(if minutes > 0 {
            [format!("-P +{minutes}")]
        } else {
            ["-h now".to_string()]
        })
        .output()
}
