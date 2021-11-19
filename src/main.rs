use std::env;

use songbird::SerenityInit;

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[allow(clippy::redundant_closure_call)]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
            let content = match command.data.name.as_str() {
                "play" => {
                    (|| async {
                        let name = match command.data.options.first() {
                            Some(name) => match &name.resolved {
                                Some(ApplicationCommandInteractionDataOptionValue::String(
                                    name,
                                )) => name.clone(),
                                _ => return "E9078: Couldn't get argument".to_string(),
                            },
                            None => return "E7058: Couldn't get argument".to_string(),
                        };
                        let name = if !name.starts_with("http") {
                            "ytsearch:".to_string() + name.as_str()
                        } else {
                            name
                        };
                        let manager = match songbird::get(&ctx).await {
                            Some(manager) => manager,
                            None => return "E4057: Couldn't get voice manager".to_string(),
                        };
                        let guild_id = match command.guild_id {
                            Some(id) => id,
                            None => return "E6296: Couldn't get server ID".to_string(),
                        };
                        let channel_id = match match guild_id.to_guild_cached(&ctx).await {
                            Some(guild) => guild,
                            None => return "E6690: Couldn't get server".to_string(),
                        }
                        .voice_states
                        .get(&command.user.id)
                        {
                            Some(state) => match state.channel_id {
                                Some(id) => id,
                                None => return "E1917: Couldn't get VC ID".to_string(),
                            },
                            None => return "Not in VC".to_string(),
                        };
                        let call = manager.join(guild_id, channel_id).await.0;
                        let mut call = call.lock().await;
                        call.enqueue_source(match songbird::ytdl(name.as_str()).await {
                            Ok(input) => input,
                            Err(_) => return "E1212: Couldn't stream source".to_string(),
                        });
                        "Successfully queued song".to_string()
                    })()
                    .await
                }
                "leave" => {
                    (|| async {
                        let manager = match songbird::get(&ctx).await {
                            Some(manager) => manager,
                            None => return "E4057: Couldn't get voice manager".to_string(),
                        };
                        let guild_id = match command.guild_id {
                            Some(id) => id,
                            None => return "E6296: Couldn't get server ID".to_string(),
                        };
                        match manager.get(guild_id) {
                            Some(call) => {
                                let mut call = call.lock().await;
                                call.queue().stop();
                                call.leave().await;
                                "Successfully left call".to_string()
                            }
                            None => "Not in VC".to_string(),
                        }
                    })()
                    .await
                }
                "skip" => {
                    (|| async {
                        let manager = match songbird::get(&ctx).await {
                            Some(manager) => manager,
                            None => return "E4057: Couldn't get voice manager".to_string(),
                        };
                        let guild_id = match command.guild_id {
                            Some(id) => id,
                            None => return "E6296: Couldn't get server ID".to_string(),
                        };
                        match manager.get(guild_id) {
                            Some(call) => {
                                let mut call = call.lock().await;
                                call.queue().skip();
                                "Successfully skipped song".to_string()
                            }
                            None => "Not in VC".to_string(),
                        }
                    })()
                    .await
                }
                "pause" => {
                    (|| async {
                        let manager = match songbird::get(&ctx).await {
                            Some(manager) => manager,
                            None => return "E4057: Couldn't get voice manager".to_string(),
                        };
                        let guild_id = match command.guild_id {
                            Some(id) => id,
                            None => return "E6296: Couldn't get server ID".to_string(),
                        };
                        match manager.get(guild_id) {
                            Some(call) => {
                                let mut call = call.lock().await;
                                call.queue().pause();
                                "Successfully paused song".to_string()
                            }
                            None => "Not in VC".to_string(),
                        }
                    })()
                    .await
                }
                "resume" => {
                    (|| async {
                        let manager = match songbird::get(&ctx).await {
                            Some(manager) => manager,
                            None => return "E4057: Couldn't get voice manager".to_string(),
                        };
                        let guild_id = match command.guild_id {
                            Some(id) => id,
                            None => return "E6296: Couldn't get server ID".to_string(),
                        };
                        match manager.get(guild_id) {
                            Some(call) => {
                                let mut call = call.lock().await;
                                call.queue().resume();
                                "Successfully resumed song".to_string()
                            }
                            None => "Not in VC".to_string(),
                        }
                    })()
                    .await
                }
                "queue" => {
                    (|| async {
                        let manager = match songbird::get(&ctx).await {
                            Some(manager) => manager,
                            None => return "E4057: Couldn't get voice manager".to_string(),
                        };
                        let guild_id = match command.guild_id {
                            Some(id) => id,
                            None => return "E6296: Couldn't get server ID".to_string(),
                        };
                        match manager.get(guild_id) {
                            Some(call) => {
                                let mut call = call.lock().await;
                                let queue = call
                                    .queue()
                                    .current_queue()
                                    .iter()
                                    .map(|track| {
                                        format!(
                                            "{} / <{}>",
                                            track
                                                .metadata()
                                                .title
                                                .clone()
                                                .unwrap_or("Unknown Song".to_string()),
                                            track
                                                .metadata()
                                                .source_url
                                                .clone()
                                                .unwrap_or("https://youtu.be/".to_string())
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                format!("Current queue:\n{}", queue.join("\n"))
                            }
                            None => "Not in VC".to_string(),
                        }
                    })()
                    .await
                }
                _ => "E9101: HOW".to_string(),
            };
            if let Err(why) = command
                .edit_original_interaction_response(&ctx.http, |response| {
                    response
                        .content(content)
                        .allowed_mentions(|mentions| mentions.empty_parse())
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_command =
            ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
                commands
                    .create_application_command(|command| {
                        command
                            .name("play")
                            .description("Play a song")
                            .create_option(|option| {
                                option
                                    .name("name")
                                    .description("the name or a URL of the song")
                                    .kind(ApplicationCommandOptionType::String)
                                    .required(true)
                            })
                    })
                    .create_application_command(|command| {
                        command.name("leave").description("Leave the VC")
                    })
                    .create_application_command(|command| {
                        command.name("skip").description("Skip the current song")
                    })
                    .create_application_command(|command| {
                        command.name("pause").description("Pause current song")
                    })
                    .create_application_command(|command| {
                        command.name("resume").description("Resume current song")
                    })
                    .create_application_command(|command| {
                        command.name("queue").description("See queue")
                    })
            })
            .await;

        println!(
            "I created the following guild command: {:#?}",
            guild_command
        );
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    // Build our client.
    let mut client = Client::builder(token)
        .register_songbird()
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
