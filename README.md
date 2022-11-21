# Aqukinn
![Minato Aqua](Aqukinn.png)

a discord bot built using the [`twilight`](https://github.com/twilight-rs/twilight) library for my own server!

## usage

- if you want to build from source:

```
git clone https://github.com/j1nxie/Aqukinn
cd Aqukinn
cargo run --release
```

- if you downloaded the release: execute the Aqukinn binary through a command-line interface.

## configuration

for now, Aqukinn has very minimal configuration, currently being handled through `.env` file placed next to your current
working directory.

- `DISCORD_TOKEN`: your discord bot token.
- `LAVALINK_PASSWORD`: your lavalink server password, configured through lavalink's `application.yml`.

## license

licensed under either of

*   Apache License, Version 2.0  
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
*   MIT license  
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## contribution

unless you explicitly state otherwise, any contribution intentionally submitted  
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be  
dual licensed as above, without any additional terms or conditions.
