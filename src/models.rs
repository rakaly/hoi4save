use std::collections::HashMap;

use crate::{de::deserialize_vec_pair, CountryTag, Hoi4Date};
use jomini::JominiDeserialize;

#[derive(JominiDeserialize, Debug, Clone)]
pub struct Hoi4Save {
    pub player: String,
    pub date: Hoi4Date,
    #[jomini(default, deserialize_with = "deserialize_vec_pair")]
    pub countries: Vec<(CountryTag, Country)>,
}

#[derive(JominiDeserialize, Debug, Clone)]
pub struct Country {
    #[jomini(default)]
    pub stability: f64,
    #[jomini(default)]
    pub war_support: f64,
    #[jomini(default)]
    pub variables: HashMap<String, f64>,
}
