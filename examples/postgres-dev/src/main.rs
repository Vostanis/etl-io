mod model;
mod schema;

use diesel::prelude::*;
use diesel::connection::SimpleConnection;
use model::User;
use schema::users;

fn establish_connection() -> PgConnection {
    PgConnection::establish("postgresql://postgres:password@localhost:5432/postgres")
        .expect("failed to connect to pg db")
}

fn up(conn: &mut PgConnection) -> QueryResult<()> {
    conn.batch_execute(
        r"CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL
        )",
    )
}

fn down(conn: &mut PgConnection) -> QueryResult<()> {
    conn.batch_execute("DROP TABLE users")
}

fn delete_user(id: i32) {
    let mut connection = establish_connection();
    diesel::delete(users::table.filter(users::id.eq(id)))
        .execute(&mut connection)
        .expect("Error deleting user");
}

fn main() -> anyhow::Result<()> {
    let data = r#"{"id": 1, "name": "John Doe", "email": "john.doe@example.com"}"#;
    let user: User = serde_json::from_str(data)?;

    let mut conn = establish_connection();
    let _ = up(&mut conn);
    diesel::insert_into(users::table)
        .values(user)
        .get_result::<(i32, String, String)>(&mut conn)
        .expect("failed saving new user");
    let _ = delete_user(1);
    let _ = down(&mut conn);

    Ok(())
}   

