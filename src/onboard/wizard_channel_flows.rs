use super::{
    print_bullet, ChannelsConfig, DiscordConfig, Input, StreamMode, TelegramConfig, WebhookConfig,
};
use anyhow::Result;
use console::style;

pub(super) fn setup_telegram_channel(config: &mut ChannelsConfig) -> Result<()> {
    println!();
    println!(
        "  {} {}",
        style("Telegram Setup").white().bold(),
        style("— talk to TopClaw from Telegram").dim()
    );
    print_bullet("1. Open Telegram and message @BotFather");
    print_bullet("2. Send /newbot and follow the prompts");
    print_bullet("3. Copy the bot token and paste it below");
    println!();

    let token: String = Input::new()
        .with_prompt("  Bot token (from @BotFather)")
        .interact_text()?;

    if token.trim().is_empty() {
        println!("  {} Skipped", style("→").dim());
        return Ok(());
    }

    print!("  {} Testing connection... ", style("⏳").dim());
    let token_clone = token.clone();
    let thread_result = std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let url = format!("https://api.telegram.org/bot{token_clone}/getMe");
        let resp = client.get(&url).send()?;
        let ok = resp.status().is_success();
        let data: serde_json::Value = resp.json().unwrap_or_default();
        let bot_name = data
            .get("result")
            .and_then(|r| r.get("username"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        Ok::<_, reqwest::Error>((ok, bot_name))
    })
    .join();
    match thread_result {
        Ok(Ok((true, bot_name))) => {
            println!(
                "\r  {} Connected as @{bot_name}        ",
                style("✅").green().bold()
            );
        }
        _ => {
            println!(
                "\r  {} Connection failed — check your token and try again",
                style("❌").red().bold()
            );
            return Ok(());
        }
    }

    print_bullet(
        "Allowlist your own Telegram identity first (recommended for secure + fast setup).",
    );
    print_bullet(
        "Use your @username without '@' (example: argenis), or your numeric Telegram user ID.",
    );
    print_bullet("Use '*' only for temporary open testing.");

    let users_str: String = Input::new()
        .with_prompt(
            "  Allowed Telegram identities (comma-separated: username without '@' and/or numeric user ID, '*' for all)",
        )
        .allow_empty(true)
        .interact_text()?;

    let allowed_users = if users_str.trim() == "*" {
        vec!["*".into()]
    } else {
        users_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    if allowed_users.is_empty() {
        println!(
            "  {} No users allowlisted — Telegram inbound messages will be denied until you add your username/user ID or '*'.",
            style("⚠").yellow().bold()
        );
    }

    config.telegram = Some(TelegramConfig {
        bot_token: token,
        allowed_users,
        stream_mode: StreamMode::Partial,
        draft_update_interval_ms: 500,
        interrupt_on_new_message: false,
        group_reply: None,
        base_url: None,
    });
    Ok(())
}

pub(super) fn setup_discord_channel(config: &mut ChannelsConfig) -> Result<()> {
    println!();
    println!(
        "  {} {}",
        style("Discord Setup").white().bold(),
        style("— talk to TopClaw from Discord").dim()
    );
    print_bullet("1. Go to https://discord.com/developers/applications");
    print_bullet("2. Create a New Application → Bot → Copy token");
    print_bullet("3. Enable MESSAGE CONTENT intent under Bot settings");
    print_bullet("4. Invite bot to your server with messages permission");
    println!();

    let token: String = Input::new().with_prompt("  Bot token").interact_text()?;

    if token.trim().is_empty() {
        println!("  {} Skipped", style("→").dim());
        return Ok(());
    }

    print!("  {} Testing connection... ", style("⏳").dim());
    let token_clone = token.clone();
    let thread_result = std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get("https://discord.com/api/v10/users/@me")
            .header("Authorization", format!("Bot {token_clone}"))
            .send()?;
        let ok = resp.status().is_success();
        let data: serde_json::Value = resp.json().unwrap_or_default();
        let bot_name = data
            .get("username")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        Ok::<_, reqwest::Error>((ok, bot_name))
    })
    .join();
    match thread_result {
        Ok(Ok((true, bot_name))) => {
            println!(
                "\r  {} Connected as {bot_name}        ",
                style("✅").green().bold()
            );
        }
        _ => {
            println!(
                "\r  {} Connection failed — check your token and try again",
                style("❌").red().bold()
            );
            return Ok(());
        }
    }

    let guild: String = Input::new()
        .with_prompt("  Server (guild) ID (optional, Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    print_bullet("Allowlist your own Discord user ID first (recommended).");
    print_bullet(
        "Get it in Discord: Settings -> Advanced -> Developer Mode (ON), then right-click your profile -> Copy User ID.",
    );
    print_bullet("Use '*' only for temporary open testing.");

    let allowed_users_str: String = Input::new()
        .with_prompt(
            "  Allowed Discord user IDs (comma-separated, recommended: your own ID, '*' for all)",
        )
        .allow_empty(true)
        .interact_text()?;

    let allowed_users = if allowed_users_str.trim().is_empty() {
        vec![]
    } else {
        allowed_users_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    if allowed_users.is_empty() {
        println!(
            "  {} No users allowlisted — Discord inbound messages will be denied until you add IDs or '*'.",
            style("⚠").yellow().bold()
        );
    }

    config.discord = Some(DiscordConfig {
        bot_token: token,
        guild_id: if guild.is_empty() { None } else { Some(guild) },
        allowed_users,
        listen_to_bots: false,
        group_reply: None,
    });
    Ok(())
}

pub(super) fn setup_webhook_channel(config: &mut ChannelsConfig) -> Result<()> {
    println!();
    println!(
        "  {} {}",
        style("Webhook Setup").white().bold(),
        style("— HTTP endpoint for custom integrations").dim()
    );

    let port: String = Input::new()
        .with_prompt("  Port")
        .default("8080".into())
        .interact_text()?;

    let secret: String = Input::new()
        .with_prompt("  Secret (optional, Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    config.webhook = Some(WebhookConfig {
        port: port.parse().unwrap_or(8080),
        secret: if secret.is_empty() {
            None
        } else {
            Some(secret)
        },
    });
    println!(
        "  {} Webhook on port {}",
        style("✅").green().bold(),
        style(&port).cyan()
    );
    Ok(())
}
