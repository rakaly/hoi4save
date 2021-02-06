use crate::Hoi4Date;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Hoi4Save {
    pub player: String,
    pub date: Hoi4Date,
}
