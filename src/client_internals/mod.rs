//! Jenkins Client

use std::fmt::Debug;
use std::string::ToString;

use log::{debug, warn};
use regex::Regex;
use reqwest::{
    StatusCode,
    blocking::{Body, Client, RequestBuilder, Response},
    header::{CONTENT_TYPE, HeaderValue, USER_AGENT},
};
use serde::Serialize;

mod errors;
pub use self::errors::{Error, Result};
mod builder;
pub mod path;
pub use self::builder::JenkinsBuilder;
pub use self::path::{Name, Path};
mod csrf;
mod tree;
pub use self::tree::{TreeBuilder, TreeQueryParam};

/// Helper type for error management
pub mod error {
    pub use super::errors::Action;
    pub use super::errors::ExpectedType;
}

#[derive(Debug, PartialEq)]
struct User {
    username: String,
    password: Option<String>,
}

/// Client struct with the methods to query Jenkins
#[derive(Debug)]
pub struct Jenkins {
    url: String,
    client: Client,
    user: Option<User>,
    csrf_enabled: bool,
    pub(crate) depth: u8,
}

/// Advanced query parameters supported by Jenkins to control the amount of data retrieved
///
/// see [taming-jenkins-json-api-depth-and-tree](https://www.cloudbees.com/blog/taming-jenkins-json-api-depth-and-tree)
#[derive(Debug)]
pub enum AdvancedQuery {
    /// depth query parameter
    Depth(u8),
    /// tree query parameter
    Tree(TreeQueryParam),
}

/// Hidden type used to represent the AdvancedQueryParams as serializer doesn't support enums
#[derive(Debug, Serialize)]
pub(crate) struct InternalAdvancedQueryParams {
    depth: Option<u8>,
    tree: Option<TreeQueryParam>,
}
impl From<AdvancedQuery> for InternalAdvancedQueryParams {
    fn from(query: AdvancedQuery) -> Self {
        match query {
            AdvancedQuery::Depth(depth) => InternalAdvancedQueryParams {
                depth: Some(depth),
                tree: None,
            },
            AdvancedQuery::Tree(tree) => InternalAdvancedQueryParams {
                depth: None,
                tree: Some(tree),
            },
        }
    }
}

impl Jenkins {
    pub(crate) fn url_api_json(&self, endpoint: &str) -> String {
        format!("{}{}/api/json", self.url, endpoint)
    }

    pub(crate) fn url(&self, endpoint: &str) -> String {
        format!("{}{}", self.url, endpoint)
    }

    fn send(&self, mut request_builder: RequestBuilder) -> Result<Response> {
        if let Some(ref user) = self.user {
            request_builder =
                request_builder.basic_auth(user.username.clone(), user.password.clone());
        }
        request_builder = request_builder.header(
            USER_AGENT,
            format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        );
        let query = request_builder.build()?;
        debug!("sending {} {}", query.method(), query.url());
        Ok(self.client.execute(query)?)
    }

