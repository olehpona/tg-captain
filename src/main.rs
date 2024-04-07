use std::collections::HashMap;
use teloxide::prelude::*;
use serde::Deserialize;
use teloxide::utils::command::BotCommands;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::env;

mod system;
mod transmission;
mod docker;

#[derive(Deserialize, Debug)]
struct Config {
    token: String,
    security: bool,
    admins: Option<Vec<u64>>,
    plugins: Vec<String>,
    sys: Option<Sys>,
    docker: Option<Docker>,
    transmission: Option<Transmission>,
}

#[derive(Deserialize, Debug)]
struct Docker {
    mode: String,
    path: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Transmission {
    rpc: String,
}

#[derive(Deserialize, Debug)]
struct Sys {
    ping: HashMap<String, String>,
}

#[derive(Clone)]
struct SecurityParameters {
    admins: Vec<UserId>,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command{
    Help
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let args: Vec<String> = env::args().collect();
    let path: &Path;
    if args.len() >= 2{
        path = Path::new(&args[1]);
    } else {
        panic!("Please provide config file path");
    }
    

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    let mut config_data = String::new();
    match file.read_to_string(&mut config_data) {
        Err(why) => panic!("couldn't read {}: {}", path.display(), why),
        Ok(_) => print!("Config readed sucessfully"),
    }
    let config: Config = match serde_yml::from_str(config_data.as_str()){
        Ok(config) => config,
        Err(x) => panic!("{}", x)
    };

    let bot = Bot::new(config.token);
    
    let mut plugin_handler = Update::filter_message();

    let mut enabled_plugin: Vec<String> = Vec::new();
    let mut help_text = "TG-CAPTAIN help\n\n".to_string();

    for plugin in config.plugins{
        if enabled_plugin.contains(&plugin){
            println!("Plugin {} is alredy enabled", plugin);
        } else {
            if plugin == "sys"{
                if let Some(ref sys_config) = config.sys {
                    plugin_handler = plugin_handler.branch(system::get_update_handler(sys_config.ping.clone()));
                    help_text += system::get_short_help().as_str();
                    help_text += "\n";
                } else {
                    panic!("Sys Config is not present");
                }
            } else if plugin == "transmission"{
                if let Some(ref transmission_config) = config.transmission {
                    plugin_handler = plugin_handler.branch(transmission::get_update_handler(&transmission_config.rpc));
                    help_text += transmission::get_short_help().as_str();
                    help_text += "\n";
                } else {
                    panic!("Transmission Config is not present");
                }
            } else if plugin == "docker"{
                if let Some(ref docker_config) = config.docker {
                    plugin_handler = plugin_handler.branch(docker::get_update_handler(&docker_config.mode, &docker_config.path.clone().unwrap_or_default()));
                    help_text += docker::get_short_help().as_str();
                    help_text += "\n";
                } else {
                    panic!("Transmission Config is not present");
                }
            } else {
                println!("Plugin {} not found", plugin);
            }
        }
        enabled_plugin.push(plugin);
    }

    let help_closure = move |bot: Bot, msg: Message|{
        show_help(bot, msg, help_text.clone())
    };
    plugin_handler = plugin_handler.branch(dptree::entry().filter_command::<Command>().endpoint(help_closure));

    let mut handler = Update::filter_message();

    let mut admins_data: Vec<UserId> = Vec::new();

    if config.security{
        match config.admins{
            Some(data) => admins_data = data.iter().map(|id| UserId(id.to_owned())).collect(),
            None => panic!("Security activated but no admins provided!")
        }
        handler = handler.branch(dptree::filter(|cfg: SecurityParameters, msg: Message| {
            if let Some(user) = msg.from() {
                cfg.admins.contains(&user.id)
            } else {
                false
            }
        }).branch(plugin_handler))
    }

    let security_parameters = SecurityParameters{
        admins: admins_data
    };

    Dispatcher::builder(bot, handler)
    .dependencies(dptree::deps![security_parameters])
    .default_handler(|upd| async move {
        log::warn!("Unhandled update: {:?}", upd);
    })
    .error_handler(LoggingErrorHandler::with_custom_text(
        "An error has occurred in the dispatcher",
    ))
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn show_help(bot: Bot, msg: Message, help_text: String )-> ResponseResult<()>{
    bot.send_message(msg.chat.id, help_text).await?;
    Ok(())
}