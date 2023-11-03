use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

// #[derive(Debug, Serialize)]
// struct Name<'a> {
//     first: &'a str,
//     last: &'a str,
// }

#[derive(Debug, Serialize)]
struct Person<'a> {
	title: &'a str,
	// name: Name<'a>,
	// marketing: bool,
}

// #[derive(Debug, Serialize)]
// struct Responsibility {
//     marketing: bool,
// }

#[derive(Debug, Deserialize)]
struct Record {
	#[allow(dead_code)]
	id: Thing,
}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
	println!("Db test");
	let db = Surreal::new::<Ws>("localhost:8000").await?;
	db.signin(Root {
		username: "root",
		password: "root",
	})
	.await?;
	db.use_ns("test").use_db("test").await?;

	let person = Person { title: "Mr" };

	let created: Vec<Record> = db.create("person").content(&person).await?;
	println!("Created: {:?}", created);

	let values = db.live("people").await?;

	Ok(())
}
