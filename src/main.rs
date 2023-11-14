use anyhow::anyhow;
use serenity::http::Http;
use serenity::{async_trait, model::prelude::Member};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::info;

struct Bot {
    token: String,
}

#[async_trait]
impl EventHandler for Bot {
    async fn guild_member_addition(&self, _ctx: Context, new_member: Member) {
        let people = include_str!("badpeople.txt");
        let people = people.split("\n").map(|x| x.to_owned()).collect::<Vec<String>>();

        for person in people {
            if new_member.user.name == person {
                let http = Http::new(&self.token);
                match new_member.ban(http, 0).await{
                    Ok(_) => {}
                    Err(e) => {println!("error: {e}")}
                };
                return;
            }
        }

    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MEMBERS;

    let client = Client::builder(&token, intents)
        .event_handler(Bot { token })
        .await
        .expect("Err creating client");

    Ok(client.into())
}
