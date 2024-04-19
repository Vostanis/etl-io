use anyhow::Result;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::Vec;
use pipe_io::core::*;

#[derive(Deserialize, Debug)]
struct RawPrice {
    chart: Chart,
}

#[derive(Serialize, Deserialize, Debug)]
struct PriceRow {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    adj_close: f64,
    volume: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Price(Vec<PriceRow>);

pipeline! {
    @ RawPrice -> Price
    {
        async fn extract(&self, init: &str) -> Result<RawPrice, pipe_io::Error> {
            let data = serde_json::from_str(&init)?;
            Ok(data)
        }

        async fn transform(&self, data: RawPrice) -> Result<Price, pipe_io::Error> {
            let base = &data.chart.result[0];
            let price = &base.indicators.quote[0];
            let adjclose = &base.indicators.adjclose[0].adjclose;
            let dates = &base.date;
            let price_set = price
                .open
                .iter()
                .zip(price.high.iter())
                .zip(price.low.iter())
                .zip(price.close.iter())
                .zip(price.volume.iter())
                .zip(adjclose.iter())
                .zip(dates.iter())
                .map(
                    |((((((open, high), low), close), volume), adj_close), date)| PriceRow {
                        date: date.clone(),
                        open: *open,
                        high: *high,
                        low: *low,
                        close: *close,
                        adj_close: *adj_close,
                        volume: *volume,
                    },
                )
                .collect::<Vec<_>>();       
            Ok(Price(price_set))
        }
    }
}

#[derive(Deserialize, Debug)]
struct Chart {
    result: Vec<ChartResult>,
}

#[derive(Deserialize, Debug)]
struct ChartResult {
    // meta: Meta,

    #[serde(rename = "timestamp", deserialize_with = "de_timestamps")]
    date: Vec<String>,
    indicators: Indicators,
}

// CIK code can either be a 10-digit string, or shortened number; de_cik handles both
fn de_timestamps<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // general deserialisation, followed by match statement (depending on type found)
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        serde_json::Value::Array(vec) => {
            let dates = vec
                .iter()
                .map(|timestamp| {
                    let ts: serde_json::Value = Deserialize::deserialize(timestamp).unwrap();
                    match ts {
                        serde_json::Value::Number(num) => {
                            if let Some(number) = num.as_i64() {
                                let dt = DateTime::from_timestamp(number, 0)
                                    .expect("invalid timestamp - value should be of type i64");
                                dt.date_naive().to_string()
                            } else {
                                panic!("ERROR! Timestamp array element did not cast as type: i64")
                            }
                        }
                        _ => panic!("ERROR! Timestamp array element is not of type: Number"),
                    }
                })
                .collect::<Vec<_>>();
            Ok(dates)
        }
        _ => Err(serde::de::Error::custom(
            "ERROR! Expected an array of timestamps of type: i64",
        )),
    }
}

// #[derive(Deserialize, Debug)]
// #[serde(rename_all = "camelCase")]
// struct Meta {
//     currency: String,
//     symbol: String,
//     exchange_name: String,
// }

#[derive(Deserialize, Debug)]
struct Indicators {
    quote: Vec<Quote>,
    adjclose: Vec<AdjClose>,
}

#[derive(Deserialize, Debug)]
struct Quote {
    open: Vec<f64>,
    high: Vec<f64>,
    close: Vec<f64>,
    low: Vec<f64>,
    volume: Vec<u64>,
}

#[derive(Deserialize, Debug)]
struct AdjClose {
    adjclose: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    // currency: String,
    // exchange: String,
    price: Price,
    #[serde(flatten)]
    fundamentals: Map<String, Vec<Fundamentals>>,
}

