use screenshots::Screen;
use std::{thread, fs, time, io::Write};
use nix::unistd;
use chrono::Local;
use std::fs::OpenOptions;
// use oxipng::{optimize_from_memory};

const VERSION: &str = env!("CARGO_PKG_VERSION");
/*
fn make_timelapse() {
    
    println!("Making timelapse");
    
    let _ = Command::new("ffmpeg")
                        .args(["-framerate", "24", "-i", "snaps/%d.png", "output.mp4"])
                        .spawn()
                        .expect("Failed to create a timelapse");
}
*/

#[tokio::main]
async fn main() {
    println!("slap {} - A simple tool to just take primary screen timelapses",VERSION);

    let mut frames: u64 = 0;
        
    let screens = Screen::all().expect("Failed to find display");
    
    // TODO: Check if primary exists and exit if doesn't explaining to the user
    let main: Screen = *screens.iter().filter(|x| x.display_info.is_primary).collect::<Vec<_>>()[0];
    
    // Get hostname
    let hostname = unistd::gethostname().expect("Failed getting hostname");
    let hostname = hostname.into_string().expect("Hostname wasn't valid UTF-8");

    // Check if there's a snaps dir
    if fs::metadata("snaps").is_err() {
        println!("Creating snaps dir");
        _ = tokio::fs::create_dir("snaps").await;
    }

    // Input file type
    let mut track_data_file = OpenOptions::new().create(true).append(true).open("track.data").expect("Failed to open file");

    println!("Emptying snaps dir");
    let _ = tokio::fs::remove_file("snaps/*").await;
    let _ = tokio::fs::remove_file("output.mp4").await;

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        // make_timelapse();
        // println!("Cleaning up snaps");
        // let _ = tokio::fs::remove_file("snaps/*").await;
        std::process::exit(0);
    });

    loop {
        // TODO check if main is locked or not
        let buf = main.capture().unwrap().to_png().expect("Capturing the screen");
        // Also check if the snap size can be reduced
        // Not worth the extra CPU anyway
        // let options = oxipng::Options::from_preset(2);
        // let opt_buf = optimize_from_memory(&buf,&options).expect("Optimization failed");
        // Check if the image is all black (Then don't save)
        // Save image with time to ensure more info with name
        // Get time in HH:MM:SS
        let time = Local::now().format("%H-%M-%S").to_string();
        fs::write(format!("snaps/{}-{}-{}.png", hostname,frames,time), buf).unwrap(); 
        // file <filename>.png
        // duration <seconds>
        let track_data = format!("file snaps/{}-{}-{}.png\n duration 0.1\n",hostname,frames,time);
        _ = track_data_file.write_all(track_data.as_bytes());
        thread::sleep(time::Duration::from_secs(60));
        
        frames+=1;
        
        let _ = std::io::stdout().flush();

        println!("Captured {} frames", frames);
    };
}
