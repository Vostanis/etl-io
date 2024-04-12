use pipe_io::etl;
use pipe_io::prelude::*;
use serde::{Deserialize, Serialize};

static EXAMPLE: &str = r#"
    {
        "first_name": "kimon",
        "last_name": "vostanis",
        "thoughts": [
            {
                "time": "10:40",
                "thought": "hmm"
            },
            {
                "time": "10:41",
                "thought": "uh-huh"
            },
            {
                "time": "10:43",
                "thought": "nuh-uh"
            }
        ]
    }
"#;

#[derive(Deserialize, Debug)]
pub struct Original {
    first_name: String,
    last_name: String,
    #[serde(rename = "thoughts")]
    thoughts_and_times: Vec<Thought>,
}

#[derive(Deserialize, Debug)]
pub struct Thought {
    // time: String,
    thought: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reformatted {
    first_name: String,
    last_name: String,
    thoughts: Vec<String>,
}

etl! {
    #[Original -> Reformatted]
    {
        async fn extract(&self, data: &str) -> Result<Original, Error> {
            let json: Original = serde_json::from_str(data)?;
            println!("Before:\n=======\n{json:#?}\n");
            Ok(json)
        }

        async fn transform(&self, input: Original) -> Result<Reformatted, Error> {
            let thoughts = input.thoughts_and_times
                .into_iter()
                .map(|row| row.thought)
                .collect();

            Ok(Reformatted {
                first_name: input.first_name,
                last_name: input.last_name,
                thoughts: thoughts,
            })
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let output_data = Pipe::<Original, Reformatted>::new()
        .extran(EXAMPLE)
        .await?;
    println!("After:\n======\n{output_data:#?}");

    Ok(())
}
