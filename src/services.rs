use actix_web::{
  get, post,
  web::{Data, Json, Path, Query},
	HttpResponse, Responder
};
use sqlx::{self, Row};
use uuid::Uuid;
use chrono::NaiveDate;

use crate::structs::{Pessoa, AppState, NovaPessoaDTO, PessoaDTO, Params};

#[post("/pessoas")]
pub async fn create_pessoa(state: Data<AppState>, body: Json<NovaPessoaDTO>) -> impl Responder {
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
		Some(v) => v.iter().any(|s| s.is_none()),
		None => false
	};

	if has_any_null_in_stack {
		return HttpResponse::UnprocessableEntity().finish();
	}

	let stack = match body.stack.clone() {
		Some(v) => v.iter().map(|s| s.clone().unwrap()).collect::<Vec<String>>(),
		None => {
			let empty_vec: Vec<String> = vec![];
			empty_vec
		}
	};

	if stack.iter().any(|s| s.chars().count() > 32) {
		return HttpResponse::UnprocessableEntity().finish();
	}

	let generated_id: String = Uuid::new_v4().to_string();

	match sqlx::query_as::<_, Pessoa>(
		"INSERT INTO pessoa (id, apelido, nome, nascimento, stack) VALUES ($1, $2, $3, $4, $5)"
	)
		.bind(generated_id.clone())
		.bind(apelido.to_string())
		.bind(nome.to_string())
		.bind(nascimento.to_string())
		.bind(stack.join(";").to_string())
		.fetch_optional(&state.db)
		.await
	{
		Ok(_) => HttpResponse::Created()
			// .insert_header(("Content-Type", "application/json"))
			.insert_header(("Location", format!("/pessoas/{}", generated_id)))
			.finish(),
		Err(_) => HttpResponse::BadRequest().finish(),
	}
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
					pessoas.iter()
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
				pessoas.iter()
					.map(|pessoa| PessoaDTO::from(pessoa.clone()))
					.collect::<Vec<PessoaDTO>>()
			),
			Err(_) => HttpResponse::Ok().json(empty_vec),
		}
}

#[get("/pessoas/{id}")]
async fn get_pessoa_by_id(state: Data<AppState>, path: Path<String>) -> impl Responder {
    let id: String = path.into_inner();

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
