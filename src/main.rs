use std::collections::HashMap;
use teloxide::prelude::*;
use serde::Deserialize;
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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let config_path = env::var("CONFIG_PATH").expect("Should be path to config.yml");
    let path = Path::new(&config_path);

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    let mut config_data = String::new();
    match file.read_to_string(&mut config_data) {
        Err(why) => panic!("couldn't read {}: {}", path.display(), why),
        Ok(_) => print!("{} contains:\n{}", path.display(), config_data),
    }
    let config: Config = match serde_yml::from_str(config_data.as_str()){
        Ok(config) => config,
        Err(x) => panic!("{}", x)
    };

    let bot = Bot::new(config.token);
    
    let mut enabled_plugin: Vec<String> = Vec::new();
    
    let mut handler = Update::filter_message();
    for plugin in config.plugins{
        if enabled_plugin.contains(&plugin){
            println!("Plugin {} is alredy enabled", plugin);
        } else {
            if plugin == "sys"{
                if let Some(ref sys_config) = config.sys {
                    handler = handler.branch(system::get_sys_update_handler(sys_config.ping.clone()));
                } else {
                    panic!("Sys Config is not present");
                }
            } else if plugin == "transmission"{
                if let Some(ref transmission_config) = config.transmission {
                    handler = handler.branch(transmission::get_torrent_update_handler(&transmission_config.rpc));
                } else {
                    panic!("Transmission Config is not present");
                }
            } else if plugin == "docker"{
                if let Some(ref docker_config) = config.docker {
                    handler = handler.branch(docker::get_docker_update_handler(&docker_config.mode, &docker_config.path.clone().unwrap_or_default()));
                } else {
                    panic!("Transmission Config is not present");
                }
            } else {
                println!("Plugin {} not found", plugin);
            }
        }
        enabled_plugin.push(plugin);
    }

    Dispatcher::builder(bot, handler)
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