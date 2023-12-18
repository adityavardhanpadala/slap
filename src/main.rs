use chrono::Local;
use nix::unistd;
use screenshots::Screen;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::{fs, io::Write, thread, time};
use tokio::process::Command;
// use oxipng::{optimize_from_memory};

mod config;

const VERSION: &str = env!("CARGO_PKG_VERSION");

async fn make_timelapse<P: AsRef<Path>>(screenshots_dir: P, screenlapses_dir: P) {
    println!("Making timelapse");

    let screenshots_dir = screenshots_dir.as_ref().to_string_lossy();
    let screenlapses_dir = screenlapses_dir.as_ref().to_string_lossy();

    let _ = Command::new("ffmpeg")
        .args([
            "-framerate",
            "24",
            "-pattern_glob",
            "-i",
            &format!("{}/*.png", screenshots_dir),
            &format!("{}/output.mp4", screenlapses_dir),
        ])
        .spawn()
        .expect("Failed to start ffmpeg to create the timelapse")
        .wait()
        .await
        .expect("Failed to complete timelapse creation");
}

#[tokio::main]
async fn main() {
    let opts = config::Opts::parse_opts();

    let mut frames: u64 = 0;

    let screens = Screen::all().expect("Failed to find display");

    // TODO: Check if primary exists and exit if doesn't explaining to the user
    let main: Screen = *screens
        .iter()
        .filter(|x| x.display_info.is_primary)
        .collect::<Vec<_>>()[0];

    // Get hostname
    let hostname = unistd::gethostname().expect("Failed getting hostname");
    let hostname = hostname.into_string().expect("Hostname wasn't valid UTF-8");

    let screenshots_dir = opts.screenshots_dir.to_string_lossy().to_string();
    let screenlapses_dir = opts.screenlapses_dir.to_string_lossy().to_string();

    // Check if there's a snaps dir
    if fs::metadata(&screenshots_dir).is_err() {
        println!("Creating snaps dir: {}", &screenshots_dir);
        _ = tokio::fs::create_dir(&opts.screenshots_dir).await;
    }

    // Input file type
    let mut track_data_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&opts.track_data_file)
        .expect("Failed to open file");

    // remove existing stuff
    clean_existing_stuff(&screenshots_dir, &screenlapses_dir).await;

    // setup handler to create the screenlapse before exiting
    let screenshots_dir_clone = opts.screenshots_dir.clone();
    let screenlapses_dir_clone = opts.screenlapses_dir.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        // make_timelapse(&screenshots_dir, &screenlapses_dir).await;
        make_timelapse(&screenshots_dir_clone, &screenlapses_dir_clone).await;
        println!("Cleaning up snaps");
        let _ =
            tokio::fs::remove_file(&format!("{}/*", screenshots_dir_clone.to_string_lossy())).await;
        std::process::exit(0);
    });

    loop {
        // TODO: check if main screen is locked or not
        save_screenshot(
            &main,
            &hostname,
            frames,
            &mut track_data_file,
            &screenshots_dir,
        );

        thread::sleep(time::Duration::from_secs(60));
        frames += 1;
        let _ = std::io::stdout().flush();

        println!("Captured {} frames", frames);
    }
}

async fn clean_existing_stuff<P: AsRef<Path>>(screenshots_dir: P, screenlapses_dir: P) {
    println!("Emptying snaps dir");
    let screenshots_dir = screenshots_dir.as_ref().to_string_lossy();
    let screenlapses_dir = screenlapses_dir.as_ref().to_string_lossy();
    let _ = tokio::fs::remove_file(format!("{}/*", screenshots_dir)).await;
    let _ = tokio::fs::remove_file(format!("{}/output.mp4", screenlapses_dir)).await;
}

// TODO:shank: cleanup this function (shouldn't require a bazillion args...)
fn save_screenshot(
    main: &Screen,
    hostname: &str,
    frames: u64,
    track_data_file: &mut File,
    screenshots_dir: &str,
) {
    let buf = main
        .capture()
        .unwrap()
        .to_png()
        .expect("Capturing the screen");
    // Also check if the snap size can be reduced
    // Not worth the extra CPU anyway
    // let options = oxipng::Options::from_preset(2);
    // let opt_buf = optimize_from_memory(&buf,&options).expect("Optimization failed");
    // Check if the image is all black (Then don't save)
    // Save image with time to ensure more info with name
    // Get time in HH:MM:SS
    let time = Local::now().format("%H-%M-%S").to_string();
    fs::write(
        format!("{}/{}-{}-{}.png", screenshots_dir, hostname, frames, time),
        buf,
    )
    .unwrap();
    // file <filename>.png
    // duration <seconds>
    let track_data = format!(
        "file {}/{}-{}-{}.png\n duration 0.1\n",
        screenshots_dir, hostname, frames, time
    );
    _ = track_data_file.write_all(track_data.as_bytes());
}
