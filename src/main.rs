use std::{env, fs};

use anyhow::anyhow;
use dotenv::dotenv;
use serenity::model::gateway::Ready;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::{command, GuildId, Interaction, InteractionResponseType};
use serenity::prelude::*;
use serenity::{async_trait, model::prelude::Member};

struct Bot;

fn add_name(options: &[CommandDataOption], guild_id: GuildId) -> String {
    let name = match options.get(0) {
        Some(some) => some.resolved.as_ref(),
        None => return String::from("Expected a username to ban"),
    };
    let name = match name {
        Some(some) => some,
        None => return String::from("Problem"),
    };

    if let CommandDataOptionValue::String(name) = name {
        println!("banning {name}");
        let file_contents = match fs::read_to_string(format!("{}", guild_id)) {
            Ok(ok) => ok,
            Err(_) => String::from(""),
        };
        let mut ban_list = file_contents
            .split("\n")
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();
        if ban_list.contains(name) {
            return String::from(format!("{name} is already on the ban list"));
        }
        ban_list.push(name.to_string());

        let _ = fs::write(format!("{guild_id}"), ban_list.join("\n"));

        format!("{name} was successfully added to the banlist")
    } else {
        "Please enter a valid user or string".to_owned()
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // we only want it if we can get a guild id from it, since the banned lists are named
            // by the guild id
            let guild_id = match command.guild_id {
                Some(guild_id) => guild_id,
                None => return,
            };

            // check if invoker of command is an admin
            let invoker = match &command.member {
                Some(some) => some,
                None => return,
            };
            let invoker_admin = match invoker.permissions {
                Some(some) => some,
                None => return,
            }.administrator();

            if !invoker_admin {
                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content("Not an admin"))
                    })
                    .await
                {
                    println!("Cannot respond to slash command: {}", why);
                }
                return
            }

            let content = match command.data.name.as_str() {
                "ban" => add_name(&command.data.options, guild_id),
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        let ban_list = match fs::read_to_string(format!("{}", new_member.guild_id)) {
            Ok(ok) => ok,
            Err(_) => return,
        };
        let ban_list = ban_list
            .split("\n")
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();

        for person in ban_list {
            if new_member.user.name == person.to_owned() {
                match new_member.ban(ctx.http, 0).await {
                    Ok(_) => {}
                    Err(e) => println!("error: {e}"),
                };
                return;
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let _ = command::Command::set_global_application_commands(ctx.http, |commands| {
            commands.create_application_command(|command| {
                command
                    .name("ban")
                    .description("Set a user or username to be banned upon joining this server")
                    .create_option(|option| option.name("user").kind(command::CommandOptionType::String).description("user to ban").required(true))
            })
        })
        .await;
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    // Get the discord token set in `Secrets.toml`
    let token = if let Ok(token) = env::var("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_BANS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Bot {})
        .await
        .expect("Err creating client");

    client.start().await.unwrap();

    Ok(())
}
