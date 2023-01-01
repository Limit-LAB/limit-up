mod_use::mod_use!(args, error, ui);

mod core;

r18::init!("tr");

fn main() {
    let args = Args::parse();

    r18::auto_detect!();

    #[cfg(debug_assertions)]
    println!("{:#?}", args);

    match args.cmd.as_ref().unwrap_or(&Command::Tui) {
        Command::Tui => Ui::setup().exec(),
    };
}
