use clap::{App, Arg};

fn main() {
    let matches = App::new("schema")
        .subcommand(
            App::new("add") //
                .about("Adds files to myapp")
                .version("0.1")
                .author("Kevin K.")
                .arg(
                    Arg::new("input")
                        .about("the file to add")
                        .index(1)
                        .required(true),
                ),
        )
        .get_matches();
}
