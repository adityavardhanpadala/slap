use tokio::process::Command;

use screenshots::Screen;
use std::{thread, fs, time, io::Write};

fn make_timelapse() {
    
    println!("Making timelapse");
    
    let _ = Command::new("ffmpeg")
                        .args(["-framerate", "24", "-i", "snaps/%d.png", "output.mp4"])
                        .spawn()
                        .expect("Failed to create a timelapse");
}

#[tokio::main]
async fn main() {
    println!("slap 0.69 - A simple tool to just take primary screen timelapses");

    let mut frames: u64 = 0;
        
    let screens = Screen::all().unwrap();
    
    let main: Screen = *screens.iter().filter(|x| x.display_info.is_primary == true).collect::<Vec<_>>()[0];
    
    println!("Capturing primary screen {:?}", main);

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
        let buf = main.capture().unwrap().to_png().unwrap();
        fs::write(format!("snaps/{}.png", frames), buf).unwrap(); 
        thread::sleep(time::Duration::from_millis(2000));
        
        frames+=1;
        
        let _ = std::io::stdout().flush();

        println!("Captured {} frames", frames);
    };
}
