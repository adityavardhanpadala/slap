use tokio::process::Command;

use screenshots::Screen;
use std::{thread, fs, time, io::Write};
use nix::unistd;
use chrono;

fn make_timelapse() {
    
    println!("Making timelapse");
    
    let _ = Command::new("ffmpeg")
                        .args(["-framerate", "24", "-i", "snaps/%d.png", "output.mp4"])
                        .spawn()
                        .expect("Failed to create a timelapse");
}

#[tokio::main]
async fn main() {
    println!("slap 0.69.1 - A simple tool to just take primary screen timelapses");

    let mut frames: u64 = 0;
        
    let screens = Screen::all().unwrap();
    
    // TODO: Check if primary exists and exit if doesn't explaining to the user
    let main: Screen = *screens.iter().filter(|x| x.display_info.is_primary == true).collect::<Vec<_>>()[0];
    
    println!("Capturing primary screen {:?}", main);
    
    // Get hostname
    let hostname = unistd::gethostname().expect("Failed getting hostname");
    let hostname = hostname.into_string().expect("Hostname wasn't valid UTF-8");

    // Check if there's a snaps dir
    if !fs::metadata("snaps").is_ok() {
        println!("Creating snaps dir");
        fs::create_dir("snaps").unwrap();
    }
    
    println!("Emptying snaps dir");
    let _ = tokio::fs::remove_file("snaps/*").await;
    let _ = tokio::fs::remove_file("output.mp4").await;
    

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        make_timelapse();
        println!("Cleaning up snaps");
        let _ = tokio::fs::remove_file("snaps/*").await;
        std::process::exit(0);
    });

    loop {
        // TODO check if main is locked or not
        let buf = main.capture().unwrap().to_png().unwrap();
        // Also check if the snap size can be reduced
        // Check if the image is all black (Then don't save)
        // Save image with time to ensure more info with name
        // Get time in HH:MM:SS
        let time = chrono::Local::now().format("%H-%M-%S").to_string();
        fs::write(format!("snaps/{}-{}-{}.png", hostname,frames,time), buf).unwrap(); 
        thread::sleep(time::Duration::from_secs(60));
        
        frames+=1;
        
        let _ = std::io::stdout().flush();

        println!("Captured {} frames", frames);
    };
}
