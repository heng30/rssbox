use crate::config;
use reqwest::header::{HeaderMap, ACCEPT, CACHE_CONTROL, USER_AGENT};
use reqwest::{Client, Proxy, Result};

pub fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert(ACCEPT, "*/*".parse().unwrap());

    headers.insert(CACHE_CONTROL, "no-cache".parse().unwrap());
    headers
}

pub fn client(enabled: bool) -> Result<Client> {
    let conf = config::socks5();
    Ok(if enabled {
        let proxy = Proxy::all(format!("socks5://{}:{}", conf.url, conf.port))?;
        Client::builder().proxy(proxy).build()?
    } else {
        Client::new()
    })
}
