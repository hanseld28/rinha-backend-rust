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

	let host = std::env::var("HOST")
		.expect("HOST_URL must be set");

	let port = std::env::var("PORT")
		.expect("PORT must be set");

	let database_pool = PgPoolOptions::new()
		.connect(&database_url)
		.await
		.expect("Error building a connection pool");

	sqlx::query("DROP TABLE IF EXISTS pessoa;")
		.execute(&database_pool)
		.await
		.err();

	sqlx::query(
		"CREATE TABLE IF NOT EXISTS pessoa (
			id VARCHAR(36),
			apelido VARCHAR(32) CONSTRAINT id_apelido_pk PRIMARY KEY,
			nome VARCHAR(100),
			nascimento CHAR(10),
			stack VARCHAR(34000),
			searchable_text TEXT GENERATED ALWAYS AS (
					LOWER(apelido || '|' || nome || '|' || stack)
			) STORED
		);"
	)
		.execute(&database_pool)
		.await
		.err();

		sqlx::query("CREATE EXTENSION IF NOT EXISTS PG_TRGM;")
			.execute(&database_pool)
			.await
			.err();

		sqlx::query("CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_pessoa_searchable_text ON pessoa USING GIST (searchable_text GIST_TRGM_OPS(SIGLEN=64));")
			.execute(&database_pool)
			.await
			.err();

	HttpServer::new(move || {
		App::new()
			.app_data(Data::new(AppState { db: database_pool.clone() }))
			.service(create_pessoa)
			.service(get_pessoas)
			.service(get_pessoa_by_id)
			.service(get_contagem_pessoas)
	})
		.bind((host, port.parse().unwrap()))?
		.run()
		.await
}