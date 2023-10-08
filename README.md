# Capstone Project

This repository holds my **Capstone Project** for _Rust Programming Bootcamp_ of Summer 2023. It has been developed over the course of 1 week. I estimate it took me about 10-12 hours of work to reach this state.

# Initial proposal

Here is how I described the project when this idea first came to mind.

___

A Telegram bot (or a website?) that holds schedules for my uni. A pretty popular concept I think.

## Features

- Subject has name, lecturer, kind(practice, lecture, labs), which week is it happening(odd, even, both), maybe link to the meeting, maybe a link to the classroom, what group and year it's for.
1. Query what subject is ongoing for a particular group on a particular date.
1. Display shcedule for a particular date.
1. Display shcedule for a general weekday.
1. Display schedule for a particular week.
1. Dsiplay schedule for a general week.
1. Provide option to remember query options(group and year most notably) for ease of use.

## Motivation

I need this in my life.

___

# Current state

Here is a list of features that I ended up developing for the project:

- `\config <group>` command allows to save user's group for further queries.
- `\subject [slot] [date]` queries a singular subject happening at a particular point in time. When `slot` is omitted, bot uses current time to figure out the slot. When `date` is omitted, bot uses current date, similarly. User can omit `slot` but specify `date` using `\subject _ <date>` syntax.
- There is good amount of feedback on invalid input to help user navigate the bot.
- It is possible to store and display meetings associated with schedule(data layout and display types allow so). Sadly, I have not populated database tables with such information, nor have I provided endpoints to do so.

This correlates with points 1, 2 and 6 from the initial proposal.

# Building & running the project

This bot is built from a Linux machine.

Most importantly, you are going to need to provide your own Telegram bot token. You can get one using Telegram's @Botfather.
Create a file called `config.toml` in the root of the repository with the following structure:

```
token = "<your:token>"
```

You'll also need latest stable version of Rust compiler and cargo, as well as `sqlx-cli` that can be installed with `cargo install sqlx-cli`.

Setup the database with:

```
cargo sqlx database setup
```

This will create a shell SQLite database file at `/tmp/test.db`. You then need to populate it with data with this command:

```
cargo run --bin setup -- sqlite:///tmp/test.db
```

You can inspect the program being run by navigating to the `src/bin/setup.rs` file.

Finally, you can run the bot:

```
cargo run --bin schedule-bot
```

Or, to see some nice logs:

```
RUST_LOG=schedule_bot=trace cargo run --bin schedule-bot
```

You can now navigate to your chat with the bot in Telegram and test out some commands.
