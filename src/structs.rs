use rayon::{str::ParallelString, prelude::ParallelIterator};
use sqlx::{FromRow, postgres::PgPool};
use serde::{Serialize, Deserialize};

pub type QueueTaskEvent = Pessoa;
pub type AppQueue = deadqueue::unlimited::Queue<QueueTaskEvent>;

#[derive(Clone)]
pub struct AppState {
  pub db: PgPool
}

pub const EMPTY_ARRAY_FLAG: &str = "@@[]";

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
		} else if p.stack.eq(EMPTY_ARRAY_FLAG) {
			Some(vec![])
		} else {
			Some(
				p.stack.par_split(';')
					.filter_map(|s| Some(s.to_owned()))
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