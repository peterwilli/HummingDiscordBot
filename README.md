# Your trades live on Discord!

![Screenshot](images/screenshot.png)

The bot works by checking the Backend API so that it automatically updates whenever a trade occurs, this way we can consolidate all that is happening in your bots in unified charts and notifications.

## How to set up

### Docker Compose

We build for both ARM and X86 servers

- Set up HummingBot Deploy if you didn't already: https://github.com/hummingbot/deploy?tab=readme-ov-file#installation
- Add the Discord Bot to Deploy's Docker Compose file (under `services:`):
  ```yml
  services:
    ...
    mdh_discord:
        container_name: mdh_discord
        image: public.ecr.aws/x1i4i6b9/mdh-discord-bot:latest
        restart: unless-stopped
        volumes:
          - "./mdh_discord:/storage/mdh_discord:rw"
        command: "--config-path /storage/mdh_discord/config.yml"
        logging:
          driver: "json-file"
          options:
              max-size: "10m"
              max-file: "5"
        networks:
          - emqx-bridge
  ```

- Run `docker compose up` which will pregenerate a config for you... Run `docker compose down` to stop the bot
- In https://discord.com/developers, create a bot and copy the bot token in the config file that is generated in `./mdh_discord`
- Create a channel on your server for trading updates, and copy the channel id in the config file (you may need to enable developer mode on Discord to be able to right click and copy the channel ID). Place this ID also in your new config file
- Invite your bot (it doesn't need admin rights): `hkttps://discord.com/oauth2/authorize?client_id=<your bot client id>&permissions=515396455488&scope=bot` (add your client ID from the Discord Developers page)
- Run `docker compose up -d` for the final start! That should be all, your Discord bot will automatically see what bots you're running and will show trades accordingly.
- Bonus: See the config.yml to enable other features, new features added frequently!
