/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::action::*;
use crate::condition::*;
use crate::config::*;
use http::Method;
use salvo::prelude::*;
use salvo::routing::{MethodFilter, PathFilter};

fn create_route(rule: &ConfigRule) -> Router {
    tracing::info!(target: "Routing", rule=rule.name, "creating route");

    let mut router = Router::new();

    if let Some(method) = rule.method.as_deref() {
        let method = Method::from_bytes(method.as_bytes());
        if method.is_err() {
            tracing::error!(target: "Routing", rule=rule.name, method=rule.method, "invalid method");
            std::process::exit(1);
        }

        tracing::info!(target: "Routing", rule=rule.name, method=rule.method, "filtering method");
        router = router.filter(MethodFilter(method.unwrap()));
    }

    if let Some(path) = rule.path.as_deref() {
        if path == "/" || path == "" {
            tracing::info!(target: "Routing", rule=rule.name, "filtering path index");
        } else {
            tracing::info!(target: "Routing", rule=rule.name, path=path, "filtering path");
            router = router.filter(PathFilter::new(path));
        }
    } else {
        router = router.filter(PathFilter::new("<**>"));
    }

    if rule.headers.is_some() {
        for header in rule.headers.as_ref().unwrap() {
            router = router.filter(ConditionHeader::new(&header.name, header.value.as_deref()));
        }
    }

    tracing::info!(target: "Routing", rule=rule.name, action=rule.action, "handling action");
    match rule.action.as_str() {
        "redirect" => router.handle(RedirectAction::new(rule)),
        "proxy" => router.handle(ProxyAction::new(rule)),
        _ => {
            tracing::error!(target: "Routing", rule=rule.name, action=rule.action, "invalid action");
            std::process::exit(1);
        }
    }
}

pub fn routers(rules: &Vec<ConfigRule>) -> Router {
    let mut router = Router::new();

    for rule in rules {
        router = router.push(create_route(rule));
    }

    router
}

#[cfg(test)]
mod tests {
    use crate::config::*;
    use salvo::http::StatusCode;
    use salvo::test::TestClient;

    #[tokio::test]
    async fn test_filter_path() {
        let config = Config::create_from_filename("tests/configs/001_filter_path.yaml");

        let resp = TestClient::get(format!("http://127.0.0.1:5800/notfound"))
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::NOT_FOUND);

        let resp = TestClient::get(format!("http://127.0.0.1:5800/test1"))
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test1");

        let resp = TestClient::post(format!("http://127.0.0.1:5800/test2/with/path"))
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test2");

        let resp = TestClient::put(format!("http://127.0.0.1:5800/test3/a/path/b"))
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test3");
    }

    #[tokio::test]
    async fn test_filter_method() {
        let config = Config::create_from_filename("tests/configs/002_filter_method.yaml");

        let resp = TestClient::post(format!("http://127.0.0.1:5800/whatever"))
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test_post");

        let resp = TestClient::get(format!("http://127.0.0.1:5800/whatever"))
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test_get");

        let resp = TestClient::delete(format!("http://127.0.0.1:5800/whatever"))
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test_delete");
    }

    #[tokio::test]
    async fn test_filter_header_condition() {
        let config = Config::create_from_filename("tests/configs/003_filter_header_condition.yaml");

        let resp = TestClient::get(format!("http://127.0.0.1:5800/whatever"))
            .add_header("Content-Type", "foo", true)
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test1");

        let resp = TestClient::get(format!("http://127.0.0.1:5800/whatever"))
            .add_header("content-type", "bar", true)
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test2");

        let resp = TestClient::get(format!("http://127.0.0.1:5800/whatever"))
            .add_header("foobar", "any value", true)
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test3");

        let resp = TestClient::get(format!("http://127.0.0.1:5800/whatever"))
            .add_header("foo", "any value", true)
            .add_header("bar", "abc", true)
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::TEMPORARY_REDIRECT);
        assert!(resp.headers().contains_key("location"));
        assert_eq!(resp.headers()["location"], "test4");

        let resp = TestClient::get(format!("http://127.0.0.1:5800/whatever"))
            .add_header("foo", "any value", true)
            .add_header("bar", "cba", true)
            .send(super::routers(&config.rules))
            .await;
        assert_eq!(resp.status_code().unwrap(), StatusCode::NOT_FOUND);
    }
}
