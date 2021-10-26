use clap::App;

fn main() {
    let matches = App::new("schema")
        .subcommand(
            App::new("setup") //
                .about("Setup database to receive schema change"),
        )
        .get_matches();
}
