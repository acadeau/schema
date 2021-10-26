use clap::App;
use postgres::{Client, Error as PostgresError, Transaction};

// Function to setup schema in postgres database
fn setup_schema(database_transaction: &mut Transaction) -> Result<(), PostgresError> {
    database_transaction.batch_execute(
        "
      CREATE SCHEMA db_state;
    ",
    )
}

fn main() {
    let _matches = App::new("schema")
        .subcommand(
            App::new("setup") //
                .about("Setup database to receive schema change"),
        )
        .get_matches();
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

        let mut transaction = conn.transaction().unwrap();
        let transaction_result = match super::setup_schema(&mut transaction) {
            Ok(_) => transaction.commit(),
            Err(_) => transaction.rollback(),
        };

        assert!(transaction_result.is_ok());

        let check_schema_created = conn
            .query(
                "SELECT EXISTS(SELECT 1 FROM pg_namespace WHERE nspname = 'db_state');",
                &[],
            )
            .unwrap();
        assert_eq!(check_schema_created.len(), 1);
        assert!(check_schema_created[0].get::<usize, bool>(0));
    }
}
