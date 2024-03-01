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
            bot.send_message(
                msg.chat.id, resp.get("ip").unwrap_or(&String::from("Impossibile reperire l'ip"))
            ).await?
        },
        SerbenCommand::Accendi => {
            match serben_start(){
                Ok(_) => bot.send_message(msg.chat.id, "Accensione").await?,
                Err(_) => bot.send_message(msg.chat.id, "Impossibile avviare il Serben").await?
            }
        },
        SerbenCommand::Spegni => {
            match serben_stop(){
                Ok(_) => bot.send_message(msg.chat.id, "Spegnimento").await?,
                Err(_) => bot.send_message(msg.chat.id, "Impossibile fermare il Serben").await?
            }
        },
        SerbenCommand::Logs(lines) => {
            match serben_logs(lines){
                Ok(logs) => bot.send_message(
                    msg.chat.id,
                    format!("Logs: {logs_text}",
                        logs_text = logs.concat()
                    )
                ).await?,
                Err(_) => bot.send_message(msg.chat.id, "Impossibile recuperare i log").await?
            }

        },
        SerbenCommand::Shutdown(seconds) => {
            let sec_word = if seconds > 0 {"secondi"} else {"secondo"};
            match erobren_shutdown(seconds){
                Ok(_) => bot.send_message(msg.chat.id, format!("Spegnimento in {seconds} {sec_word}🛑")).await?,
                Err(_) => bot.send_message(msg.chat.id, "Impossibile spegnere il server").await?
            }
        },
    };
    Ok(())
}

// TODO: interface docker
fn serben_start() -> std::io::Result<Output> {
    Command::new("docker")
        .args(["start", "project-ozone-3"])
        .output()
}

// TODO: interface docker
fn serben_stop() -> std::io::Result<Output> {
    Command::new("docker")
        .args(["stop", "project-ozone-3"])
        .output()
}

// TODO: add log reading
fn serben_logs(lines: i32) -> std::io::Result<[String; 2]> {

    // Command::new("docker")
    //     .args([""])
    //     .output()
    Ok([
        String::from("ciao"),
        lines.to_string()
    ])
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
        .output();
}
