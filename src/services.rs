use std::sync::Arc;

use rayon::prelude::*;
use actix_web::{
  get, post,
  web::{Data, Json, Path, Query},
	HttpResponse, Responder
};
use redis::AsyncCommands;
use sqlx::{self, Row};
use uuid::Uuid;
use chrono::NaiveDate;

use crate::structs::{Pessoa, AppState, NovaPessoaDTO, PessoaDTO, Params, EMPTY_ARRAY_FLAG, AppQueue};

#[post("/pessoas")]
pub async fn create_pessoa(
	body: Json<NovaPessoaDTO>,
	redis_pool: Data<deadpool_redis::Pool>,
	queue: Data<Arc<AppQueue>>
) -> impl Responder {
	let apelido = match body.apelido.clone() {
		Some(field) => field,
		None => String::from("null")
	};

	if apelido.eq("null") || apelido.chars().count() > 32 {
		return HttpResponse::UnprocessableEntity().finish();
	}

	let nome = match body.nome.clone() {
		Some(field) => field,
		None => String::from("null")
	};

	if nome.eq("null") || nome.chars().count() > 100 {
		return HttpResponse::UnprocessableEntity().finish();
	}

	let nascimento = match body.nascimento.clone() {
		Some(field) => field,
		None => String::from("null")
	};

	if nascimento.eq("null") {
		return HttpResponse::UnprocessableEntity().finish();
	}

	let nascimento_date_parts_iter = nascimento.split("-").map(str::to_string);
	let nascimento_date_parts = nascimento_date_parts_iter.clone().collect::<Vec<String>>();

	if nascimento_date_parts_iter.clone().count() != 3
		|| nascimento_date_parts[0].chars().count() != 4
		|| nascimento_date_parts[1].chars().count() !=2
		|| nascimento_date_parts[1].chars().count() > 12
		|| nascimento_date_parts[2].chars().count() != 2
		|| nascimento_date_parts[2].chars().count() > 31 {
		return HttpResponse::BadRequest().finish();
	}

	if NaiveDate::parse_from_str(&nascimento, "%Y-%m-%d").is_err() {
		return HttpResponse::UnprocessableEntity().finish();
	}

	let has_any_null_in_stack = match body.stack.clone() {
		Some(v) => v.par_iter().any(|s| s.is_none()),
		None => false
	};

	if has_any_null_in_stack {
		return HttpResponse::UnprocessableEntity().finish();
	}

	let exceeded_max_length_by_item_in_stack = match body.stack.clone() {
		Some(v) => v.iter().any(|op| {
			match op {
				Some(s) => s.chars().count() > 32,
				None => false
			}
		}),
		None => false
	};

	if exceeded_max_length_by_item_in_stack {
		return HttpResponse::UnprocessableEntity().finish();
	}

	let stack = match body.stack.clone() {
		Some(v) => v.par_iter().map(|s| s.clone().unwrap()).collect::<Vec<String>>(),
		None => vec![String::from("")],
	};

	let verified_stack = if stack.is_empty() {
		EMPTY_ARRAY_FLAG.to_string()
	} else {
		stack.join(";").to_string()
	};

	let generated_id: String = Uuid::new_v4().to_string();
	let new_pessoa = Pessoa {
		id: generated_id.clone(),
		apelido: apelido.clone(),
		nome: nome.clone(),
		nascimento: nascimento.clone(),
		stack: verified_stack.clone()
	};

	let mut redis = redis_pool.get_ref().get().await.unwrap();

	let existing_persoa: Option<String> = redis.get(
		generated_id.clone()
	).await.unwrap_or(None);

	if existing_persoa.is_some() {
		return HttpResponse::UnprocessableEntity().finish();
	};

	queue.push(new_pessoa.clone());

	let pessoa_dto_json = serde_json::to_string(
		&PessoaDTO::from(new_pessoa.clone())
	).unwrap();

	redis.set::<_, _, ()>(
		&generated_id,
		&pessoa_dto_json
	)
		.await
		.unwrap();

	HttpResponse::Created()
		.insert_header(("Location", format!("/pessoas/{}", generated_id)))
		.finish()
}

#[get("/pessoas")]
pub async fn get_pessoas(state: Data<AppState>, query: Query<Params>) -> impl Responder {
	let empty_vec: Vec<Pessoa> = vec![];

	if !query.t.is_empty() {
		let term = format!("%{}%", query.t.to_lowercase().to_string());
		return match sqlx::query_as::<_, Pessoa>(
				"SELECT id, apelido, nome, nascimento, stack FROM pessoa WHERE searchable_text LIKE $1 LIMIT 50"
			)
				.bind(term)
				.fetch_all(&state.db)
				.await
			{
				Ok(pessoas) => HttpResponse::Ok().json(
					pessoas.par_iter()
						.map(|pessoa| PessoaDTO::from(pessoa.clone()))
						.collect::<Vec<PessoaDTO>>()
				),
				Err(_) => HttpResponse::Ok().json(empty_vec),
			}
	}

	match sqlx::query_as::<_, Pessoa>(
			"SELECT id, apelido, nome, nascimento, stack FROM pessoa"
		)
			.fetch_all(&state.db)
			.await
		{
			Ok(pessoas) => HttpResponse::Ok().json(
				pessoas.par_iter()
					.map(|pessoa| PessoaDTO::from(pessoa.clone()))
					.collect::<Vec<PessoaDTO>>()
			),
			Err(_) => HttpResponse::Ok().json(empty_vec),
		}
}

#[get("/pessoas/{id}")]
async fn get_pessoa_by_id(path: Path<String>, state: Data<AppState>, redis_pool: Data<deadpool_redis::Pool>) -> impl Responder {
    let id: String = path.into_inner();

		let mut redis = redis_pool.get_ref().get().await.unwrap();
		let cached_json_data: Option<String> = redis.get(id.clone()).await.unwrap_or(None);

		if cached_json_data.is_some() {
				return HttpResponse::Ok()
					.content_type("application/json")
					.body(cached_json_data.unwrap());
		}

    match sqlx::query_as::<_, Pessoa>(
    	"SELECT id, apelido, nome, nascimento, stack FROM pessoa WHERE id = $1"
    )
    	.bind(id.to_string())
		.fetch_one(&state.db)
		.await
	{
		Ok(pessoa) => HttpResponse::Ok().json(PessoaDTO::from(pessoa)),
		Err(_) => HttpResponse::NotFound().finish(),
	}
}

#[get("/contagem-pessoas")]
async fn get_contagem_pessoas(state: Data<AppState>) -> impl Responder {
    match sqlx::query("SELECT CAST(COUNT(id) AS TEXT) as count FROM pessoa")
		.fetch_one(&state.db)
		.await
	{
		Ok(row) => HttpResponse::Ok().json(
			row.get::<&str, &str>("count").parse::<i32>().unwrap()
		),
		Err(_) => HttpResponse::Ok().json(0),
	}
}
