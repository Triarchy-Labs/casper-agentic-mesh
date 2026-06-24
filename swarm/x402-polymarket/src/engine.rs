use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Default)]
pub struct Market {
    #[serde(rename = "outcomePrices")]
    pub outcome_prices: Option<Vec<String>>,
}

pub fn extract_macro_sentiment(markets: Vec<Value>) -> Option<f64> {
    let mut aggregated_prob = 0.0;
    let mut count = 0.0;

    for market_val in markets {
        if let Ok(market) = serde_json::from_value::<Market>(market_val)
            && let Some(prices) = market.outcome_prices
            && let Some(yes_price_str) = prices.first()
            && let Ok(yes_price) = yes_price_str.parse::<f64>() {
                aggregated_prob += yes_price;
                count += 1.0;
        }
    }

    if count > 0.0 {
        Some(aggregated_prob / count)
    } else {
        None
    }
}
