use std::{iter, path::PathBuf, time::SystemTime};

use chrono::{NaiveDate, Utc};
use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
};
use tokio::{fs::File, io::AsyncWriteExt};

use super::{helper, DatabaseContainer};

#[group]
#[prefix = "images"]
#[commands(save, random)]
struct Images;

#[command]
#[allowed_roles("Static")]
pub async fn save(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let database = data
        .get::<DatabaseContainer>()
        .expect("Expected DatabaseContainer in TypeMap")
        .clone();
    let database = database.read().await;
    let author_display_name = helper::find_author(ctx, args.clone()).await;
    if let Some(author) = author_display_name {
        for attachment in &msg.attachments {
            let file_name = PathBuf::try_from(attachment.filename.clone())?;
            let image = match attachment.download().await {
                Ok(image) => {
                    println!("Downloading {:?}", file_name);
                    image
                }
                Err(why) => {
                    eprintln!("Error downloading image: {:?}", why);
                    msg.channel_id
                        .say(&ctx.http, "[Save] Error downloading image")
                        .await?;
                    break;
                }
            };
            let mut dest = PathBuf::from("./images/");
            dest.push(&attachment.filename);
            let mut file = match File::create(&dest).await {
                Ok(file) => {
                    println!("Creating {:?}", &dest);
                    file
                }
                Err(why) => {
                    eprintln!("Error creating file {:?}: {:?}", &dest, why);
                    msg.channel_id
                        .say(&ctx.http, "[Save] Error creating file")
                        .await?;
                    break;
                }
            };

            match file.write_all(&image).await {
                Ok(_) => {
                    eprintln!("Wrote {} bytes to file", &attachment.size);
                    msg.channel_id
                        .say(&ctx.http, format!("Saved {:?}", file_name))
                        .await?;
                }
                Err(why) => {
                    eprintln!("Error writing to file: {:?}", why);
                    msg.channel_id
                        .say(&ctx.http, "[Save] Error writing to file")
                        .await?;
                    break;
                }
            };
            let date = Utc::now().date().to_string();
            let file_name = file_name.to_str();
            sqlx::query!(
                r#"INSERT INTO images ( image, author, date ) VALUES ( ?, ?, ? )"#,
                file_name,
                author,
                date
            )
            .execute(&*database)
            .await?;
            let num_of_images = helper::num_of_images(&None, &database).await;
            let reply = format!("Added image #{}.", num_of_images);
            msg.channel_id.say(&ctx.http, reply).await?;
        }
    } else {
        msg.channel_id
            .say(&ctx.http, "[Add] Member not found.")
            .await?;
    }
    Ok(())
}

#[command]
pub async fn random(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let database = data
        .get::<DatabaseContainer>()
        .expect("Expected DatabaseContainer in TypeMap")
        .clone();
    let database = database.read().await;
    let author_display_name = helper::find_author(ctx, args).await;

    let column_length = helper::num_of_images(&author_display_name, &*database).await;
    let rand_rowid: i64 = (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % column_length as u64)
        .try_into()?;

    if let Some(author) = author_display_name {
        let images = sqlx::query!(r#"SELECT image, date FROM images WHERE author = ?"#, author)
            .fetch_all(&*database)
            .await?;
        let date = images.get(rand_rowid as usize).unwrap().date.clone();
        let date = NaiveDate::parse_from_str(&date, "%Y-%m-%dUTC")?
            .format("%b %d, %Y")
            .to_string();
        let file_name = PathBuf::try_from(images.get(rand_rowid as usize).unwrap().image.clone())?;
        let reply = format!(r#"-{} ({})"#, author, date);
        println!("Sending {} to channel", reply);
        msg.channel_id
            .send_files(&ctx.http, iter::once(&file_name), |m| m.content(reply))
            .await?;
    } else {
        let images = sqlx::query!(r#"SELECT image, author, date FROM images"#)
            .fetch_all(&*database)
            .await?;
        let date = images.get(rand_rowid as usize).unwrap().date.clone();
        let date = NaiveDate::parse_from_str(&date, "%Y-%m-%dUTC")?
            .format("%b %d, %Y")
            .to_string();
        let file_name = PathBuf::try_from(images.get(rand_rowid as usize).unwrap().image.clone())?;
        let reply = format!(
            r#"-{} ({})"#,
            images.get(rand_rowid as usize).unwrap().author,
            date,
        );
        println!("Sending {} to channel", reply);
        msg.channel_id
            .send_files(&ctx.http, iter::once(&file_name), |m| m.content(reply))
            .await?;
    }
    Ok(())
}
