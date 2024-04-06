use teloxide::{
    prelude::*,
    dispatching::DpHandlerDescription,
    utils::command::BotCommands, RequestError,
};
use bollard::{API_DEFAULT_VERSION, container::{KillContainerOptions, PruneContainersOptions, RenameContainerOptions, RestartContainerOptions, StartContainerOptions, StopContainerOptions}, image::{ListImagesOptions, PruneImagesOptions}, network::{ListNetworksOptions, PruneNetworksOptions}, volume::{ListVolumesOptions, PruneVolumesOptions}, Docker};
use std::collections::HashMap;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command{
    Docker (String)
}

pub fn get_short_help()-> String{
    return "Docker plugin. Usage /docekr [mode]. For detail help /docker help".to_string();
}

pub fn get_update_handler(mode:&String, value: &String) -> Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription>{
    let docker = get_docker(mode, value);
    let command_closure = move |bot, msg, cmd| {
        command_handler(bot, msg, cmd, docker.clone())
    };
    Update::filter_message()
    .branch(dptree::entry().filter_command::<Command>().endpoint(command_closure))
}

fn get_docker(mode: &String, value: &String) -> Docker{
    if mode == "default"{
        Docker::connect_with_local_defaults().unwrap()
    } else if mode == "unix"{
        Docker::connect_with_socket("/var/run/docker.sock", 120, API_DEFAULT_VERSION).unwrap()
    } else if mode == "http"{
        Docker::connect_with_http(value.as_str(), 120, API_DEFAULT_VERSION).unwrap()
    } else {
        Docker::connect_with_local_defaults().unwrap()
    }
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    docker: Docker) -> ResponseResult<()>{
    match cmd {
        Command::Docker(data) => {
            let com: Vec<&str> = data.trim().split(" ").collect();
            if com[0] == "info" || com[0] == ""{
                bot.send_message(msg.chat.id, get_docker_info(&docker).await).await?;
            } else if com[0] == "container"{
                container_command_handler(&bot, &msg, &com, &docker).await?;
            } else if com[0] == "image" {
                image_command_handler(&bot, &msg, &com, &docker).await?;
            } else if com[0] == "network" {
                network_command_handler(&bot, &msg, &com, &docker).await?;
            } else if com[0] == "volume"{
                volumes_command_handler(&bot, &msg, &com, &docker).await?;
            } else if com[0] == "help"{
                bot.send_message(msg.chat.id, get_docker_command_help_text()).await?;
            } else {
                bot.send_message(msg.chat.id, get_docker_command_help_text()).await?;
            } 
        }
    }
    Ok(())
}

async fn get_docker_info(docker: &Docker) -> String{
    let data = docker.version().await.unwrap();
    format!("OS: {}\nKernel: {}\nPlatform: {:?}\nVersion: {}\nApi: {}\nArch: {}", data.os.unwrap(), data.kernel_version.unwrap(), data.platform.unwrap().name, data.version.unwrap(), data.api_version.unwrap(), data.arch.unwrap())
}

