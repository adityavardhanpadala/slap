use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "slap")]
#[command(version=crate::VERSION)]
#[command(about = "A simple tool to just take primary screen timelapses")]
pub struct Cli {
    /// directory to save the screenshots
    /// (default: ./snaps/)
    #[arg(short = 's', long)]
    screenshots_dir: Option<PathBuf>,

    /// output directory to save the screenlapse
    /// (default: ./lapses/)
    #[arg(short = 'o', long)]
    screenlapses_dir: Option<PathBuf>,

    /// track data file
    /// (default: ./track.data)
    track_data_file: Option<PathBuf>,
}

/// Opts is the final options that that app shall use.
#[derive(Debug, Clone)]
pub struct Opts {
    pub screenshots_dir: PathBuf,
    pub screenlapses_dir: PathBuf,
    pub track_data_file: PathBuf,
}

impl Opts {
    pub fn parse_opts() -> Self {
        let cli = Cli::parse();
        Self::from(cli)
    }
}

impl From<Cli> for Opts {
    fn from(cli: Cli) -> Self {
        let screenshots_dir = cli.screenshots_dir.unwrap_or_else(|| "./snaps/".into());
        let screenlapses_dir = cli.screenlapses_dir.unwrap_or_else(|| "./lapses/".into());
        let track_data_file = cli.track_data_file.unwrap_or_else(|| "./track.data".into());
        Self {
            screenshots_dir,
            screenlapses_dir,
            track_data_file,
        }
    }
}
