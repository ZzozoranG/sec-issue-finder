use serde::{Deserialize, Serialize};
use std::future::Future;
#[cfg(feature = "test-utils")]
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use crate::error::SecFinderError;
use crate::types::{Dependency, Ecosystem};

const OSV_QUERY_BATCH_URL: &str = "https://api.osv.dev/v1/querybatch";
#[cfg(feature = "test-utils")]
const OSV_MOCK_RESPONSE_FILE_ENV: &str = "SEC_ISSUE_FINDER_OSV_MOCK_RESPONSE_FILE";

#[derive(Clone)]
pub struct OsvClient {
    http: Arc<dyn OsvHttpClient>,
    query_batch_url: String,
}

impl OsvClient {
    pub fn new() -> Self {
        #[cfg(feature = "test-utils")]
        {
            // Internal integration-test hook. Production builds do not compile this
            // transport, so setting the env var cannot alter normal OSV behavior.
            if let Ok(path) = std::env::var(OSV_MOCK_RESPONSE_FILE_ENV) {
                return Self {
                    http: Arc::new(FileOsvHttpClient {
                        response_path: PathBuf::from(path),
                    }),
                    query_batch_url: OSV_QUERY_BATCH_URL.to_string(),
                };
            }
        }

        Self::with_http_client(reqwest::Client::new())
    }

    pub fn with_http_client(http: reqwest::Client) -> Self {
        Self {
            http: Arc::new(ReqwestOsvHttpClient { http }),
            query_batch_url: OSV_QUERY_BATCH_URL.to_string(),
        }
    }

    #[cfg(test)]
    fn with_transport(http: Arc<dyn OsvHttpClient>) -> Self {
        Self {
            http,
            query_batch_url: OSV_QUERY_BATCH_URL.to_string(),
        }
    }

    pub async fn query_batch(
        &self,
        dependencies: &[Dependency],
    ) -> Result<Vec<OsvQueryResult>, SecFinderError> {
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        let request_body = build_query_batch_request(dependencies);
        let response = self
            .http
            .post_json(&self.query_batch_url, &request_body)
            .await?;

        let status = response.status();
        let body = response.body;

        if !status.is_success() {
            return Err(SecFinderError::OsvStatus { status, body });
        }

        parse_query_batch_response(dependencies, &body)
    }
}

trait OsvHttpClient: Send + Sync {
    fn post_json<'a>(
        &'a self,
        url: &'a str,
        body: &'a OsvQueryBatchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OsvHttpResponse, SecFinderError>> + Send + 'a>>;
}

struct ReqwestOsvHttpClient {
    http: reqwest::Client,
}

impl OsvHttpClient for ReqwestOsvHttpClient {
    fn post_json<'a>(
        &'a self,
        url: &'a str,
        body: &'a OsvQueryBatchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OsvHttpResponse, SecFinderError>> + Send + 'a>> {
        Box::pin(async move {
            let response = self.http.post(url).json(body).send().await?;
            let status = response.status();
            let body = response.text().await?;
            Ok(OsvHttpResponse { status, body })
        })
    }
}

#[cfg(feature = "test-utils")]
struct FileOsvHttpClient {
    response_path: PathBuf,
}

#[cfg(feature = "test-utils")]
impl OsvHttpClient for FileOsvHttpClient {
    fn post_json<'a>(
        &'a self,
        _url: &'a str,
        _body: &'a OsvQueryBatchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OsvHttpResponse, SecFinderError>> + Send + 'a>> {
        Box::pin(async move {
            let body = std::fs::read_to_string(&self.response_path).map_err(|source| {
                SecFinderError::ReadOsvMockResponse {
                    path: self.response_path.clone(),
                    source,
                }
            })?;

            Ok(OsvHttpResponse {
                status: reqwest::StatusCode::OK,
                body,
            })
        })
    }
}

struct OsvHttpResponse {
    status: reqwest::StatusCode,
    body: String,
}

impl OsvHttpResponse {
    fn status(&self) -> reqwest::StatusCode {
        self.status
    }
}

impl Default for OsvClient {
    fn default() -> Self {
        Self::new()
    }
}

