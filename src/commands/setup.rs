use crate::{utilities::types::GuildSettings, Context, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{CreateEmbed, CreateEmbedFooter};
use tracing::info;

///
#[poise::command(
    prefix_command,
    slash_command,
    category = "Settings",
    subcommands("set", "view")
)]
pub async fn prefix(context: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = context.guild_id() {
        let id = guild_id.get();

        let pf = context.data().guild_data.read().await;

        let guild_settings = pf.get(&id);

        match guild_settings {
            Some(guild_settings) => {
                let embed = CreateEmbed::default()
                    .title("Prefix")
                    .description(format!("`{}`", guild_settings.prefix))
                    .footer(CreateEmbedFooter::new("Prefix"));

                let builder = CreateReply::default().embed(embed);

                context.send(builder).await.unwrap();

                Ok(())
            }
            None => {
                Err(Error::from("No guild settings found"))
            }
        }
    } else {
        let embed = CreateEmbed::default()
            .title("Prefix")
            .description("`+`")
            .footer(CreateEmbedFooter::new("Prefix"));

        let builder = CreateReply::default().embed(embed);

        context.send(builder).await.unwrap();

        Ok(())
    }
}

/// Sets the prefix for the guild.
#[poise::command(
    prefix_command,
    slash_command,
    category = "Settings",
    required_permissions = "ADMINISTRATOR",
    guild_only = true
)]
pub async fn set(context: Context<'_>, prefix: Option<String>) -> Result<(), Error> {
    if let Some(guild_id) = context.guild_id() {
        let prefix = prefix.unwrap_or_else(|| "+".to_string());

        if prefix.contains(' ') {
            return Err(Error::from("Prefix cannot contain spaces."));
        }

        let id = guild_id.get();

        let new_prefix = {
            let mut pf = context.data().guild_data.write().await;
            // update guild settings
            let setting = GuildSettings {
                prefix: prefix.clone(),
                owner_id: guild_id
                    .to_guild_cached(context.cache())
                    .unwrap()
                    .owner_id
                    .get(),
                mute_type: "timeout".to_string(),
                mute_role: 0,
                default_mute_duration: 60000,
            };

            let guild_setting = pf.entry(id).or_insert(setting);
            guild_setting.prefix = prefix.clone();

            prefix
        };

        {
            let data = context.data();
            let database = &data.sqlite;
            let guild_id = id as i64;

            let info = sqlx::query!(
                "UPDATE guild_settings SET prefix = ? WHERE guild_id = ?",
                new_prefix,
                guild_id
            )
            .execute(database)
            .await
            .unwrap()
            .rows_affected();

            info!("Prefix set to {new_prefix} for guild {guild_id}, {info} rows affected");
        }

        let embed = CreateEmbed::new()
            .color(0x008b_0000)
            .title("Prefix")
            .description(format!("Prefix set to ```{new_prefix}```"));

        let builder = CreateReply::default().embed(embed);

        context.send(builder).await.unwrap();

        return Ok(());
    }

    Err(Error::from("Not in Guild."))
}

/// Views current guild's prefix commands' prefix.
#[poise::command(prefix_command, slash_command, category = "Settings")]
pub async fn view(context: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = context.guild_id() {
        let id = guild_id.get();
        let pf = context.data().guild_data.read().await;

        let guild_settings = pf.get(&id);

        match guild_settings {
            Some(guild_settings) => {
                let embed = CreateEmbed::default()
                    .title("Prefix")
                    .description(format!("`{}`", guild_settings.prefix))
                    .footer(CreateEmbedFooter::new("Prefix"));

                let builder = CreateReply::default().embed(embed);

                context.send(builder).await.unwrap();

                return Ok(());
            }
            None => {
                return Err(Error::from("No guild settings found"));
            }
        }
    }

    let embed = CreateEmbed::default()
        .title("Prefix")
        .description("`+`")
        .footer(CreateEmbedFooter::new("Prefix"));

    let builder = CreateReply::default().embed(embed);

    context.send(builder).await.unwrap();

    Ok(())
}
