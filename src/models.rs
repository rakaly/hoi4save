use serde::{Deserialize};
use crate::Hoi4Date;

#[derive(Deserialize, Debug, Clone)]
pub struct Hoi4Save {
    pub player: String,
    pub date: Hoi4Date,
}