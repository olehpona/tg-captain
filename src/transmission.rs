use teloxide::{
    prelude::*,
    types::{ MessageKind, MediaKind},
    dispatching::DpHandlerDescription,
    utils::command::BotCommands, RequestError,
};
use transmission_rpc::types;
use transmission_rpc::TransClient;
use substring::Substring;


extern crate mime;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command{
    Transmission (String)
}

pub fn get_short_help()-> String{
    return "Transsmision plugin. Usage /transmission [mode]. For detail help /transmission help".to_string();
}

pub fn get_update_handler(url: &String) -> Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription>{
    let url_clone1 = url.to_owned();
    let url_clone2 = url.to_owned();

    let file_closure = move |bot, msg| {
        add_file(bot, msg, url_clone1.clone())
    };
    let command_closure = move |bot, msg, cmd: Command| {
        command_handler(bot, msg, cmd, url_clone2.clone())
    };
    Update::filter_message()
    .branch(
        dptree::entry()
            .filter_command::<Command>()
            .endpoint(command_closure),
    )
    .branch(
        dptree::entry()
        .filter(|msg: Message| file_filter(msg))
        .endpoint(file_closure)
    )
}

fn file_filter(msg: Message) -> bool{
    if let MessageKind::Common(data) = &msg.kind {
        if let MediaKind::Document(doc) = &data.media_kind{
            let mime: mime::Mime = "application/x-bittorrent".parse().unwrap();
            if Some(mime) == doc.document.mime_type{
                return true;
            }
        }
    }
    return false;
}

async fn add_file(bot:Bot, msg: Message ,url:String) -> ResponseResult<()>{
    let mut client = TransClient::new(url.parse().unwrap());
    let add: types::TorrentAddArgs = types::TorrentAddArgs {
        filename: Some(
            get_download_link(&bot, &msg).await
        ),
        ..types::TorrentAddArgs::default()
    };

    match client.torrent_add(add).await{
        Ok(res) =>{ if res.is_ok(){
            bot.send_message(msg.chat.id, "Added sucessfully").await?;
        } else {
            bot.send_message(msg.chat.id, "Failed").await?;
        }},
        Err(x) => {bot.send_message(msg.chat.id, format!("Failed with err: {}", x)).await?;}

    }

    Ok(())
}

async fn get_download_link(bot: &Bot,msg: &Message) -> String{
    let mut link = String::new();
    if let MessageKind::Common(data) = &msg.kind {
        if let MediaKind::Document(doc) = &data.media_kind{
            let mime: mime::Mime = "application/x-bittorrent".parse().unwrap();
            if Some(mime) == doc.document.mime_type{
                let data = bot.get_file(&doc.document.file.id).await.unwrap();
                link = format!("https://api.telegram.org/file/bot{}/{}", bot.token(), data.path);
            }
        }
    }
    return link;
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    url: String
) -> ResponseResult<()>{
        let mut client = TransClient::new(url.parse().unwrap());
        match cmd {
            Command::Transmission(command) => {
                let com: Vec<&str> = command.trim().split(" ").collect();
                if com[0] == "" || com[0] == "list"{
                    bot.send_message(msg.chat.id, list_torrent(&mut client).await).await?;
                } else if com[0] == "stop"{
                    let is_value = com.len()>=2;
                    if is_value{
                        bot.send_message(msg.chat.id, pause_torrent(&mut client, com[1].parse().unwrap()).await).await?;
                    } else {
                        bot.send_message(msg.chat.id, "Please provide torrent id").await?;
                    }
                } else if com[0] == "start"{
                    let is_value = com.len()>=2;
                    if is_value{
                        bot.send_message(msg.chat.id, start_torrent(&mut client, com[1].parse().unwrap()).await).await?;
                    } else {
                        bot.send_message(msg.chat.id, "Please provide torrent id").await?;
                    }
                } else if com[0] == "remove"{
                    let is_value = com.len()>=3;
                    if is_value{
                        let with_data = match com[2].to_lowercase().as_str(){
                            "yes" => true,
                            "no" => false,
                            "n" => false,
                            "y" => true,
                            _ => false
                        };
                        bot.send_message(msg.chat.id, remove_torrent(&mut client, com[1].parse().unwrap(), with_data).await).await?;
                    } else {
                        bot.send_message(msg.chat.id, "Please provide 2 parameters (torrent id, with_data (yes,no) )").await?;
                    }
                } else if com[0] == "help"{
                    bot.send_message(msg.chat.id, get_command_handler_help_text()).await?;
                }
            }
        }
        Ok(())
    }

async fn list_torrent(client: &mut TransClient) -> String{
    let mut message = String::new();

    match client.torrent_get(None, None).await{
        Ok(res) => {
            for torrent in res.arguments.torrents{
                message += format!("{:?}: {:?} {:?} {:.0}%\n",
                    torrent.id.unwrap(),
                    torrent.name.unwrap().substring(0,10).to_owned() + "...",
                    torrent.status.unwrap(),
                    ((torrent.total_size.unwrap() as f64 - torrent.left_until_done.unwrap() as f64)/torrent.total_size.unwrap() as f64)*100.0
                ).as_str();
            }
        },
        Err(x) => println!("{}", x)
    } 


    return message
}

async fn pause_torrent(client: &mut TransClient, id: i64) -> String{
    match client
    .torrent_action(types::TorrentAction::Stop, vec![types::Id::Id(id)])
    .await{
        Ok(res) => {
            if res.is_ok(){
                return "Paused successfully".to_string();
            } else {
                return "Pause failed".to_string();
            }
        },
        Err(x) => format!("Pause failed with err: {}", x)
    }
}

async fn start_torrent(client: &mut TransClient, id: i64) -> String{
    match client
    .torrent_action(types::TorrentAction::Start, vec![types::Id::Id(id)])
    .await{
        Ok(res) => {
            if res.is_ok(){
                return "Started successfully".to_string();
            } else {
                return "Start failed".to_string();
            }
        },
        Err(x) => format!("Start with err: {}", x)
    }
}

async fn remove_torrent(client: &mut TransClient, id: i64, with_data: bool) -> String{
    return match client.torrent_remove(vec![types::Id::Id(id)], with_data).await{
        Ok(res) => {
            if res.is_ok(){
                if with_data{
                    return "Removed with data successfully".to_string()
                } else {
                    return "Removed without data successfully".to_string()
                }
            }else {
                return "Removed failed".to_string()
            }
    },
        Err(x) => format!("Removed with err: {}", x)
    }
}

fn get_command_handler_help_text() -> String {
    let help_text = r#"
Transmission Command Usage:

/transmission [subcommand] [arguments]

Available Subcommands:

list or "" (empty)
  Lists all torrents in the client.

stop [torrent_id]
  Stops (pauses) the torrent with the specified ID.
  Example: /transmission stop 1234

start [torrent_id]
  Starts (resumes) the torrent with the specified ID.
  Example: /transmission start 5678

remove [torrent_id] [with_data]
  Removes the torrent with the specified ID from the client.
  The 'with_data' argument can be 'yes' or 'no' (or 'y' or 'n') to specify whether to delete the downloaded data or not.
  Example: /transmission remove 9012 no

If no subcommand is provided or an invalid subcommand is given, the command will list all torrents by default.

Just drop .torrent file to start downloading it.

Note: Replace [torrent_id] with the actual ID of the torrent you want to operate on.
"#.to_string();

    help_text
}
