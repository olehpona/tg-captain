# Tg-captain

Tg-captain is simple teloxide bot created for get system info and manage your server at any point in the world with telegram.

## Plugins

Tg-captain was developed for easy addition and updating of the system-based plugin. Each plugin works as a separate recipient of updates that are collected in one dispatcher. I hope that this will help to develop this platform in the future.
For today project contains 3 plugins

1. "Sys". Show stats and info about system state (Dont work properly in docker container)<br>
2. "Transmission". Plugin for basic work with transmission rpc<br>
3. "Docker". Plugin for working with docker (Show list of container, images, volumes, network; detail info about container; manage container state; clean space with prune command)

## Config

Tg-captain parse yaml file as a config with that structure

```
token: <Your telegram bot token>
security: true # set true or false if you want to filter users
admins: [<chat-id>] # list of users that will be allowed to work with tg-captain
plugins: ["docker", "transmission", "sys"] #list of plugins that will be enabled
sys: #only used when sys plugin enabled
  ping:
    #key pair for ping command must be like <Service name>: <http or https>:<ip>:<port>
    Cockpit: http:127.0.0.1:9090
docker: #only used when docker plugin enabled
  mode: unix #Can be http, unix, default (will be used system default configuration to work with docker)
  path: /var/run/docker.sock #Only used in unix and http mode where in unix mode it is path to unix sock and in http mode it is path to http server
transmission: #only used when transmission plugin enabled
  rpc: http://127.0.0.1:9091/transmission/rpc #path to transmission rpc. Must be like <http or https>://<url>/transmission/rpc!
```

## Building

You can build bot only using `cargo build --release`<br> or build docker using Dockerfile in this repo `docker buildx build -t <your container tag> .`

## Running

For running tg-captain local you must set command arg like this `./tg-captain <path-to-your-config>`.
Or you can use this docker compose file example for running it

```
version: "3"
services:
  tg-captain:
    image: <your-image-name>
    container_name: tg-captain
    network_mode: host #used host for simple setup sys ping function
    volumes:
      - <path to docker sock (default is /var/run/docker.sock)>:/var/run/docker.sock:ro # should be set when using unix mode in docker plugin config
      - <path to your config.yml>:/data/config/config.yml:ro
```

Or if you want set tg-captain as servise there is systemd service example

```
Description=Tg captain
After=network.target

[Service]
ExecStart=<path to tg-captain> <path to config>
Restart=on-failure
[Install]
WantedBy=multi-user.target
```
