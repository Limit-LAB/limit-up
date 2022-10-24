use clap::Parser;

mod_use::mod_use!(command);

fn main() {
    let args = Args::parse();
    println!("{:#?}", args);

    // TODO
}
