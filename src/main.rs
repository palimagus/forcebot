use std::env;

use serenity::async_trait;
use serenity::model::prelude::ChannelId;
use serenity::model::guild::Guild;
use serenity::model::voice::VoiceState;
use serenity::model::channel::{Message, ChannelType};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;
struct UserChannel;

impl TypeMapKey for UserChannel {
    type Value = Vec<String>;
}

async fn reg<S: Into<String>>(ctx: &Context, channel_id: S) {
    let mut data = ctx.data.write().await;
    let channels = data.get_mut::<UserChannel>().unwrap();
    channels.push(channel_id.into());
}

fn connections_count(guild: &Guild, channel_id: ChannelId) -> usize {
    guild.voice_states
        .values()
        .filter(|state| {
            match state.channel_id {
                Some(id) => id == channel_id,
                None => false,
            }
        })
    .count()
}

#[async_trait]
impl EventHandler for Handler {

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let guild = ctx.cache.guild(new.guild_id.unwrap()).unwrap();
        let user_name;
        let from;
        let to;

        match old {
            Some(old) => { from = old.channel_id.unwrap().to_string(); },
            None => { from = "".to_string(); },
        }

        match new.channel_id {
            Some(channel_id) => { to = channel_id.to_string(); },
            None => { to = "".to_string(); },
        }

        match new.member {
            Some(ref member) => { user_name = member.user.name.to_owned(); },
            None => { user_name = "Unknown".to_string(); },
        }

        println!("{} moved from {:?} to {:?}", user_name, from, to);
        // If guild id == 191237328927195147
        // And if to == 1026145931298619543
        // Create a custom channel with the name of the user
        // Move the user to the custom channel
        let guild_id = guild.id.to_owned().to_string();
        if guild_id == "191237328927195147" && to == "1026145931298619543" {
            let channel = guild.create_channel(&ctx, |c| {
                c.name("🔰 Salon de ".to_owned() + &user_name)
                .kind(ChannelType::Voice)
            }).await.unwrap();

            // Save the channel id in a hashmap
            reg(&ctx, channel.id.to_string()).await;
            
            // Get member from new voice state
            let member = new.member.unwrap();
            // Move member to the new channel
            member.move_to_voice_channel(&ctx, channel).await.unwrap();
        }

        // If a member leaved, check how many members are in the channel
        if from != "" {
            let count = connections_count(&guild, ChannelId::from(from.parse::<u64>().unwrap()));
            // Get the list of custom channels ids
            let data = ctx.data.read().await;
            let channels = data.get::<UserChannel>().unwrap();
            // If the channel is a custom channel and if there is no more members in the channel
            if channels.contains(&from) && count == 0 {
                // Delete the channel
                ChannelId::from(from.parse::<u64>().unwrap()).delete(&ctx).await.unwrap();
            }

            println!("{} members left in channel {}", count, from);
        }

    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is online.", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("TOKEN").expect("Expected token.");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;
    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<UserChannel>(Vec::new());
    }
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