async fn container_command_handler(bot: &Bot, msg: &Message, com: &Vec<&str>, docker: &Docker)-> ResponseResult<()>{
    if com.len() >= 2{
        if com[1] == "list" || com[1] == ""{
            bot.send_message(msg.chat.id, get_containers_info(&docker).await).await?;
        } else if com[1] == "detail" || com[1] == "det"{
            if com.len() >=3 {
                bot.send_message(msg.chat.id, get_container_details(docker, com[2].to_string()).await).await?;
            } else {
                bot.send_message(msg.chat.id, "Please provide container name").await?;
            }
        } else if com[1] == "stop"{
            if com.len() >=3 {
                bot.send_message(msg.chat.id, stop_container(docker, com[2].to_string()).await).await?;
            } else {
                bot.send_message(msg.chat.id, "Please provide container name").await?;
            }
        } else if com[1] == "start"{
            if com.len() >=3 {
                bot.send_message(msg.chat.id, start_container(docker, com[2].to_string()).await).await?;
            } else {
                bot.send_message(msg.chat.id, "Please provide container name").await?;
            }
        } else if com[1] == "pause"{
            if com.len() >=3 {
                bot.send_message(msg.chat.id, pause_container(docker, com[2].to_string()).await).await?;
            } else {
                bot.send_message(msg.chat.id, "Please provide container name").await?;
            }
        } else if com[1] == "unpause"{
            if com.len() >=3 {
                bot.send_message(msg.chat.id, unpause_container(docker, com[2].to_string()).await).await?;
            } else {
                bot.send_message(msg.chat.id, "Please provide container name").await?;
            }
        } else if com[1] == "kill" {
            if com.len() >=3 {
                bot.send_message(msg.chat.id, kill_container(docker, com[2].to_string()).await).await?;
            } else {
                bot.send_message(msg.chat.id, "Please provide container name").await?;
            }
        } else if com[1] == "restart"{
            if com.len() >=3 {
                bot.send_message(msg.chat.id, restart_container(docker, com[2].to_string()).await).await?;
            } else {
                bot.send_message(msg.chat.id, "Please provide container name").await?;
            }
        } else if com[1] == "rename"{
            if com.len() >= 4{
                bot.send_message(msg.chat.id, rename_container(docker, com[2].to_string(), com[3].to_string()).await).await?;
            } else {
                bot.send_message(msg.chat.id, "Please provide old container name and new conteiner name").await?;
            }
        } else if com[1] == "prune"{
            bot.send_message(msg.chat.id, prune_container(docker).await).await?;
        }
    } else {
        bot.send_message(msg.chat.id, get_containers_info(&docker).await).await?;
    }
    Ok(())
}

async fn get_containers_info(docker: &Docker) -> String{
    let option = bollard::container::ListContainersOptions::<String>{all:true, ..Default::default()};
    let data = docker.list_containers(Some(option)).await.unwrap();

    let mut message = String::new();

    for conteiner in data{
        message += format!("Names: {:?}\nState: {}\nStatus: {}\n", conteiner.names.unwrap(), conteiner.state.unwrap(), conteiner.status.unwrap()).as_str();
    }

    message
}

async fn get_container_details(docker: &Docker, name: String) -> String {
    let mut filters = HashMap::new();
    filters.insert("name".to_string(), vec![name]);
    let option = bollard::container::ListContainersOptions::<String> {
        all: true,
        filters,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(option)).await.unwrap();
    let mut message = String::new();

    for container in containers {
        message += &format!("Id: {}\n", container.id.as_ref().unwrap_or(&"N/A".to_string()));
        message += &format!("Names: {:?}\n", container.names.as_ref().unwrap_or(&Vec::new()));
        message += &format!("Image: {}\n", container.image.as_ref().unwrap_or(&"N/A".to_string()));

        message += "Ports:\n";
        if let Some(ports) = &container.ports {
            for port in ports {
                message += &format!(
                    "  IP: {}\n  Private: {}\n  Public: {}\n  Type: {}\n",
                    port.ip.as_ref().unwrap_or(&"N/A".to_string()),
                    port.private_port,
                    port.public_port.as_ref().unwrap_or(&0),
                    port.typ.as_ref().unwrap()
                );
            }
        } else {
            message += "  N/A\n";
        }

        message += "Labels:\n";
        if let Some(labels) = &container.labels {
            for (label1, label2) in labels {
                message += &format!("  {}: {}\n", label1, label2);
            }
        } else {
            message += "  N/A\n";
        }
        message += "Networks:\n";
        if let Some(network_settings) = &container.network_settings {
            if let Some(networks) = &network_settings.networks {
                for (network, endpoint) in networks {
                    message += &format!(
                        "  Network: {} ({})\n  IP: {} ({})\n  Gateway: {}\n  DNS: {:?}\n  Links: {:?}\n",
                        network,
                        endpoint.network_id.as_ref().unwrap_or(&"N/A".to_string()),
                        endpoint.ip_address.as_ref().unwrap_or(&"N/A".to_string()),
                        endpoint.ip_prefix_len.as_ref().unwrap_or(&-1),
                        endpoint.gateway.as_ref().unwrap_or(&"N/A".to_string()),
                        endpoint.dns_names.as_ref().unwrap_or(&Vec::new()),
                        endpoint.links.as_ref().unwrap_or(&Vec::new())
                    );
                }
            } else {
                message += "  N/A\n";
            }
        } else {
            message += "  N/A\n";
        }
        message += "Mounts:\n";
        if let Some(mounts) = &container.mounts {
            for mount in mounts {
                message += &format!(
                    "  Type: {}\n  Name: {}\n  Source: {}\n  Destination: {}\n  Driver: {}\n  Mode: {}\n  Read-Write: {}\n  Propagation: {}\n",
                    mount.typ.as_ref().unwrap(),
                    mount.name.as_ref().unwrap_or(&"N/A".to_string()),
                    mount.source.as_ref().unwrap_or(&"N/A".to_string()),
                    mount.destination.as_ref().unwrap_or(&"N/A".to_string()),
                    mount.driver.as_ref().unwrap_or(&"N/A".to_string()),
                    mount.mode.as_ref().unwrap_or(&"N/A".to_string()),
                    mount.rw.as_ref().unwrap_or(&false),
                    mount.propagation.as_ref().unwrap_or(&"N/A".to_string())
                );
            }
        } else {
            message += "  N/A\n";
        }
        message += "\n\n";
    }

    message
}