#[tokio::main]
async fn main() {
    let json_data = r#"
    {
        "chart": {
            "result": [
                {
                    "meta": {
                        "currency": "USD",
                        "symbol": "NVDA",
                        "exchangeName": "NMS"
                    },
                    "timestamp": [
                        1710862018,
                        1710862019,
                        1710862020
                    ],
                    "indicators": {
                        "quote": [
                            {
                                "open": [
                                    866,
                                    234,
                                    123
                                ],
                                "high": [
                                    877.760009765625,
                                    877.760009765625,
                                    877.760009765625
                                ],
                                "close": [
                                    865.22998046875,
                                    865.22998046875,
                                    865.22998046875
                                ],
                                "low": [
                                    850.1199951171875,
                                    850.1199951171875,
                                    850.1199951171875
                                ],
                                "volume": [
                                    26058699,
                                    26058699,
                                    26058699
                                ]
                            }
                        ],
                        "adjclose": [
                            {
                                "adjclose": [
                                    865.22998046875,
                                    865.22998046875,
                                    865.22998046875
                                ]
                            }
                        ]
                    }
                }
            ],
            "error": null
        }
    }"#;

    let price = Pipe::<RawPrice, Price>::new()
        .extran(json_data)
        .await
        .unwrap();

    // FUNDAMENTALS DATASET
    // create string variables
    let ticker = "NVDA";
    let time_in_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("UNIX time for now")
        .as_secs()
        .to_string();
    let metrics = "quarterlyNetIncome,annualNetIncome,quarterlyTotalRevenue,annualTotalRevenue,quarterlyDilutedEPS,annualDilutedEPS,quarterlyTotalDebt,annualTotalDebt";
    let url = format!("https://query2.finance.yahoo.com/ws/fundamentals-timeseries/v1/finance/timeseries/{ticker}?symbol={ticker}&type={metrics}&period1=1483142400&period2={time_in_unix}");
    println!("{url:#?}");
    let response = reqwest::get(url).await.unwrap().text().await.unwrap();
    let ts: Timeseries = serde_json::from_str(&response).unwrap();
    let metrics = &ts.timeseries.result;
    let mut fdmt: Map<String, Vec<Fundamentals>> = Map::new();
    for metric in metrics {
        match metric {
            Metric::QuarterlyNetIncome {
                quarterly_net_income,
            } => {
                let vec = quarterly_net_income
                    .iter()
                    .map(|x| Fundamentals {
                        date: x.as_of_date.clone(),
                        currency: x.currency_code.clone(),
                        period: x.period_type.clone(),
                        raw: x.reported_value.raw.clone(),
                        fmt: x.reported_value.fmt.clone(),
                    })
                    .collect::<Vec<Fundamentals>>();
                fdmt.insert("Quarterly Net Income".to_string(), vec)
            }
            Metric::AnnualNetIncome { annual_net_income } => {
                let vec = annual_net_income
                    .iter()
                    .map(|x| Fundamentals {
                        date: x.as_of_date.clone(),
                        currency: x.currency_code.clone(),
                        period: x.period_type.clone(),
                        raw: x.reported_value.raw.clone(),
                        fmt: x.reported_value.fmt.clone(),
                    })
                    .collect::<Vec<Fundamentals>>();
                fdmt.insert("Annual Net Income".to_string(), vec)
            }
            Metric::QuarterlyTotalRevenue {
                quarterly_total_revenue,
            } => {
                let vec = quarterly_total_revenue
                    .iter()
                    .map(|x| Fundamentals {
                        date: x.as_of_date.clone(),
                        currency: x.currency_code.clone(),
                        period: x.period_type.clone(),
                        raw: x.reported_value.raw.clone(),
                        fmt: x.reported_value.fmt.clone(),
                    })
                    .collect::<Vec<Fundamentals>>();
                fdmt.insert("Quarterly Total Revenue".to_string(), vec)
            }
            Metric::AnnualTotalRevenue {
                annual_total_revenue,
            } => {
                let vec = annual_total_revenue
                    .iter()
                    .map(|x| Fundamentals {
                        date: x.as_of_date.clone(),
                        currency: x.currency_code.clone(),
                        period: x.period_type.clone(),
                        raw: x.reported_value.raw.clone(),
                        fmt: x.reported_value.fmt.clone(),
                    })
                    .collect::<Vec<Fundamentals>>();
                fdmt.insert("Annual Total Revenue".to_string(), vec)
            }
            Metric::QuarterlyDilutedEPS {
                quarterly_diluted_eps,
            } => {
                let vec = quarterly_diluted_eps
                    .iter()
                    .map(|x| Fundamentals {
                        date: x.as_of_date.clone(),
                        currency: x.currency_code.clone(),
                        period: x.period_type.clone(),
                        raw: x.reported_value.raw.clone(),
                        fmt: x.reported_value.fmt.clone(),
                    })
                    .collect::<Vec<Fundamentals>>();
                fdmt.insert("Quarterly Diluted EPS".to_string(), vec)
            }
            Metric::AnnualDilutedEPS { annual_diluted_eps } => {
                let vec = annual_diluted_eps
                    .iter()
                    .map(|x| Fundamentals {
                        date: x.as_of_date.clone(),
                        currency: x.currency_code.clone(),
                        period: x.period_type.clone(),
                        raw: x.reported_value.raw.clone(),
                        fmt: x.reported_value.fmt.clone(),
                    })
                    .collect::<Vec<Fundamentals>>();
                fdmt.insert("Annual Diluted EPS".to_string(), vec)
            }
            Metric::QuarterlyTotalDebt {
                quarterly_total_debt,
            } => {
                let vec = quarterly_total_debt
                    .iter()
                    .map(|x| Fundamentals {
                        date: x.as_of_date.clone(),
                        currency: x.currency_code.clone(),
                        period: x.period_type.clone(),
                        raw: x.reported_value.raw.clone(),
                        fmt: x.reported_value.fmt.clone(),
                    })
                    .collect::<Vec<Fundamentals>>();
                fdmt.insert("Quarterly Diluted Debt".to_string(), vec)
            }
            Metric::AnnualTotalDebt { annual_total_debt } => {
                let vec: Vec<Fundamentals> = annual_total_debt
                    .iter()
                    .map(|x| Fundamentals {
                        date: x.as_of_date.clone(),
                        currency: x.currency_code.clone(),
                        period: x.period_type.clone(),
                        raw: x.reported_value.raw.clone(),
                        fmt: x.reported_value.fmt.clone(),
                    })
                    .collect::<Vec<Fundamentals>>();
                fdmt.insert("Annual Total Debt".to_string(), vec)
            }
        };
    }

    let dataset = Output {
        // currency: currency.to_string(),
        // exchange: exchange_name.to_string(),
        price: price,
        fundamentals: fdmt.clone(),
    };
    
    println!("{:#?}", &dataset);
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Timeseries {
    pub timeseries: FdmtResponse,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FdmtResponse {
    pub result: Vec<Metric>,
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Metric {
    QuarterlyNetIncome {
        #[serde(rename = "quarterlyNetIncome")]
        quarterly_net_income: Vec<Data>,
    },
    AnnualNetIncome {
        #[serde(rename = "annualNetIncome")]
        annual_net_income: Vec<Data>,
    },
    QuarterlyTotalRevenue {
        #[serde(rename = "quarterlyTotalRevenue")]
        quarterly_total_revenue: Vec<Data>,
    },
    AnnualTotalRevenue {
        #[serde(rename = "annualTotalRevenue")]
        annual_total_revenue: Vec<Data>,
    },
    QuarterlyDilutedEPS {
        #[serde(rename = "quarterlyDilutedEPS")]
        quarterly_diluted_eps: Vec<Data>,
    },
    AnnualDilutedEPS {
        #[serde(rename = "annualDilutedEPS")]
        annual_diluted_eps: Vec<Data>,
    },
    QuarterlyTotalDebt {
        #[serde(rename = "quarterlyTotalDebt")]
        quarterly_total_debt: Vec<Data>,
    },
    AnnualTotalDebt {
        #[serde(rename = "annualTotalDebt")]
        annual_total_debt: Vec<Data>,
    },
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub as_of_date: String,
    pub currency_code: String,
    pub period_type: String,
    pub reported_value: Reported,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Reported {
    pub fmt: String,
    pub raw: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Fundamentals {
    pub date: String,
    pub currency: String,
    pub period: String,
    pub raw: f64,
    pub fmt: String,
}
