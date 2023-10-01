use sqlx::{FromRow, postgres::PgPool};
use serde::{Serialize, Deserialize};

pub struct AppState {
  pub db: PgPool
}

#[derive(Clone, Serialize, FromRow)]
pub struct Pessoa {
  pub id: String,
  pub apelido: String,
  pub nome: String,
  pub nascimento: String,
  pub stack: String,
}

#[derive(Deserialize)]
pub struct NovaPessoaDTO {
	pub apelido: Option<String>,
	pub nome: Option<String>,
	pub nascimento: Option<String>,
	pub stack: Option<Vec<Option<String>>>,
}

#[derive(Serialize)]
pub struct PessoaDTO {
	pub id: String,
  pub apelido: String,
	pub nome: String,
  pub nascimento: String,
  pub stack: Option<Vec<String>>
}

impl PessoaDTO {
	fn new(id: String, apelido: String, nome: String, nascimento: String, stack: Option<Vec<String>>) -> PessoaDTO {
		PessoaDTO {
			id,
			apelido,
			nome,
			nascimento,
			stack
		}
	}

	pub fn from(p: Pessoa) -> PessoaDTO {
		let stack = if p.stack.is_empty() {
			None
		}	else {
			Some(
				p.stack.split(";")
					.map(|s| s.to_owned())
					.collect::<Vec<String>>()
			)
		};

		PessoaDTO::new(
			p.id.to_string(),
			p.apelido.to_string(),
			p.nome.to_string(),
			p.nascimento.to_string(),
			stack,
		)
	}
}

#[derive(Deserialize)]
pub struct Params {
  pub t: String,
}