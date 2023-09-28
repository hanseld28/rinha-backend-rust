use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

mod structs;
use structs::AppState;

mod services;
use services::{create_pessoa, get_pessoas, get_pessoa_by_id, get_contagem_pessoas};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv().ok();

	let database_url = std::env::var("DATABASE_URL")
		.expect("DATABASE_URL must be set");

	let database_pool = PgPoolOptions::new()
		.max_connections(300)
		.connect(&database_url)
		.await
		.expect("Error building a connection pool");

	sqlx::query(
		"CREATE TABLE IF NOT EXISTS pessoa (
			id VARCHAR(36),
			apelido VARCHAR(32) CONSTRAINT id_apelido_pk PRIMARY KEY,
			nome VARCHAR(100),
			nascimento CHAR(10),
			stack VARCHAR(2048)
		);"
	)
		.execute(&database_pool)
		.await
		.expect("Error creating table 'pessoa'");

	HttpServer::new(move || {
		App::new()
			.app_data(Data::new(AppState { db: database_pool.clone() }))
			.service(create_pessoa)
			.service(get_pessoas)
			.service(get_pessoa_by_id)
			.service(get_contagem_pessoas)
	})
		.bind(("127.0.0.1", 8080))?
		.run()
		.await
}