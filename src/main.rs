use structopt::StructOpt;

mod cli;

fn main() -> std::io::Result<()> {
    let args = cli::MyOrganizer::from_args();
    cli::organizer_files(args.path)?;
    Ok(())
}
