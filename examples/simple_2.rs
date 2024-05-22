use pipe_io::{ETL, pipe, pipeline};
use serde::{Deserialize, Serialize};

// Imagine we want to deserialize & reformat the following JSON:
static EXAMPLE: &str = r#"
    {
        "prices": [
            {
                "open": 2000,
                "close": 2100
            },
            {
                "open": 1900,
                "close": 2134
            },
            {
                "open": 2134,
                "close": 2200
            }
        ]
    }
"#;

#[derive(Serialize, Deserialize)]
struct I { // <--- Input Type
    prices: Vec<Prices>,
}

#[derive(Serialize, Deserialize)]
struct Prices {
    open: i32,
    close: i32,
}

// We want to transform this input to the following output:
// {
//     "open": [
//         2000,
//         1900,
//         2134
//     ],
//     "close": [
//         2100,
//         2134,
//         2200
//     ]
// }

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct O { // <--- Output Type
    open: Vec<i32>,
    close: Vec<i32>,
}

pipeline! {
    I -> O {
        async fn extract(&self, data: &str) -> pipe_io::Result<I> {
            let json: I = serde_json::from_str(data)?;
            Ok(json)
        }

        async fn transform(&self, input: I) -> pipe_io::Result<O> {
            let opens = input.prices.iter()
                .map(|cell| cell.open)
                .collect();

            let closes = input.prices.iter()
                .map(|cell| cell.close)
                .collect();

            Ok(O {
                open: opens,
                close: closes,
            })
        }
    }
}

#[tokio::main]
async fn main() {
    let output1 = pipe_io::Pipe::<I, O>::new()
        .extran(EXAMPLE)
        .await
        .expect("Failed to extract and transform the data.");

    let output2 = pipe![I -> O]
        .extran(EXAMPLE)
        .await
        .expect("Failed to extract and transform the data.");

    assert_eq!(output1, output2);
    println!("Simple 2 passed!");
}
