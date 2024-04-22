use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Define the struct
#[derive(Insertable, Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

mod schema {
    diesel::table! {
        users {
            id -> Integer,
            name -> Text,
            email -> Text,
        }
    }
}

use schema::users;

// Function to deserialize and insert into DB
fn deserialize_and_insert(data: &str, conn: &mut PgConnection) -> anyhow::Result<()> {
    // Deserialize the data into a User struct
    let user: User = serde_json::from_str(data)?;

    // Insert the user into the database
    diesel::insert_into(users::table)
        .values(&user)
        .execute(conn)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let data = r#"{"id": 1, "name": "John Doe", "email": "john.doe@example.com"}"#;
    let mut conn = 
        PgConnection::establish("postgresql://postgres:password@<host_ip_or_domain>:5433/postgres")
        .expect("failed to connect");
    deserialize_and_insert(data, &mut conn)?;

    Ok(())
}