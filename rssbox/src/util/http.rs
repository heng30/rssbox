use crate::config;
use reqwest::{Client, Proxy, Result};

pub fn client(enabled: bool) -> Result<Client> {
    let conf = config::socks5();
    Ok(if enabled {
        let proxy = Proxy::all(format!("socks5://{}:{}", conf.url, conf.port))?;
        Client::builder().proxy(proxy).build()?
    } else {
        Client::new()
    })
}