async fn stop_container(docker: &Docker, name: String) -> String{
    let options = Some(StopContainerOptions{
        t: 30,
    });
    match docker.stop_container(name.as_str(), options).await{
        Ok(_) => "Stoped successfully".to_string(),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn start_container(docker: &Docker, name: String) -> String{
    match docker.start_container(name.as_str(), None::<StartContainerOptions<String>>).await{
        Ok(_) => "Started successfully".to_string(),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn pause_container(docker: &Docker, name: String) -> String{
    match docker.pause_container(name.as_str()).await{
        Ok(_) => "Paused successfully".to_string(),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn unpause_container(docker: &Docker, name: String) -> String{
    match docker.unpause_container(name.as_str()).await{
        Ok(_) => "Unpaused successfully".to_string(),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn kill_container(docker: &Docker, name: String) -> String{
    let options = Some(KillContainerOptions{
        signal: "SIGINT",
    });
    match docker.kill_container(name.as_str(), options).await{
        Ok(_) => "Killed successfully".to_string(),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn restart_container(docker: &Docker, name: String) -> String{
    let options = Some(RestartContainerOptions{
        t: 30,
    });
    match docker.restart_container(name.as_str(), options).await{
        Ok(_) => "Restarted successfully".to_string(),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn rename_container(docker: &Docker, name: String, new_name: String) -> String{
    let required = RenameContainerOptions {
    name: new_name.as_str()
    };
    match docker.rename_container(name.as_str(), required).await{
        Ok(_) => "Renamed successfully".to_string(),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn prune_container(docker: &Docker) -> String {
    let options = PruneContainersOptions::<String> {
        ..Default::default()
    };
    match docker.prune_containers(Some(options)).await{
        Ok(data) => format!("Pruned successfully\nRemoved: {:?}\nFree up space: {}", data.containers_deleted.unwrap_or_default(), data.space_reclaimed.unwrap_or_default()),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn image_command_handler(bot: &Bot, msg: &Message, com: &Vec<&str>, docker: &Docker)-> ResponseResult<()>{
    if com.len() >= 2{
        if com[1] == "list"{
            bot.send_message(msg.chat.id, list_images(docker).await).await?;
        } else if com[1] == "prune"{
            bot.send_message(msg.chat.id, prune_images(docker).await).await?;
        }
    } else {
        bot.send_message(msg.chat.id, list_images(docker).await).await?;
    }
    Ok(())
}

async fn list_images(docker: &Docker) -> String {
    let options = Some(ListImagesOptions::<String>{
        all: true,
        ..Default::default()
      });

      match docker.list_images(options).await{
        Ok(data) => {
            let mut message = String::new();
            for image in data{
                message += format!("Id: {}\nTags: {:?}\nSize: {:?}\nShared Size: {}\nConteiners: {}\n\n",
                image.id,
                image.repo_tags,
                image.size,
                image.shared_size,
                image.containers).as_str();
            }
            return message;
        },
        Err(x) => format!("Failed with err: {x}").to_string()
      }
}

async fn prune_images(docker: &Docker) -> String{
    let options = PruneImagesOptions::<String> {
        ..Default::default()
    };
    match docker.prune_images(Some(options)).await{
        Ok(data) => format!("Pruned successfully\nRemoved: {:?}\nFree up space: {}", data.images_deleted.unwrap_or_default(), data.space_reclaimed.unwrap_or_default()),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn network_command_handler(bot: &Bot, msg: &Message, com: &Vec<&str>, docker: &Docker)-> ResponseResult<()>{
    if com.len() >= 2{
        if com[1] == "list"{
            bot.send_message(msg.chat.id, list_networks(docker).await).await?;
        } else if com[1] == "prune"{
            bot.send_message(msg.chat.id, prune_networks(docker).await).await?;
        }
    } else {
        bot.send_message(msg.chat.id, list_networks(docker).await).await?;
    }
    Ok(())
}

async fn list_networks(docker: &Docker) -> String{
    let options = ListNetworksOptions::<String> {
        ..Default::default()
    };

    match docker.list_networks(Some(options)).await {
        Ok(data) => {
            let mut result = String::new();
            for network in data{
                result.push_str(&format!("Id: {}\n", network.id.as_ref().unwrap_or(&"N/A".to_string())));
                result.push_str(&format!("Name: {}\n", network.name.as_ref().unwrap_or(&"N/A".to_string())));
                result.push_str(&format!("Created: {}\n", network.created.as_ref().unwrap_or(&"N/A".to_string())));
                result.push_str(&format!("Scope: {}\n", network.scope.as_ref().unwrap_or(&"N/A".to_string())));
                result.push_str(&format!("Driver: {}\n", network.driver.as_ref().unwrap_or(&"N/A".to_string())));
                result.push_str(&format!("Enable IPv6: {}\n", network.enable_ipv6.unwrap_or(false)));
                result.push_str(&format!("Internal: {}\n", network.internal.unwrap_or(false)));
                result.push_str(&format!("Attachable: {}\n", network.attachable.unwrap_or(false)));
                result.push_str(&format!("Ingress: {}\n", network.ingress.unwrap_or(false)));
            
                if let Some(ipam) = &network.ipam {
                    result.push_str("IPAM:\n");
                    result.push_str(&format!("  Driver: {}\n", ipam.driver.as_ref().unwrap_or(&"N/A".to_string())));
                    if let Some(config) = &ipam.config {
                        result.push_str("  Configs:\n");
                        for conf in config {
                            result.push_str(&format!("    Subnet: {}\n", conf.subnet.as_ref().unwrap_or(&"N/A".to_string())));
                            result.push_str(&format!("    Gateway: {}\n", conf.gateway.as_ref().unwrap_or(&"N/A".to_string())));
                        }
                    }
                }
            
                if let Some(containers) = &network.containers {
                    result.push_str("Containers:\n");
                    for (id, container) in containers {
                        result.push_str(&format!("  Id: {}\n", id));
                        result.push_str(&format!("    Name: {}\n", container.name.as_ref().unwrap_or(&"N/A".to_string())));
                        result.push_str(&format!("    IPv4 Address: {}\n", container.ipv4_address.as_ref().unwrap_or(&"N/A".to_string())));
                        result.push_str(&format!("    IPv6 Address: {}\n", container.ipv6_address.as_ref().unwrap_or(&"N/A".to_string())));
                    }
                }
            
                if let Some(options) = &network.options {
                    result.push_str("Options:\n");
                    for (key, value) in options {
                        result.push_str(&format!("  {}: {}\n", key, value));
                    }
                }
            
                if let Some(labels) = &network.labels {
                    result.push_str("Labels:\n");
                    for (key, value) in labels {
                        result.push_str(&format!("  {}: {}\n", key, value));
                    }
                }
                result.push_str("\n\n");
            }

            result
        },
        Err(x) => format!("Failed with err: {x}").to_string()
    }  
}

async fn prune_networks(docker: &Docker) -> String{
    let options = PruneNetworksOptions::<String> {
        ..Default::default()
    };
    match docker.prune_networks(Some(options)).await{
        Ok(data) => format!("Pruned successfully\nRemoved: {:?}", data.networks_deleted.unwrap_or_default()),
        Err(x) => format!("Failed with err: {x}")
    }
}

async fn volumes_command_handler(bot: &Bot, msg: &Message, com: &Vec<&str>, docker: &Docker)-> ResponseResult<()>{
    if com.len() >= 2{
        if com[1] == "list"{
            bot.send_message(msg.chat.id, list_volumes(docker).await).await?;
        } else if com[1] == "prune"{
            bot.send_message(msg.chat.id, prune_volumes(docker).await).await?;
        }
    } else {
        bot.send_message(msg.chat.id, list_volumes(docker).await).await?;
    }
    Ok(())
}

async fn list_volumes(docker: &Docker) -> String{
    let options = ListVolumesOptions::<String> {
        ..Default::default()
    };
    match docker.list_volumes(Some(options)).await {
        Ok(data) => {
            let mut result = String::new();

            for volume in data.volumes.unwrap(){
                result.push_str(&format!("Name: {}\n", volume.name));
                result.push_str(&format!("Driver: {}\n", volume.driver));
                result.push_str(&format!("Mountpoint: {}\n", volume.mountpoint));
            
                result.push_str(&format!("Created At: {}\n", volume.created_at.as_ref().unwrap_or(&"N/A".to_string())));
            
                result.push_str("Status:\n");
                if let Some(status) = &volume.status {
                    for (key, value) in status {
                        result.push_str(&format!("  {}: {:?}\n", key, value));
                    }
                } else {
                    result.push_str("  N/A\n");
                }
            
                result.push_str("Labels:\n");
                if volume.labels.is_empty() {
                    result.push_str("  N/A\n");
                } else {
                    for (key, value) in &volume.labels {
                        result.push_str(&format!("  {}: {}\n", key, value));
                    }
                }
            
                result.push_str("Options:\n");
                if volume.options.is_empty() {
                    result.push_str("  N/A\n");
                } else {
                    for (key, value) in &volume.options {
                        result.push_str(&format!("  {}: {}\n", key, value));
                    }
                }
            
                result.push_str("Usage Data:\n");
                if let Some(usage_data) = &volume.usage_data {
                    result.push_str(&format!("  Size: {}\n", usage_data.size));
                    result.push_str(&format!("  Refs: {}\n", usage_data.ref_count));
                } else {
                    result.push_str("  N/A\n");
                }
                result.push_str("\n\n");
            }
            result
        },
        Err(x) => format!("Failed with err: {x}").to_string()
    }
}

async fn prune_volumes(docker: &Docker) -> String{
    let options = PruneVolumesOptions ::<String> {
        ..Default::default()
    };
    match docker.prune_volumes(Some(options)).await{
        Ok(data) => format!("Pruned successfully\nRemoved: {:?}\nFree up space: {}", data.volumes_deleted.unwrap_or_default(), data.space_reclaimed.unwrap_or_default()),
        Err(x) => format!("Failed with err: {x}")
    }
}

fn get_docker_command_help_text() -> String {
    let help_text = r#"
Docker Command Usage:

/docker [subcommand] [arguments]

Available Subcommands:

info or "" (empty)
  Displays information about the Docker installation.

container [sub-subcommand] [arguments]
  Manages Docker containers.
  Sub-subcommands:
    list or ""                  - Lists all containers
    detail or det [name]        - Shows details of a container
    stop [name]                 - Stops a container
    start [name]                - Starts a container
    pause [name]                - Pauses a container
    unpause [name]              - Unpauses a container
    kill [name]                 - Kills a container
    restart [name]              - Restarts a container
    rename [old] [new]          - Renames a container
    prune                       - Removes all stopped containers

image [sub-subcommand] [arguments]
  Manages Docker images.
  Sub-subcommands:
    list                        - Lists all images
    prune                       - Removes unused images

network [sub-subcommand] [arguments]
  Manages Docker networks.
  Sub-subcommands:
    list                        - Lists all networks
    prune                       - Removes unused networks

volume [sub-subcommand] [arguments]
  Manages Docker volumes.
  Sub-subcommands:
    list                        - Lists all volumes
    prune                       - Removes unused volumes

If no subcommand is provided or an invalid subcommand is given, the default behavior is to display the Docker information.

Note: Replace [name], [old], and [new] with the actual names/identifiers of the Docker resources you want to manage.
"#.to_string();

    help_text
}