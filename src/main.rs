mod_use::mod_use!(args, error, ui);

mod core;

fn run() -> Result<()> {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    println!("{:#?}", args);

    match args.cmd.as_ref().unwrap_or(&Command::Tui) {
        Command::Tui => Ui::setup().exec(),
    };

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[Error]: {}", e);
    }
}
