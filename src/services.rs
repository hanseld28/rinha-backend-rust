use actix_web::{
  get, post,
  web::{Data, Json, Path, Query},
	HttpResponse, Responder
};
use sqlx::{self, Row};
use uuid::Uuid;

use crate::structs::{Pessoa, AppState, NovaPessoaDTO, PessoaDTO, Params};

#[post("/pessoas")]
pub async fn create_pessoa(state: Data<AppState>, body: Json<NovaPessoaDTO>) -> impl Responder {
	let generated_id: String = Uuid::new_v4().to_string();

	let stack = match body.stack.clone() {
		Some(s) => s,
		None => {
			let empty_vec: Vec<String> = vec![];
			empty_vec
		}
	};

	match sqlx::query_as::<_, Pessoa>(
		"INSERT INTO pessoa (id, apelido, nome, nascimento, stack) VALUES ($1, $2, $3, $4, $5) RETURNING id, apelido, nome, nascimento, stack"
	)
		.bind(generated_id)
		.bind(body.apelido.to_string())
		.bind(body.nome.to_string())
		.bind(body.nascimento.to_string())
		.bind(stack.join(";").to_string())
		.fetch_one(&state.db)
		.await
	{
		Ok(row) => HttpResponse::Created()
			// .insert_header(("Content-Type", "application/json"))
			.insert_header(("Location", format!("/pessoas/{}", row.id)))
			.finish(),
		Err(_) => HttpResponse::InternalServerError()
			.finish(),
	}
}

#[get("/pessoas")]
pub async fn get_pessoas(state: Data<AppState>, query: Query<Params>) -> impl Responder {
	let empty_vec: Vec<Pessoa> = vec![];

	if !query.t.is_empty() {
		let term = format!("%{}%", query.t.to_string());
		return match sqlx::query_as::<_, Pessoa>(
				"SELECT id, apelido, nome, nascimento, stack FROM pessoa WHERE apelido LIKE $1 OR nome LIKE $1 OR stack LIKE $1 LIMIT 50"
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
