use std::{fs::create_dir, path::PathBuf};

use clap::Parser;
use home::home_dir;
use todotui::model::Model;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    directory: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    match args.directory {
        Some(dir) => Model::new(dir).main_loop(),
        None => match home_dir() {
            Some(mut dir) => {
                dir.push("todotui_data");
                if dir.as_path().metadata().is_ok() {
                    Model::new(dir).main_loop();
                    return;
                }
                match create_dir(dir.clone()) {
                    Ok(_) => Model::new(dir).main_loop(),
                    Err(err) => println!("{}", err),
                }
            }
            None => panic!("Home directory discovery failed :("),
        },
    };
}
