use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use reqwest::{Client, ClientBuilder, Method};
use serde::Deserialize;

#[derive(Parser)]
struct Arguments {
    #[clap(short, long)]
    count: usize,
}

#[derive(Debug, Deserialize)]
struct Recording {
    artist_name: String,
    release_name: String,
    track_name: String,
    recording_mbid: Option<String>,
    listen_count: usize,
}

struct RecordingWithLength {
    recording: Recording,
    length: usize,
}

#[derive(Deserialize)]
struct Recordings {
    recordings: Vec<Recording>,
    total_recording_count: usize,
}

#[derive(Deserialize)]
struct ListenBrainzResponse<T> {
    payload: T,
}

#[derive(Deserialize)]
struct MusicBrainzRecording {
    length: usize,
    // other stuff is uninteresting
}

#[derive(Deserialize)]
struct BadDataMatcher {
    artist_name: Option<String>,
    release_name: Option<String>,
    track_name: Option<String>,
}

impl BadDataMatcher {
    fn matches(&self, recording: &Recording) -> bool {
        let mut matches = true;
        if let Some(name) = &self.artist_name {
            matches &= name == &recording.artist_name;
        }
        if let Some(name) = &self.release_name {
            matches &= name == &recording.release_name;
        }
        if let Some(name) = &self.track_name {
            matches &= name == &recording.track_name;
        }
        matches
    }
}

#[derive(Deserialize)]
struct BadData {
    match_with: BadDataMatcher,
    recording_mbid: Option<String>,
}

async fn get_recordings(first: usize) -> anyhow::Result<Recordings> {
    let request = reqwest::get(
        format!("https://api.listenbrainz.org/1/stats/user/liquidev/recordings?count=100&offset={first}&range=this_year"),
    )
    .await?;
    let bytes = request.bytes().await?;
    let json = std::str::from_utf8(&bytes)?;
    let response = serde_json::from_str::<ListenBrainzResponse<Recordings>>(json)?;
    Ok(response.payload)
}

async fn get_recording_length(client: Client, mbid: &str) -> anyhow::Result<usize> {
    std::fs::create_dir_all("musicbrainz_cache")?;

    println!("getting length of {mbid}");
    let cached_path = format!("musicbrainz_cache/{mbid}.json");
    let json = if let Ok(json) = std::fs::read_to_string(&cached_path) {
        println!("- cached");
        json
    } else {
        println!("- from musicbrainz");
        loop {
            let start = Instant::now();
            let request = client
                .request(
                    Method::GET,
                    format!("https://musicbrainz.org/ws/2/recording/{mbid}?fmt=json"),
                )
                .build()?;
            let response = client.execute(request).await?;
            let end = Instant::now();
            let time_taken = end - start;
            let wait_time = Duration::from_millis(1000).saturating_sub(time_taken);
            tokio::time::sleep(wait_time).await;
            if response.status().as_u16() == 503 {
                println!("getting rate limited, need to restart request");
                continue;
            }

            let bytes = response.bytes().await?;
            std::fs::write(&cached_path, &bytes)?;
            break std::str::from_utf8(&bytes)?.to_owned();
        }
    };

    let response = serde_json::from_str::<MusicBrainzRecording>(&json)?;
    Ok(response.length)
}

async fn a_main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    let mut current_recording_index = 0;
    let mut all_recordings = vec![];
    while all_recordings.len() < args.count {
        let mut batch = get_recordings(current_recording_index).await?;
        if batch.recordings.is_empty() {
            break;
        }
        current_recording_index += batch.recordings.len();
        all_recordings.append(&mut batch.recordings);
        println!("request done, now at {}", all_recordings.len());
    }

    let bad_data = {
        let file = std::fs::read_to_string("bad_data.json")?;
        serde_json::from_str::<Vec<BadData>>(&file)?
    };
    let skipped = {
        let file = std::fs::read_to_string("skip.json")?;
        serde_json::from_str::<Vec<BadDataMatcher>>(&file)?
    };

    let mut recordings_with_length = vec![];
    let client = ClientBuilder::new()
        .user_agent("nerdsniped-by-spotify-wrapped/0.1.0 ( contact@liquidev.net )")
        .build()?;
    for recording in all_recordings {
        println!("getting length for {recording:#?}");
        let skip = skipped.iter().any(|matcher| matcher.matches(&recording));
        if !skip {
            let mbid = recording.recording_mbid.as_deref().or_else(|| {
                bad_data
                    .iter()
                    .find(|bad_datum| bad_datum.match_with.matches(&recording))
                    .and_then(|bad_datum| bad_datum.recording_mbid.as_ref())
                    .map(|mbid| mbid.as_str())
            });
            if let Some(mbid) = mbid {
                let length = get_recording_length(client.clone(), mbid).await?;
                recordings_with_length.push(RecordingWithLength { recording, length })
            } else {
                println!("WARNING: Recording does not have a valid MBID.");
            }
        } else {
            println!("SKIPPING");
        }
    }

    println!("sorting by play time");
    recordings_with_length.sort_by(|a, b| {
        let a_time = a.recording.listen_count * a.length;
        let b_time = b.recording.listen_count * b.length;
        a_time.cmp(&b_time).reverse()
    });

    for (i, result) in recordings_with_length.iter().enumerate() {
        println!(
            "{}. {} - {}",
            i + 1,
            result.recording.artist_name,
            result.recording.track_name
        );
        println!("    from {}", result.recording.release_name);
        let ms_played = result.recording.listen_count * result.length;
        println!("    minutes played: {:0}", (ms_played as f32) / 60000.0);
        println!();
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    match a_main().await {
        Ok(()) => (),
        Err(error) => println!("error: {error}"),
    }
}
