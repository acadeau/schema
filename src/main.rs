use std::env;

use clap::{App, Arg};
use postgres::{Client, Error as PostgresError};

// Function to setup schema in postgres database
fn setup(client: &mut Client) -> Result<(), PostgresError> {
    let mut transaction = client.transaction()?;
    let initialisation_result = transaction
        .batch_execute("CREATE SCHEMA db_state;")
        .and_then(|_| {
            transaction.batch_execute(
                "
              CREATE TABLE db_state.changes (
                id serial PRIMARY KEY,
                hash text NOT NULL,
                name text NOT NULL
              );",
            )
        });
    match initialisation_result {
        Ok(_) => transaction.commit(),
        Err(_) => transaction.rollback(),
    }
}

fn main() -> Result<(), PostgresError> {
    let matches = App::new("schema")
        .arg(
            Arg::new("database")
                .short('d')
                .value_name("DATABASE URI")
                .about("Sets a custom database uri")
                .takes_value(true),
        )
        .subcommand(
            App::new("setup") //
                .about("Setup database to receive schema change"),
        )
        .get_matches();

    let database_uri = match matches.value_of("database") {
        Some(database) => database.to_string(),
        None => env::var("DATABASE_URI").expect("Can't find database URI"),
    };

    let mut client = postgres::Client::connect(&database_uri, postgres::NoTls)
        .expect("Can't connect to database");

    match matches.subcommand_name() {
        Some("setup") => setup(&mut client)?,
        None => println!("Command was not specified"),
        _ => println!("Some other subcommand was used"),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use testcontainers::{
        clients,
        images::{self, generic::WaitFor},
        Docker,
    };

    #[test]
    fn setup_tool() {
        let docker = clients::Cli::default();

        let db = "test";
        let user = "user-test";
        let password = "user-password";

        let generic_postgres = images::generic::GenericImage::new("postgres:14-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", db)
            .with_env_var("POSTGRES_USER", user)
            .with_env_var("POSTGRES_PASSWORD", password);

        let node = docker.run(generic_postgres);

        let connection_string = &format!(
            "postgres://{}:{}@localhost:{}/{}",
            user,
            password,
            node.get_host_port(5432)
                .expect("container can't retrieve port"),
            db
        );

        let mut conn = postgres::Client::connect(connection_string, postgres::NoTls).unwrap();

        let setup_result = super::setup(&mut conn);

        assert!(setup_result.is_ok());

        let check_schema_created = conn
            .query(
                "SELECT EXISTS(SELECT 1 FROM pg_namespace WHERE nspname = 'db_state');",
                &[],
            )
            .unwrap();
        assert_eq!(check_schema_created.len(), 1);
        assert!(check_schema_created[0].get::<usize, bool>(0));

        let check_table_created = conn
            .query(
                "SELECT EXISTS(SELECT FROM information_schema.tables
                  WHERE  table_schema = 'db_state'
                  AND    table_name   = 'changes');",
                &[],
            )
            .unwrap();
        assert_eq!(check_table_created.len(), 1);
        assert!(check_table_created[0].get::<usize, bool>(0));
    }
}
