use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod structs;
use structs::{AppState, AppQueue};

mod utils;
use utils::start_batch_inserts_queue;

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

	let app_state = AppState {
		db: database_pool.clone()
	};
	let app_state_clone = app_state.clone();

	let queue = Arc::new(AppQueue::new());
	let queue_clone = queue.clone();

	tokio::spawn(async move {
		start_batch_inserts_queue(app_state_clone, queue_clone).await;
	});

	HttpServer::new(move || {
		App::new()
			.app_data(Data::new(AppState { db: database_pool.clone() }))
			.app_data(Data::new(queue.clone()))
			.service(create_pessoa)
			.service(get_pessoas)
			.service(get_pessoa_by_id)
			.service(get_contagem_pessoas)
	})
		.bind((host, port.parse().unwrap()))?
		.run()
		.await
}