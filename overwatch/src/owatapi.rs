use super::TodayResponse;
use reqwest;

const OWAPI_BASE: &str = "https://overwatcharcade.today/api/v1/overwatch";
pub const OWTODAY_URL: &str = "https://overwatcharcade.today/overwatch";

pub fn fetch_today() -> Result<TodayResponse, failure::Error> {
    Ok(reqwest::get(&format!("{}/today", OWAPI_BASE))?.json()?)
}