fn build_query_batch_request(dependencies: &[Dependency]) -> OsvQueryBatchRequest {
    OsvQueryBatchRequest {
        queries: dependencies
            .iter()
            .map(|dependency| OsvQuery {
                package: OsvPackage {
                    ecosystem: osv_ecosystem(dependency.ecosystem),
                    name: dependency.name.clone(),
                },
                version: dependency.version.clone(),
            })
            .collect(),
    }
}

fn parse_query_batch_response(
    dependencies: &[Dependency],
    body: &str,
) -> Result<Vec<OsvQueryResult>, SecFinderError> {
    let response: OsvQueryBatchResponse =
        serde_json::from_str(body).map_err(SecFinderError::OsvMalformedResponse)?;

    if response.results.len() != dependencies.len() {
        return Err(SecFinderError::OsvResponseLengthMismatch {
            expected: dependencies.len(),
            actual: response.results.len(),
        });
    }

    Ok(dependencies
        .iter()
        .cloned()
        .zip(response.results)
        .map(|(dependency, result)| OsvQueryResult {
            dependency,
            vulnerabilities: result.vulnerabilities,
        })
        .collect())
}

fn osv_ecosystem(ecosystem: Ecosystem) -> String {
    match ecosystem {
        Ecosystem::Npm => "npm".to_string(),
        Ecosystem::Dart => "Pub".to_string(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OsvQueryResult {
    pub dependency: Dependency,
    pub vulnerabilities: Vec<OsvVulnerability>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct OsvVulnerability {
    pub id: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub severity: Vec<OsvSeverity>,
    #[serde(default)]
    pub affected: Vec<OsvAffected>,
    #[serde(default)]
    pub references: Vec<OsvReference>,
    #[serde(default)]
    pub modified: Option<String>,
    #[serde(default)]
    pub published: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct OsvSeverity {
    #[serde(rename = "type")]
    pub severity_type: String,
    pub score: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct OsvAffected {
    #[serde(default)]
    pub ranges: Vec<OsvRange>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct OsvRange {
    #[serde(default)]
    pub events: Vec<OsvRangeEvent>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct OsvRangeEvent {
    #[serde(default)]
    pub fixed: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct OsvReference {
    #[serde(rename = "type")]
    pub reference_type: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
struct OsvQueryBatchRequest {
    queries: Vec<OsvQuery>,
}

#[derive(Debug, Serialize)]
struct OsvQuery {
    package: OsvPackage,
    version: String,
}

#[derive(Debug, Serialize)]
struct OsvPackage {
    ecosystem: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct OsvQueryBatchResponse {
    results: Vec<OsvResult>,
}

#[derive(Debug, Deserialize)]
struct OsvResult {
    #[serde(default, rename = "vulns")]
    vulnerabilities: Vec<OsvVulnerability>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use reqwest::StatusCode;
    use serde_json::json;

    use crate::error::SecFinderError;
    use crate::types::{Dependency, Ecosystem};

    use super::{
        build_query_batch_request, OsvClient, OsvHttpClient, OsvHttpResponse, OsvQueryBatchRequest,
    };

    #[test]
    fn builds_request_body_for_one_dependency() {
        let body = serde_json::to_value(build_query_batch_request(&[dependency(
            "lodash", "4.17.20",
        )]))
        .unwrap();

        assert_eq!(
            body,
            json!({
                "queries": [
                    {
                        "package": {
                            "ecosystem": "npm",
                            "name": "lodash"
                        },
                        "version": "4.17.20"
                    }
                ]
            })
        );
    }

    #[test]
    fn builds_request_body_for_multiple_dependencies_in_order() {
        let body = serde_json::to_value(build_query_batch_request(&[
            dependency("lodash", "4.17.20"),
            dependency("@scope/pkg", "1.0.0"),
        ]))
        .unwrap();

        assert_eq!(
            body,
            json!({
                "queries": [
                    {
                        "package": {
                            "ecosystem": "npm",
                            "name": "lodash"
                        },
                        "version": "4.17.20"
                    },
                    {
                        "package": {
                            "ecosystem": "npm",
                            "name": "@scope/pkg"
                        },
                        "version": "1.0.0"
                    }
                ]
            })
        );
    }

    #[tokio::test]
    async fn empty_dependency_list_does_not_send_request() {
        let client =
            OsvClient::with_transport(Arc::new(MockOsvHttpClient::with_responses(Vec::new())));

        let results = client.query_batch(&[]).await.unwrap();

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn successful_no_vulnerability_response() {
        let client =
            OsvClient::with_transport(Arc::new(MockOsvHttpClient::with_responses(vec![Ok(
                OsvHttpResponse {
                    status: StatusCode::OK,
                    body: r#"{"results":[{}]}"#.to_string(),
                },
            )])));
        let dependencies = vec![dependency("left-pad", "1.3.0")];

        let results = client.query_batch(&dependencies).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].dependency, dependencies[0]);
        assert!(results[0].vulnerabilities.is_empty());
    }

    #[tokio::test]
    async fn successful_vulnerability_response_preserves_dependency_order() {
        let client = OsvClient::with_transport(Arc::new(MockOsvHttpClient::with_responses(vec![Ok(
            OsvHttpResponse {
                status: StatusCode::OK,
                body: r#"{"results":[{"vulns":[{"id":"OSV-1","summary":"first"}]},{"vulns":[{"id":"OSV-2","aliases":["CVE-0000-0002"]}]}]}"#.to_string(),
            },
        )])));
        let dependencies = vec![dependency("first", "1.0.0"), dependency("second", "2.0.0")];

        let results = client.query_batch(&dependencies).await.unwrap();

        assert_eq!(results[0].dependency.name, "first");
        assert_eq!(results[0].vulnerabilities[0].id, "OSV-1");
        assert_eq!(
            results[0].vulnerabilities[0].summary.as_deref(),
            Some("first")
        );
        assert_eq!(results[1].dependency.name, "second");
        assert_eq!(results[1].vulnerabilities[0].id, "OSV-2");
        assert_eq!(results[1].vulnerabilities[0].aliases, vec!["CVE-0000-0002"]);
    }

    #[tokio::test]
    async fn non_2xx_response_returns_error() {
        let client =
            OsvClient::with_transport(Arc::new(MockOsvHttpClient::with_responses(vec![Ok(
                OsvHttpResponse {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    body: "broken".to_string(),
                },
            )])));

        let error = client
            .query_batch(&[dependency("left-pad", "1.3.0")])
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            SecFinderError::OsvStatus {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn malformed_response_returns_error() {
        let client =
            OsvClient::with_transport(Arc::new(MockOsvHttpClient::with_responses(vec![Ok(
                OsvHttpResponse {
                    status: StatusCode::OK,
                    body: "{not json".to_string(),
                },
            )])));

        let error = client
            .query_batch(&[dependency("left-pad", "1.3.0")])
            .await
            .unwrap_err();

        assert!(matches!(error, SecFinderError::OsvMalformedResponse(_)));
    }

    fn dependency(name: &str, version: &str) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: version.to_string(),
            ecosystem: Ecosystem::Npm,
            package_url: Some(format!("pkg:npm/{name}@{version}")),
            direct: true,
            dev: false,
            source_file: PathBuf::from("package-lock.json"),
        }
    }

    struct MockOsvHttpClient {
        responses: Mutex<Vec<Result<OsvHttpResponse, SecFinderError>>>,
        requests: Mutex<Vec<serde_json::Value>>,
    }

    impl MockOsvHttpClient {
        fn with_responses(responses: Vec<Result<OsvHttpResponse, SecFinderError>>) -> Self {
            Self {
                responses: Mutex::new(responses),
                requests: Mutex::new(Vec::new()),
            }
        }
    }

    impl OsvHttpClient for MockOsvHttpClient {
        fn post_json<'a>(
            &'a self,
            _url: &'a str,
            body: &'a OsvQueryBatchRequest,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<OsvHttpResponse, SecFinderError>>
                    + Send
                    + 'a,
            >,
        > {
            Box::pin(async move {
                self.requests
                    .lock()
                    .unwrap()
                    .push(serde_json::to_value(body).unwrap());
                self.responses.lock().unwrap().remove(0)
            })
        }
    }
}
