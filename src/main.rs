mod_use::mod_use!(args, error, ui);

mod core;

fn main() {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    println!("{:#?}", args);

    match args.cmd.as_ref().unwrap_or(&Command::Tui) {
        Command::Tui => Ui::setup().exec(),
    };
}
