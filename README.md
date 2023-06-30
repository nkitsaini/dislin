# Dislin

This is an implementation of a cloudflare worker in rust that can listen to [linear](https://linear.app) webhook triggers
and for any new comment send the same to discord.



# Dev Workflow
Create a `.dev.vars` file and fill with following details
```sh
DISCORD_WEBHOOK_URL= # webhook url for discord channel
LINEAR_API_KEY= # linear api key (used for fetching comment metadata like creator's name
```


## Commands
```sh
cargo test
wrangler dev # or npx wrangler dev
wrangler publish # or npx wrangler publish
```

# Prodcution
Relies on two environments to be preset (both worker secret and normal environments work)
```sh
DISCORD_WEBHOOK_URL= # webhook url for discord channel
LINEAR_API_KEY= # linear api key (used for fetching comment metadata like creator's name
```
