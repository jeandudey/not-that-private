use crate::*;

use reqwest::blocking::ClientBuilder;
use scraper::{Html, Selector};
use url::Url;

use serde::Serialize;

use snafu::ResultExt;

const TRAMITES_SAIME_BASE_URL: &str = "https://tramites.saime.gob.ve/index.php";
const FAKE_USER_AGENT: &str = "Mozilla/5.0 (Windows; x86; rv:75.0) Gecko/20100101 Firefox/75.0";

const V: &str = "V";

#[derive(Debug, Serialize)]
struct BuscarSaimeContacto {
    cedula: String,
    nacionalidad: &'static str,
    token: String,
}

pub fn get<C: ToString>(ci: C) -> Result<serde_json::Value, Error> {
    let client = ClientBuilder::new()
        .cookie_store(true)
        .build().unwrap();

    // Create URL for the registration page.
    let mut ep = Url::parse(TRAMITES_SAIME_BASE_URL).unwrap();
    ep.set_query(Some("r=/usuario/usuario/registro"));

    // Do a request to the registration page, we need to get the session cookies
    // and also scrap the Usuario[tokenCSRF] value.
    let req = client.get(ep)
        .header("Host", "tramites.saime.gob.ve")
        .header("Accept-Language", "es-VE,es;q=0.5")
        .header("User-Agent", FAKE_USER_AGENT)
        .build()
        .unwrap();

    let res = client.execute(req).context(Execute)?;

    let token = scrap_crsf_token(&res.text().context(InvalidText)?)?;

    let params = BuscarSaimeContacto {
        cedula: ci.to_string(),
        nacionalidad: V,
        token,
    };

    let mut ep = Url::parse(TRAMITES_SAIME_BASE_URL).unwrap();
    ep.set_query(Some("r=/usuario/usuario/BuscarSaimeContacto"));

    let req = client.post(ep)
        .header("Accept", "application/json, text/javascript, */*; q=0.01")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Accept-Language", "es-VE,es;q=0.5")
        .header("Host", "tramites.saime.gob.ve")
        .header("Origin", "https://tramites.saime.gob.ve")
        .header("Referer", "https://tramites.saime.gob.ve/index.php?r=usuario/usuario/registro")
        .header("User-Agent", "Mozilla/5.0 (Windows; x86; rv:75.0) Gecko/20100101 Firefox/75.0")
        .header("X-Requested-With", "XMLHttpRequest")
        .form(&params)
        .build()
        .unwrap();

    let res = client.execute(req).context(Execute)?;
    let content = res.text().context(InvalidText)?;

    Ok(serde_json::from_str(content.as_str()).context(InvalidJson)?)
}

fn scrap_crsf_token(html: &str) -> Result<String, Error> {
    let document = Html::parse_document(html);
    let csrf_select = Selector::parse(r#"input[name="Usuario[tokenCSRF]"#).unwrap();

    // Select the first element that matches in the document.
    let possible_tokens = document.select(&csrf_select);

    let mut token = None;
    for csrf in possible_tokens {
        let value = csrf.value();
        token = value.attrs().filter(|(k,_)| *k == "value").next();
        if token.is_some() {
            break;
        }
    }

    if let Some(token) = token {
        Ok(token.1.to_string())
    } else {
        Err(Error::CsrfTokenNotFound)
    }
}
