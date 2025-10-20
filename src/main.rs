use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Arc;
use std::sync::mpsc::sync_channel;
use tokio::sync::Mutex;

use poise::serenity_prelude as serenity;

pub struct EditMessage { /* private fields */ }

mod metric_clock;

struct Data {
    ping_check_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    metric_clock_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


/*
=========================Custom Commands=========================
 */
///ping to get latency
#[poise::command(
    prefix_command,
    slash_command
)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let ping = ctx.ping().await;
    ctx.say(format!("Ping: {}ms", ping.as_millis())).await?;
    Ok(())
}

///checks ping every x seconds by editing the same message
#[poise::command(
    prefix_command,
    slash_command
)]
async fn check_connection(ctx: Context<'_>,check_freq:Option<u64>,
                          user_to_ping:Option<String>,
                          ping_max:Option<u128>,
)-> Result<(), Error> {

    let user_to_ping= user_to_ping.unwrap_or("0".to_string());
    let check_freq= check_freq.unwrap_or(60);
    let ping_max = ping_max.unwrap_or(999);
    println!("{}", user_to_ping.len());
    println!("{}", user_to_ping!="0");
    if user_to_ping.len()<17 || user_to_ping.len()>19 || user_to_ping=="0" {
        ctx.say("Incorrect user id format, it should be 17-19 digit long and numbers only").await?;
    }

    let initial_message = ctx.say("Checking ping...").await?;
    let message = initial_message.message().await?;
    let ctx_clone = ctx.serenity_context().clone();
    let channel_id = message.channel_id;
    let message_id = message.id;

    // Prevent multiple concurrent tasks
    let ping_check_handle_arc = ctx.data().ping_check_handle.clone();
    {
        let guard = ping_check_handle_arc.lock().await;
        if guard.is_some() {
            ctx.say("Check connection task is already running.").await?;
            return Ok(());
        }
    }

    let handle = tokio::spawn(async move {
        loop {
            let start = std::time::Instant::now();
            if let Ok(mut msg) = channel_id.message(&ctx_clone, message_id).await {
                // perform an edit and measure round-trip latency using explicit builders
                let builder = serenity::EditMessage::new().content("Checking ping...");
                if msg.edit(&ctx_clone, builder).await.is_ok() {
                let elapsed = start.elapsed();
                    let _ = msg
                        .edit(&ctx_clone, serenity::EditMessage::new().content(format!("Ping: {}ms", elapsed.as_millis())))
                        .await;
                    if elapsed.as_millis() > ping_max && user_to_ping != "0" {
                        let _ = channel_id.say(&ctx_clone, format!(
                            "<@{}> General Hi Ping says 'Hi your ping was {}ms for a while, you should reconnect'",
                            user_to_ping, elapsed.as_millis())).await;
                    }
                }
            }
            sleep(Duration::from_secs(check_freq)).await;
        }
    });

    // store handle
    let mut guard = ping_check_handle_arc.lock().await;
    *guard = Some(handle);

    println!("Started check connection task.");
    Ok(())
}
///Stop checking connection
#[poise::command(prefix_command, slash_command)]
async fn stop_check_connection(ctx: Context<'_>) -> Result<(), Error> {
    let ping_check_handle_arc = ctx.data().ping_check_handle.clone();
    let mut guard = ping_check_handle_arc.lock().await;
    if let Some(handle) = guard.take() {
        handle.abort();
        ctx.say("Stopped checking connection.").await?;
        println!("Stopped checking connection.");
    } else {
        ctx.say("No check connection task is running.").await?;
    }
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
    let mut utc_offset_string = format!("+{}", utc_offset);
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
    let mut utc_offset_string = format!("+{}", utc_offset);
    if utc_offset < 0 {
        utc_offset_string = format!("-{}", utc_offset);
    }
    let timezone = utc_offset * 3600;

    let metric_time_now = metric_clock::get_metric_h_m_string(utc_offset);
    let initial_message = ctx.say(format!("🕒 Current Metric Time at UTC{} is: {}", utc_offset_string, metric_time_now)).await?;
    let message = initial_message.message().await?;

    let ctx_clone = ctx.serenity_context().clone();
    let channel_id = message.channel_id;
    let message_id = message.id;

    // Prevent multiple concurrent metric clock tasks
    let metric_handle_arc = ctx.data().metric_clock_handle.clone();
    {
        let guard = metric_handle_arc.lock().await;
        if guard.is_some() {
            ctx.say("Metric clock is already running.").await?;
            return Ok(());
        }
    }

    let utc_offset_string_cloned = utc_offset_string.clone();

    let handle = tokio::spawn(async move {
        let mut last_discord_update = std::time::Instant::now();
        let mut last_metric_second = 0;

        loop {
            let cycle_start = std::time::Instant::now();

            let current_metric_second = metric_clock::get_metric_seconds(&timezone);
            let current_metric_minute = metric_clock::get_metric_minutes(&timezone);
            let current_metric_hour = metric_clock::get_metric_hours(&timezone);

            // Only update Discord every metric minute (86400 milliseconds)
            if last_discord_update.elapsed() >= Duration::from_millis(86400) {
                let mut minute_str = current_metric_minute.to_string();
                let mut hour_str = current_metric_hour.to_string();

                if current_metric_minute < 9 {
                    minute_str = format!("0{}", current_metric_minute);
                }
                if current_metric_hour < 9 {
                    hour_str = format!("0{}", current_metric_hour);
                }
                last_metric_second = current_metric_second;
                let new_content = format!("🕒 Current Metric Time at UTC{} is: {}:{}", utc_offset_string_cloned, hour_str, minute_str);

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

    // store handle
    let mut guard = metric_handle_arc.lock().await;
    println!("Started metric clock task.");
    *guard = Some(handle);

    Ok(())
}

/// Stop the live metric clock
#[poise::command(prefix_command, slash_command)]
async fn stop_metric_clock(ctx: Context<'_>) -> Result<(), Error> {
    let metric_handle_arc = ctx.data().metric_clock_handle.clone();
    let mut guard = metric_handle_arc.lock().await;
    if let Some(handle) = guard.take() {
        handle.abort();
        ctx.say("Stopped metric clock.").await?;
        println!("Stopped metric clock.");
    } else {
        ctx.say("No metric clock is running.").await?;
    }
    Ok(())
}

/// Sync slash commands (Owner only)
#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn sync_slash(ctx: Context<'_>) -> Result<(), Error> {
    let framework_options = ctx.framework().options();
    let commands = &framework_options.commands;

    // This re-runs the global registration
    poise::builtins::register_globally(ctx.serenity_context(), commands).await?;
    ctx.say("Slash commands synced!").await?;
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

    println!("Loading bot...");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ping(),
                metric_time(),
                secure_command(),
                metric_clock(),
                stop_metric_clock(),
                check_connection(),
                stop_check_connection(),
                sync_slash(),
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
                    Ok(_) => println!("✅ Successfully registered slash commands, ready to go!"),
                    Err(e) => println!("❌ Failed to register commands: {}", e),
                }
                Ok(Data {
                    ping_check_handle: Arc::new(Mutex::new(None)),
                    metric_clock_handle: Arc::new(Mutex::new(None)),
                })
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot.

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

