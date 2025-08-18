use reqwest::{RequestBuilder, header::HeaderName, header::HeaderValue};
use serde::Deserialize;

use super::{Jenkins, path::Path};
use crate::client::Result;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Crumb {
    crumb: String,
    crumb_request_field: String,
}

impl Jenkins {
    pub(crate) async fn add_csrf_to_request(
        &self,
        request_builder: RequestBuilder,
    ) -> Result<RequestBuilder> {
        if self.csrf_enabled {
            let crumb = self.get_csrf().await?;
            Ok(request_builder.header(
                HeaderName::from_lowercase(crumb.crumb_request_field.to_lowercase().as_bytes())?,
                HeaderValue::from_str(&crumb.crumb)?,
            ))
        } else {
            Ok(request_builder)
        }
    }

    pub(crate) async fn get_csrf(&self) -> Result<Crumb> {
        let crumb: Crumb = self.get(&Path::CrumbIssuer).await?.json().await?;
        Ok(crumb)
    }
}
