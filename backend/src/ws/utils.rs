use actix_web::HttpRequest;

use crate::http_errors::HttpErrors;

pub(super) type KeyValueVec = Vec<(String, String)>;

pub fn parse_protocol_header(req: &HttpRequest) -> Result<(Vec<String>, KeyValueVec), HttpErrors> {
    let mut key_value: Vec<(String, String)> = vec![];
    let mut protocols: Vec<String> = vec![];
    let Some(header) = req.headers().get("sec-websocket-protocol") else {
        return Err(HttpErrors::NoTokenProvided);
    };
    let header = header
        .to_str()
        .map_err(|_| HttpErrors::NoTokenProvided)?
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    let elements = header.splitn(2, ",").collect::<Vec<&str>>();

    for elem in elements {
        let value = elem.splitn(2, ".").collect::<Vec<&str>>();

        if value.len() > 1 {
            key_value.push((value[0].to_string(), value[1].to_string()));
        } else {
            protocols.push(value[0].to_string());
        }
    }

    Ok((protocols, key_value))
}
