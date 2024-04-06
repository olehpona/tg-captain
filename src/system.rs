use teloxide::{
    dispatching::DpHandlerDescription,
    prelude::*,
    utils::command::BotCommands, RequestError,
};
use sysinfo::{
    Components, Disks, Networks, System,
};
use std::collections::HashMap;
use httping::ping;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command{
    Sys(String)
}

pub fn get_short_help()-> String{
    return "Sys plugin. Usage /sys [mode]. For detail help /sys help".to_string();
}

pub fn get_update_handler(host_info: HashMap<String, String>) -> Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription> {
    let answer_closure = move |bot, msg, cmd| {
        answer(bot, msg, cmd, host_info.clone())
    };
    Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(answer_closure),
        )
}

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    host_info: HashMap<String, String>) -> ResponseResult<()> {
    match cmd {
        Command::Sys(mode) => {
            if mode == "system" || mode == ""{
                bot.send_message(msg.chat.id, get_system_info().await).await?;
            } else if mode =="net" || mode == "network"{
                bot.send_message(msg.chat.id, get_network_info()).await?;
            } else if mode =="disk" || mode == "mount"{
                bot.send_message(msg.chat.id, get_disk_info()).await?;
            } else if mode =="ping"{
                bot.send_message(msg.chat.id, "FETCHING...").await?;
                bot.send_message(msg.chat.id, get_host_info(host_info).await).await?;
            } else if mode == "temp"{
                bot.send_message(msg.chat.id, get_temp_info()).await?;
            } else if mode == "shutdown"{
                bot.send_message(msg.chat.id, shutdown()).await?;
            } else if mode == "reboot"{
                bot.send_message(msg.chat.id, reboot()).await?;
            } else if mode == "sleep"{
                bot.send_message(msg.chat.id, sleep()).await?;
            } else if mode == "hibernate"{
                bot.send_message(msg.chat.id, hibernate()).await?;
            } else if mode == "help"{
                bot.send_message(msg.chat.id, get_info_help_text()).await?;
            }
        }
    }
    Ok(())
}

async fn get_system_info() -> String{
    let mut sys = System::new();
    let up_time = System::uptime();

    let mut cpu_v = String::new();
    sys.refresh_cpu();
    tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.2)).await;
    sys.refresh_all();
    for cpu in sys.cpus() {
        cpu_v += format!(" {} ({:.1} GHz): {}%\n",cpu.name(),(cpu.frequency() as f32)/1000.0, cpu.cpu_usage()).as_str();
    }
    format!("CPU:\n{}\nRAM: {:.2}/{:.2} Gb\nUp Time: {:?}m", cpu_v, (sys.used_memory() as f32)/1073741824.0, (sys.total_memory() as f32)/1073741824.0, up_time/60)
}

fn get_network_info() -> String{
    let mut networks_stat = String::new();
    let networks = Networks::new_with_refreshed_list();
    for (interface_name, data) in &networks {
        networks_stat += format!(
            "{interface_name}: {:.3} MB (down) / {:.3} MB (up)\n",
            (data.total_received() as f32)*0.00000095367432,
            (data.total_transmitted() as f32)*0.00000095367432,
        ).as_str();

    }
    format!("{}", networks_stat)
}

fn get_disk_info() -> String{
    let mut mounts_data = String::new();
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        mounts_data += format!("{:?}--{:?}-->{:?} {:.2}/{:.2} Gb\n", disk.name(), disk.file_system(), disk.mount_point(), (disk.available_space() as f32)/1073741824.0, (disk.total_space() as f32)/1073741824.0).as_str();
    }

    format!("{}", mounts_data)
}   

async fn get_host_info(services : HashMap<String, String>) -> String{
    let mut data = String::new();
    for (key, value) in services.into_iter() {
        let addr: Vec<&str> = value.split(":").collect();
        
        match ping("", addr[1], addr[0],addr[2].parse().unwrap(),).await{
            Ok(_) => data += format!("{}: OK\n", key).as_str(),
            Err(_) => data += format!("{}: BAD\n", key).as_str()
        }
    }


    return data;
}

fn get_temp_info() -> String{
    let components = Components::new_with_refreshed_list();
    let mut temps = String::new();
    for component in &components {
        temps += format!("{component:?}\n").as_str();
    }
    if temps == ""{
        return "No data".to_string();
    } else {
        return temps;
    }
}

fn shutdown() -> String {
    match system_shutdown::shutdown() {
        Ok(_) => "Shutting down".to_string(),
        Err(error) => format!("Failed to shut down: {}", error),
    }
}

fn reboot() -> String {
    match system_shutdown::reboot() {
        Ok(_) => "Rebooting".to_string(),
        Err(error) => format!("Failed to reboot: {}", error),
    }
}

fn sleep() -> String {
    match system_shutdown::sleep() {
        Ok(_) => "Switching to sleep mode".to_string(),
        Err(error) => format!("Failed to sleep: {}", error),
    }
}

fn hibernate() -> String {
    match system_shutdown::hibernate() {
        Ok(_) => "Hibernating".to_string(),
        Err(error) => format!("Failed to hibernate: {}", error),
    }
}

fn get_info_help_text() -> String {
    let help_text = r#"
Info Command Usage:

/sys [mode]

Available Modes:

system or "" (empty)
  Displays system information such as CPU, memory, and operating system details.

net or network
  Shows network information, including active network interfaces and their configurations.

disk or mount
  Provides information about mounted disk partitions, file systems, and disk usage.

ping
  Fetches and displays the host information, which may include details like hostname, IP address, etc.

temp
  Retrieves and shows the current temperature readings for the system.

shutdown
  Initiates a system shutdown process.

reboot
  Reboots the system.

sleep
  Puts the system into sleep mode.

hibernate
  Hibernates the system.

If no mode is specified or an invalid mode is provided, the command will display the system information by default.

Note: The 'shutdown', 'reboot', 'sleep', and 'hibernate' modes require appropriate permissions to execute successfully.
"#.to_string();

    help_text
}