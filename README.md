# jenkins-api.rs [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Release Doc](https://docs.rs/jenkins_api/badge.svg)](https://docs.rs/jenkins_api) [![Crate](https://img.shields.io/crates/v/jenkins_api.svg)](https://crates.io/crates/jenkins_api)

Bindings to [Jenkins JSON API](https://www.jenkins.io/doc/book/using/remote-access-api/)

## Example

```rust
extern crate tokio;

use jenkins_api::JenkinsBuilder;
use jenkins_api::build::BuildStatus;
use jenkins_api::job::BuildableJob;

#[tokio::main]
fn main() {
    let jenkins = JenkinsBuilder::new("http://localhost:8080")
        .with_user("user", Some("password"))
        .build().unwrap();

    let job = jenkins.get_job("job name").await.unwrap();

    let to_build = if let Some(short_build) = job.last_build.clone() {
        let build = short_build.get_full_build(&jenkins).await.unwrap();
        println!(
            "last build for job {} at {} was {:?}",
            job.name, build.timestamp, build.result
        );
        if let Some(result) = build.result {
            result != BuildStatus::Success
        } else {
            true
        }
    } else {
        println!("job {} was never built", job.name);
        true
    };

    if to_build {
        println!("triggering a new build");
        job.as_variant::<jenkins_api::job::FreeStyleProject>().unwrap()
            .build(&jenkins).await.unwrap();
    }
}
```
