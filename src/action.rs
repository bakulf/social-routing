/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::config::*;
use crate::errors::Error;
use crate::proxy::Proxy;
use salvo::prelude::*;

pub struct RedirectAction {
    redirect_to: String,
    has_params: bool,
    status_code: StatusCode,
}

#[handler]
impl RedirectAction {
    pub fn new(rule: &ConfigRule) -> RedirectAction {
        if rule.redirect_to.is_none() {
            tracing::error!(target: "RedirectAction", rule = rule.name, "Invalid `redirect_to` value");
            std::process::exit(1);
        }

        let status_code = match StatusCode::from_u16(
            rule.redirect_status
                .unwrap_or(StatusCode::TEMPORARY_REDIRECT.as_u16()),
        ) {
            Ok(code) => code,
            Err(_) => {
                tracing::error!(target: "RedirectAction", rule=rule.name, value=rule.redirect_status.unwrap(), "Invalid `redirect_status`");
                std::process::exit(1);
            }
        };

        if !status_code.is_redirection() {
            tracing::error!(target: "RedirectAction", rule=rule.name, value=rule.redirect_status.unwrap(), "The status code is not for redirects");
            std::process::exit(1);
        }

        tracing::info!(target: "RedirectAction", rule=rule.name, url=rule.redirect_to, status_code= status_code.as_u16(),"creating a RedirectAction handler");

        RedirectAction {
            redirect_to: rule.redirect_to.as_deref().unwrap().to_string(),
            has_params: rule.path.is_some() && rule.path.as_ref().unwrap().contains('<'),
            status_code: status_code,
        }
    }

    fn handle(&self, req: &mut Request, res: &mut Response) {
        match self.has_params {
            false => self.redirect(res, &self.redirect_to),
            true => {
                let mut redirect_to = self.redirect_to.clone();
                for key in req.params().keys() {
                    redirect_to =
                        redirect_to.replace(&format!("<{}>", key), req.param(key).unwrap());
                }
                self.redirect(res, &redirect_to)
            }
        }
    }

    fn redirect(&self, res: &mut Response, uri: &str) {
        match Redirect::with_status_code(self.status_code, uri) {
            Ok(v) => v.render(res),
            Err(e) => {
                tracing::error!(target: "RedirectAction", error=e.to_string(), uri= uri, "Invalid redirect");
            }
        }
    }
}

pub struct ProxyAction {
    proxy: Proxy,
}

#[handler]
impl ProxyAction {
    pub fn new(rule: &ConfigRule) -> ProxyAction {
        if rule.proxy_url.is_none() {
            tracing::error!(target: "ProxyAction", rule = rule.name, "Invalid `proxy_url` value");
            std::process::exit(1);
        }

        tracing::info!(target: "ProxyAction", rule=rule.name, url=rule.proxy_url, "creating a ProxyAction handler");

        match Proxy::create(rule.proxy_url.as_deref().unwrap()) {
            Ok(proxy) => ProxyAction { proxy: proxy },
            Err(e) => {
                tracing::error!(target: "ProxyAction", rule = rule.name, url=rule.proxy_url, error=e.to_string(), "Invalid proxy_url");
                std::process::exit(1);
            }
        }
    }

    async fn handle(&self, req: &mut Request, res: &mut Response) -> Result<(), Error> {
        self.proxy.handle(req, res).await
    }
}

#[cfg(test)]
mod tests {
    use crate::config::*;
    use crate::routers;
    use salvo::http::StatusCode;
    use salvo::test::TestClient;

    #[tokio::test]
    async fn test_redirect_action() {
        let config = Config::create_from_filename("tests/configs/004_redirect_action.yaml");

        let resp = TestClient::get(format!("http://127.0.0.1:5800/test1"))
            .send(routers::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test");

        let resp = TestClient::post(format!("http://127.0.0.1:5800/test2/42/b/hello"))
            .send(routers::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test242bhello");

        let resp = TestClient::post(format!("http://127.0.0.1:5800/test3/42/b/hello"))
            .send(routers::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "/test3?a=42&b=hello");

        let resp = TestClient::post(format!("http://127.0.0.1:5800/test4"))
            .send(routers::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::PERMANENT_REDIRECT);
    }
}
