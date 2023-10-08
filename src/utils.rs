use std::{sync::Arc, time::Duration};

use deadqueue::unlimited::Queue;
use sqlx::Postgres;

use crate::structs::{AppState, Pessoa};

pub async fn execute_batch_inserts(state: AppState, queue: Arc<Queue<Pessoa>>) {
	match queue.len() {
		x if x > 0 => {
			let mut query_builder: sqlx::QueryBuilder<Postgres> = sqlx::QueryBuilder::new(
				"INSERT INTO pessoa (id, apelido, nome, nascimento, stack) "
			);

			let mut items: Vec<Pessoa> = vec![];

			for _i in 0..500 {
				if queue.is_empty() {
					break;
				}

				let item: Pessoa = queue.pop().await;
				items.push(item);
			}

			query_builder.push_values(items, |mut b, item| {
				b.push_bind(item.id);
				b.push_bind(item.apelido);
				b.push_bind(item.nome);
				b.push_bind(item.nascimento);
				b.push_bind(item.stack);
			});

			match query_builder.build()
				.execute(&state.db)
				.await
			{
				Ok(_) => (),
				Err(_) => ()
			};
		},
		_ => ()
	}
}

pub async fn start_batch_inserts_queue(state: AppState, queue: Arc<Queue<Pessoa>>) {
	loop {
		tokio::time::sleep(Duration::from_millis(500)).await;

		match queue.len() {
			0 => continue,
			_ => {
				execute_batch_inserts(state.clone(), queue.clone()).await
			}
		}
	}
}