# Aqukinn
![Minato Aqua](Aqukinn.png)
a discord bot built using the [`serenity-rs`](https://github.com/serenity-rs/serenity) library for my own server!
# usage
- if you want to build from source:
```
git clone https://github.com/j1nxie/Aqukinn
cd Aqukinn
cargo run --release
```
- if you downloaded the release: execute the Aqukinn binary through a command-line interface.
# configuration
for now, Aqukinn has very minimal configuration, currently being handled through `.env` file placed next to your current
working directory.
- `DISCORD_TOKEN`: your discord bot token.
- `LAVALINK_PASSWORD`: your lavalink server password, configured through lavalink's `application.yml`.
- `RUST_DEBUG`: rust's debug level for your logs.
# releases
you can either build from source using above instructions, or download releases from the releases tab and sidebar.
# acknowledgements
- [serenity-rs](https://github.com/serenity-rs/serenity/)
- [songbird](https://github.com/serenity-rs/songbird/)
- [lavalink-rs](https://gitlab.com/vicky5124/lavalink-rs) and [lavalink](https://github.com/freyacodes/Lavalink)
