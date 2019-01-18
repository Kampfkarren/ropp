use crate::structures::{PlaceId, Result, RoppError};
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE, COOKIE, USER_AGENT},
    Client,
};

pub fn upload_place(place_id: PlaceId, place_content: String, roblosecurity: &str) -> Result<()> {
    match Client::new()
        .post(&format!(
            "https://data.roblox.com/Data/Upload.ashx?assetid={}&type=Place&name=&description=",
            place_id
        ))
        .body(place_content)
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/xml")
        .header(COOKIE, format!(".ROBLOSECURITY={}", roblosecurity))
        .header(USER_AGENT, "Roblox/WinInet")
        .send()
    {
        Err(e) => Err(e.into()),
        Ok(mut response) => {
            if response.text().expect("response wasn't text??") == place_id.to_string() {
                Ok(())
            } else {
                Err(RoppError::PublishError(response.status()))
            }
        }
    }
}
