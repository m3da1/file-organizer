use clap::Parser;

mod cli;
mod error;
mod tui;

fn main() {
    let args = cli::MyOrganizer::parse();

    if let Err(e) = cli::organizer_files(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
