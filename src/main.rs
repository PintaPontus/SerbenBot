use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::process::{Command, Output};
use log::{debug, log};
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    log::info!("Starting command bot...");
    let bot = Bot::from_env();
    SerbenCommand::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Sono supportati questi comandi:")]
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
        SerbenCommand::Help => bot.send_message(
            msg.chat.id, SerbenCommand::descriptions().to_string()
        ).await?,
        SerbenCommand::Ip => {
            let resp = reqwest::get("https://api.ipify.org?format=json")
                .await?
                .json::<HashMap<String, String>>()
                .await?;
            bot.send_message(msg.chat.id, resp.get("ip").unwrap_or(&format!("Impossibile reperire l'ip:\n{error}"))).await?
        },
        SerbenCommand::Accendi => {
            match serben_start(){
                Ok(output) => bot.send_message(msg.chat.id, format!("Accensione: {}", String::from_utf8(output.stdout).unwrap())).await?,
                Err(error) => bot.send_message(msg.chat.id, format!("Impossibile avviare il Serben:\n{error}")).await?
            }
        },
        SerbenCommand::Spegni => {
            match serben_stop(){
                Ok(output) => bot.send_message(msg.chat.id, format!("Spegnimento: {}", String::from_utf8(output.stdout).unwrap())).await?,
                Err(error) => bot.send_message(msg.chat.id, format!("Impossibile fermare il Serben:\n{error}")).await?
            }
        },
        SerbenCommand::Logs(lines) => {
            match serben_logs(lines){
                Ok(logs) => bot.send_message(msg.chat.id, format!("Logs:\n{}", String::from_utf8(logs.stdout).unwrap())).await?,
                Err(error) => bot.send_message(msg.chat.id, format!("Impossibile recuperare i log:\n{error}")).await?
            }

        },
        SerbenCommand::Shutdown(seconds) => {
            let sec_word = if seconds > 0 {"secondi"} else {"secondo"};
            match erobren_shutdown(seconds){
                Ok(_) => bot.send_message(msg.chat.id, format!("Spegnimento in {seconds} {sec_word}🛑")).await?,
                Err(error) => bot.send_message(msg.chat.id, format!("Impossibile spegnere il server:\n{error}")).await?
            }
        },
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
fn erobren_shutdown(seconds: i32) -> std::io::Result<Output> {
    Command::new("shutdown")
        .args(["-s", "-t", &seconds.to_string(), "-f"])
        .output()
}

#[cfg(not(target_os = "windows"))]
fn erobren_shutdown(seconds: i32) -> std::io::Result<Output> {
    Command::new("shutdown")
        .args(["-P", format!("+{seconds}").as_str()])
        .output()
}
