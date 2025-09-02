# Changes

# 0.9.0 (2025/09/02)

* BREAKING: Switched to `reqwest::Client` over `reqwest::blocking::Client`. `jenkins_api` is now fully `async`.
* Added `Send + Sync` bounds to `Error` type
* Updated to Rust 2024
* Added `User-Agent` to request header
* Added a timeout to `JenkinsBuilder`
* Added method to query artifacts as `bytes::Bytes`

# 0.8.0 (2020/05/23)

* Jenkins agent port changed from an u32 to an enum AgentPort #50
* New method to get config.xml from a job #55
* Added feature extra-fields-visibility that allow public access to fields that were not parsed as part of a specific object type #52
* Folder management #51
* Multibranch pipeline project management #36

# 0.7.0 (2019/11/17)

* Removed dependency to failure

# 0.6.0 (2019/09/17)

* Update dependencies
* Changed some fields of jobs to be optional (`full_name`, `full_display_name`, `color`)
* Changed some fields of builds to be optional (`full_display_name`) and added some fields (`display_name`, `timestamp`)

# 0.5.2 (2018/11/10)

* Support assigned labels field on computers

# 0.5.1 (2018/10/10)

* Updated dependencies
* Should now work with all configurations for crumb header in Jenkins

# 0.5.0 (2018/07/02)

* Updated variants types to be able to keep variant information when navigating between objects (ie `CommonJob` -> `FreeStyleProject` -> `ShortBuild` -> `FreeStyleBuild` without going through `CommonBuild`)
* Updated visibility of some of the structs in `client`
* Added a new method `get_object_as` that let the user decide the amount of data returned. See [taming-jenkins-json-api-depth-and-tree](https://www.cloudbees.com/blog/taming-jenkins-json-api-depth-and-tree)
* Removed deprecated methods

# 0.4.2 (2018/06/19)

* Decrease log level
* Deprecated most functions of traits `Job` and `Build`
* Add fields on TimeInQueueAction
* Support MultiJobProject and MultiJobBuild

# 0.4.1 (2018/06/13)

* Can get nodes linked to Jenkins
* Support build flow jobs

# 0.4.0 (2018/05/24)

* Change all data structures to extendable trait / struct instead of enum
* Can change depth in requests when building Jenkins client
* All short items derive Serialize
* Can target build by alias

# 0.3.1 (2018/05/21)

* Get artifacts of a build
* Support external jobs
* Support maven projects
* Feature to toggle between permissive/strict json parsing for Jenkins responses

# 0.3.0 (2018/05/13)

* Default enum variant renamed to Unknown
* Changed `Error::InvalidUrl` `expected` field to an Enum (`error::ExpectedType`)
* A `Build` can have many variants, for now either a free style or a pipeline
* Adding `Action` and change set variants
* Support pipeline `Job`
* Support more types of `View`
* Support matrix projects

# 0.2.2 (2018/05/10)

* Can deserialize git informations from a build
* Can trigger job remotely (GET request with a token)
* Can poll configured SCM of a project
* Can build job with parameters
* Can deserialize actions from a queue item
* Logging request and error responses

# 0.2.1 (2018/05/04)

* Can deserialize actions from a build

# 0.2.0 (2018/04/25)

* Fix case for error messages
* Better Queue management
* Can trigger job without parameters
* Can get console text from a build