    fn error_for_status(response: Response) -> Result<Response> {
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            warn!("got an error: {status}");
        }
        Ok(response.error_for_status()?)
    }

    pub(crate) fn get(&self, path: &Path) -> Result<Response> {
        self.get_with_params(path, [("depth", &self.depth.to_string())])
    }

    pub(crate) fn get_with_params<T: Serialize>(&self, path: &Path, qps: T) -> Result<Response> {
        let query = self
            .client
            .get(&self.url_api_json(&path.to_string()))
            .query(&qps);
        Self::error_for_status(self.send(query)?)
    }

    pub(crate) fn post(&self, path: &Path) -> Result<Response> {
        let mut request_builder = self.client.post(&self.url(&path.to_string()));

        request_builder = self.add_csrf_to_request(request_builder)?;

        Self::error_for_status(self.send(request_builder)?)
    }

    pub(crate) fn post_with_body<T: Into<Body> + Debug>(
        &self,
        path: &Path,
        body: T,
        qps: &[(&str, &str)],
    ) -> Result<Response> {
        let mut request_builder = self.client.post(&self.url(&path.to_string()));

        request_builder = self.add_csrf_to_request(request_builder)?;

        request_builder = request_builder.header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        debug!("{body:?}");
        request_builder = request_builder.query(qps).body(body);
        let response = self.send(request_builder)?;

        if response.status() == StatusCode::INTERNAL_SERVER_ERROR {
            // get the error before reading the body. In this case it can't be OK
            let error = match response.error_for_status_ref() {
                Ok(_) => unreachable!(),
                Err(err) => err,
            };

            let body = response.text()?;

            let re = Regex::new(r"java.lang.([a-zA-Z]+): (.*)").unwrap();
            if let Some(captures) = re.captures(&body) {
                match captures.get(1).map(|v| v.as_str()) {
                    Some("IllegalStateException") => {
                        warn!(
                            "got an IllegalState error: {}",
                            captures.get(0).map(|v| v.as_str()).unwrap_or("unspecified")
                        );
                        Err(Error::IllegalState {
                            message: captures
                                .get(2)
                                .map(|v| v.as_str())
                                .unwrap_or("no message")
                                .to_string(),
                        })
                    }
                    Some("IllegalArgumentException") => {
                        warn!(
                            "got an IllegalArgument error: {}",
                            captures.get(0).map(|v| v.as_str()).unwrap_or("unspecified")
                        );
                        Err(Error::IllegalArgument {
                            message: captures
                                .get(2)
                                .map(|v| v.as_str())
                                .unwrap_or("no message")
                                .to_string(),
                        })
                    }
                    Some(_) => {
                        warn!(
                            "got an Unknwon error: {}",
                            captures.get(0).map(|v| v.as_str()).unwrap_or("unspecified")
                        );
                        Ok(())
                    }
                    _ => Ok(()),
                }?;
            }
            Err(error.into())
        } else {
            Ok(Self::error_for_status(response)?)
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn can_post_with_body() {
        let jenkins_client = crate::JenkinsBuilder::new(&mockito::server_url())
            .disable_csrf()
            .build()
            .unwrap();

        let _mock = mockito::mock("POST", "/mypath").with_body("ok").create();

        let response =
            jenkins_client.post_with_body(&super::Path::Raw { path: "/mypath" }, "body", &[]);

        assert!(response.is_ok());
        assert_eq!(response.unwrap().text().unwrap(), "ok");
    }

    #[test]
    fn can_post_with_body_and_get_error_state() {
        let jenkins_client = crate::JenkinsBuilder::new(&mockito::server_url())
            .disable_csrf()
            .build()
            .unwrap();

        let _mock = mockito::mock("POST", "/error-IllegalStateException")
            .with_status(500)
            .with_body("hviqsuvnqsodjfsqjdgo java.lang.IllegalStateException: my error\nvzfjsd")
            .create();

        let response = jenkins_client.post_with_body(
            &super::Path::Raw {
                path: "/error-IllegalStateException",
            },
            "body",
            &[],
        );

        assert!(response.is_err());
        assert_eq!(
            format!("{:?}", response),
            r#"Err(IllegalState { message: "my error" })"#
        );
    }

    #[test]
    fn can_post_with_body_and_get_error_argument() {
        let jenkins_client = crate::JenkinsBuilder::new(&mockito::server_url())
            .disable_csrf()
            .build()
            .unwrap();

        let _mock = mockito::mock("POST", "/error-IllegalArgumentException")
            .with_status(500)
            .with_body("hviqsuvnqsodjfsqjdgo java.lang.IllegalArgumentException: my error\nvzfjsd")
            .create();

        let response = jenkins_client.post_with_body(
            &super::Path::Raw {
                path: "/error-IllegalArgumentException",
            },
            "body",
            &[],
        );

        assert!(response.is_err());
        assert_eq!(
            format!("{:?}", response),
            r#"Err(IllegalArgument { message: "my error" })"#
        );
    }

    #[test]
    fn can_post_with_body_and_get_error_new() {
        let jenkins_client = crate::JenkinsBuilder::new(&mockito::server_url())
            .disable_csrf()
            .build()
            .unwrap();

        let _mock = mockito::mock("POST", "/error-NewException")
            .with_status(500)
            .with_body("hviqsuvnqsodjfsqjdgo java.lang.NewException: my error\nvzfjsd")
            .create();

        let response = jenkins_client.post_with_body(
            &super::Path::Raw {
                path: "/error-NewException",
            },
            "body",
            &[],
        );

        assert!(response.is_err());
        assert_eq!(
            format!("{:?}", response),
            r#"Err(reqwest::Error { kind: Status(500), url: Url { scheme: "http", host: Some(Ipv4(127.0.0.1)), port: Some(1234), path: "/error-NewException", query: None, fragment: None } })"#,
        );
    }

    #[test]
    fn can_post_with_query_params() {
        let jenkins_client = crate::JenkinsBuilder::new(&mockito::server_url())
            .disable_csrf()
            .build()
            .unwrap();

        let mock = mockito::mock("POST", "/mypath?a=1")
            .with_body("ok")
            .create();

        let response = jenkins_client.post_with_body(
            &super::Path::Raw { path: "/mypath" },
            "body",
            &[("a", "1")],
        );

        assert!(response.is_ok());
        assert_eq!(response.unwrap().text().unwrap(), "ok");
        mock.assert()
    }
}
