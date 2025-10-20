use std::fs;
use std::time::Duration;
use tokio::time::sleep;

use poise::serenity_prelude as serenity;

// TODO : create ping voicechat mesurerment command

struct ClockState {
    running: bool,
}
pub struct EditMessage { /* private fields */ }

mod metric_clock;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


/*
=========================Custom Comands=========================
 */
///Pong!
#[poise::command(
    prefix_command,
    slash_command
)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command
)]
/// Shows current time in metric (decimal) format (UTC+2 by default, specify offset in hours)
async fn metric_time(ctx: Context<'_>, utc_offset:Option<i32>
) -> Result<(), Error> {
    let utc_offset = utc_offset.unwrap_or(2);
    let mut utc_offset_string = format!("+{}", utc_offset);;
    if utc_offset<0{
        utc_offset_string = format!("-{}", utc_offset);
    }
    let metric_time_now= metric_clock::get_metric_time_string(utc_offset);
    let time_string = format!("Current Metric Time is {} (UTC{})",metric_time_now, utc_offset_string);
    ctx.say(time_string).await?;
    Ok(())
}

/// Command using role IDs
#[poise::command(prefix_command, slash_command)]
async fn secure_command(ctx: Context<'_>) -> Result<(), Error> {
    let allowed_roles_ids = vec![
        1429573519183843538 // Admin role ID
    ];

    if !check_roles_by_id(ctx, &allowed_roles_ids).await {
        return Ok(());
    }

    ctx.say("Permission granted!").await?;
    Ok(())
}

/// Live metric (decimal) clock that updates every metric minute
#[poise::command(prefix_command, slash_command)]
async fn metric_clock(
    ctx: Context<'_>, utc_offset: Option<i32>
) -> Result<(), Error> {
    let utc_offset = utc_offset.unwrap_or(2);
    if utc_offset < -12 || utc_offset > 14 {
        ctx.say("Please provide a valid UTC offset between -12 and +14.").await?;
        return Ok(());
    }
    let mut utc_offset_string = format!("+{}", utc_offset);;
    if utc_offset<0{
        utc_offset_string = format!("-{}", utc_offset);
    }
    let timezone = utc_offset * 3600;


    let metric_time_now = metric_clock::get_metric_h_m_string(utc_offset);//
    let initial_message = ctx.say(format!("🕒 Current Metric Time at UTC{} is: {}",utc_offset_string, metric_time_now)).await?;
    let message = initial_message.message().await?;

    let ctx_clone = ctx.serenity_context().clone();
    let channel_id = message.channel_id;
    let message_id = message.id;

    tokio::spawn(async move {
        let mut last_discord_update = std::time::Instant::now();
        let mut last_metric_second = 0;

        loop {
            let cycle_start = std::time::Instant::now();

            // Your existing metric clock logic
            let current_metric_second = metric_clock::get_metric_seconds(&timezone);
            let current_metric_minute = metric_clock::get_metric_minutes(&timezone);
            let current_metric_hour = metric_clock::get_metric_hours(&timezone);

            // Only update Discord every metric minute (86400 milliseconds)
            if last_discord_update.elapsed() >= Duration::from_millis(86400) {

                let mut minute_str = current_metric_minute.to_string();
                let mut hour_str = current_metric_hour.to_string();

                if current_metric_minute<9 {
                    minute_str = format!("0{}", current_metric_minute);
                }
                if current_metric_hour<9 {
                    hour_str = format!("0{}", current_metric_hour);
                }
                last_metric_second = current_metric_second;
                let new_content = format!("🕒 Current Metric Time at UTC{} is: {}:{}",utc_offset_string, hour_str, minute_str);

                if let Ok(mut message) = channel_id.message(&ctx_clone, message_id).await {
                    let builder = serenity::EditMessage::new().content(new_content);
                    message.edit(&ctx_clone, builder).await.ok();
                }

                last_discord_update = std::time::Instant::now();
            }

            if current_metric_second != last_metric_second {
                last_metric_second = current_metric_second;
            }

            let cycle_time = cycle_start.elapsed();
            if cycle_time < Duration::from_nanos(864000000) {
                sleep(Duration::from_nanos(864000000) - cycle_time).await;
            }
        }
    });

    Ok(())
}
/*
=========================Utility Functions=========================
 */
async fn check_roles_by_id(ctx: Context<'_>, required_role_ids: &[u64]) -> bool {
    let guild_id = match ctx.guild() {
        Some(guild) => guild.id, // Extract the ID from CacheRef before await
        None => return false,
    };

    // Now use guild_id for the member lookup
    let member = match guild_id.member(ctx, ctx.author().id).await {
        Ok(member) => member,
        Err(_) => return false,
    };

    member.roles.iter().any(|role_id| {
        required_role_ids.contains(&role_id.get())
    })
}

fn get_token() -> String {
    let contents = fs::read_to_string("BOT_TOKEN.txt").expect("Something went wrong reading the file");
    contents
}


#[tokio::main]
async fn main() {
    // bot token from file
    let token = get_token();

    // permissions the bot will request
    let intents = serenity::GatewayIntents::GUILD_MESSAGES |
        serenity::GatewayIntents::MESSAGE_CONTENT |
        serenity::GatewayIntents::non_privileged();


    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ping(),
                metric_time(),
                secure_command(),
                metric_clock(),
            ], // Add your commands to this vector.
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                case_insensitive_commands: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // This registers your application commands globally, making slash commands available.
                match poise::builtins::register_globally(ctx, &framework.options().commands).await {
                    Ok(_) => println!("✅ Successfully registered slash commands!"),
                    Err(e) => println!("❌ Failed to register commands: {}", e),
                }
                Ok(Data {})
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot.

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

