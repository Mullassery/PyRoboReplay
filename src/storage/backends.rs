use crate::storage::backend::{StorageBackend, StorageError, StorageResult, StorageStats};
use std::sync::Mutex;
use tokio::runtime::Runtime;

// ============================================================================
// PostgreSQL backend
//
// Real implementation using `tokio-postgres`. The `StorageBackend` trait is
// entirely synchronous (no async runtime is threaded through the rest of the
// codebase), so this backend owns a small dedicated multi-threaded Tokio
// runtime and drives every async call through `Runtime::block_on`. The
// runtime and the live `Client` are created in `connect()` and stored behind
// a `Mutex` (matching the pattern already used by `SqliteBackend`) so that
// `close()` can genuinely drop the connection even though it only takes
// `&self`.
// ============================================================================

pub struct PostgresBackend {
    connection_string: String,
    runtime: Mutex<Option<Runtime>>,
    client: Mutex<Option<tokio_postgres::Client>>,
}

const POSTGRES_SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS missions (
        mission_id VARCHAR(255) PRIMARY KEY,
        data       TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
    CREATE TABLE IF NOT EXISTS events (
        mission_id VARCHAR(255) NOT NULL,
        event_id   VARCHAR(255) NOT NULL,
        data       TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        PRIMARY KEY (mission_id, event_id),
        FOREIGN KEY (mission_id) REFERENCES missions(mission_id) ON DELETE CASCADE
    );
    CREATE TABLE IF NOT EXISTS reports (
        mission_id VARCHAR(255) PRIMARY KEY,
        report     TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        FOREIGN KEY (mission_id) REFERENCES missions(mission_id) ON DELETE CASCADE
    );
";

impl PostgresBackend {
    pub fn new(connection_string: &str) -> Self {
        PostgresBackend {
            connection_string: connection_string.to_string(),
            runtime: Mutex::new(None),
            client: Mutex::new(None),
        }
    }

    fn not_connected() -> StorageError {
        StorageError::ConnectionFailed(
            "PostgreSQL backend not connected; call connect() first".to_string(),
        )
    }

    /// Run an async closure against the live client on this backend's
    /// dedicated runtime. Returns an error if `connect()` hasn't succeeded.
    fn with_client<F, T>(&self, f: F) -> StorageResult<T>
    where
        F: for<'c> FnOnce(
            &'c tokio_postgres::Client,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, tokio_postgres::Error>> + Send + 'c>,
        >,
    {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        runtime
            .block_on(f(client))
            .map_err(|e| StorageError::BackendError(e.to_string()))
    }
}

impl StorageBackend for PostgresBackend {
    fn connect(&mut self) -> StorageResult<()> {
        let runtime = Runtime::new().map_err(|e| {
            StorageError::ConnectionFailed(format!("failed to start async runtime: {}", e))
        })?;

        let connection_string = self.connection_string.clone();
        let client = runtime.block_on(async move {
            let (client, connection) =
                tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await?;

            // The connection object performs the actual IO; it must be
            // polled continuously for the client to make progress. Spawn it
            // as a background task on our dedicated runtime for the
            // lifetime of this backend.
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    tracing::error!("PostgreSQL connection error: {}", e);
                }
            });

            client.batch_execute(POSTGRES_SCHEMA).await?;

            Ok::<_, tokio_postgres::Error>(client)
        });

        let client = client.map_err(|e| {
            StorageError::ConnectionFailed(format!("failed to connect to PostgreSQL: {}", e))
        })?;

        *self.client.lock().unwrap() = Some(client);
        *self.runtime.lock().unwrap() = Some(runtime);
        Ok(())
    }

    fn store_mission(&self, mission_id: &str, data: &str) -> StorageResult<()> {
        let mission_id = mission_id.to_string();
        let data = data.to_string();
        self.with_client(move |client| {
            Box::pin(async move {
                client
                    .execute(
                        "INSERT INTO missions (mission_id, data) VALUES ($1, $2) \
                         ON CONFLICT (mission_id) DO UPDATE SET data = EXCLUDED.data",
                        &[&mission_id, &data],
                    )
                    .await
                    .map(|_| ())
            })
        })
    }

    fn retrieve_mission(&self, mission_id: &str) -> StorageResult<String> {
        let mission_id_owned = mission_id.to_string();
        let row = self.with_client(move |client| {
            Box::pin(async move {
                client
                    .query_opt(
                        "SELECT data FROM missions WHERE mission_id = $1",
                        &[&mission_id_owned],
                    )
                    .await
            })
        })?;
        row.map(|r| r.get::<_, String>(0))
            .ok_or_else(|| StorageError::NotFound(format!("mission '{}' not found", mission_id)))
    }

    fn store_event(&self, mission_id: &str, event_id: &str, data: &str) -> StorageResult<()> {
        let mission_id = mission_id.to_string();
        let event_id = event_id.to_string();
        let data = data.to_string();
        self.with_client(move |client| {
            Box::pin(async move {
                client
                    .execute(
                        "INSERT INTO events (mission_id, event_id, data) VALUES ($1, $2, $3) \
                         ON CONFLICT (mission_id, event_id) DO UPDATE SET data = EXCLUDED.data",
                        &[&mission_id, &event_id, &data],
                    )
                    .await
                    .map(|_| ())
            })
        })
    }

    fn retrieve_event(&self, mission_id: &str, event_id: &str) -> StorageResult<String> {
        let mission_id_owned = mission_id.to_string();
        let event_id_owned = event_id.to_string();
        let row = self.with_client(move |client| {
            Box::pin(async move {
                client
                    .query_opt(
                        "SELECT data FROM events WHERE mission_id = $1 AND event_id = $2",
                        &[&mission_id_owned, &event_id_owned],
                    )
                    .await
            })
        })?;
        row.map(|r| r.get::<_, String>(0)).ok_or_else(|| {
            StorageError::NotFound(format!(
                "event '{}' not found for mission '{}'",
                event_id, mission_id
            ))
        })
    }

    fn store_report(&self, mission_id: &str, report: &str) -> StorageResult<()> {
        let mission_id = mission_id.to_string();
        let report = report.to_string();
        self.with_client(move |client| {
            Box::pin(async move {
                client
                    .execute(
                        "INSERT INTO reports (mission_id, report) VALUES ($1, $2) \
                         ON CONFLICT (mission_id) DO UPDATE SET report = EXCLUDED.report",
                        &[&mission_id, &report],
                    )
                    .await
                    .map(|_| ())
            })
        })
    }

    fn retrieve_report(&self, mission_id: &str) -> StorageResult<String> {
        let mission_id_owned = mission_id.to_string();
        let row = self.with_client(move |client| {
            Box::pin(async move {
                client
                    .query_opt(
                        "SELECT report FROM reports WHERE mission_id = $1",
                        &[&mission_id_owned],
                    )
                    .await
            })
        })?;
        row.map(|r| r.get::<_, String>(0))
            .ok_or_else(|| StorageError::NotFound(format!("report for mission '{}' not found", mission_id)))
    }

    fn list_missions(&self, limit: Option<usize>) -> StorageResult<Vec<String>> {
        let rows = self.with_client(move |client| {
            Box::pin(async move {
                match limit {
                    Some(limit) => {
                        client
                            .query(
                                "SELECT mission_id FROM missions ORDER BY created_at DESC LIMIT $1",
                                &[&(limit as i64)],
                            )
                            .await
                    }
                    None => {
                        client
                            .query("SELECT mission_id FROM missions ORDER BY created_at DESC", &[])
                            .await
                    }
                }
            })
        })?;
        Ok(rows.iter().map(|r| r.get::<_, String>(0)).collect())
    }

    fn delete_mission(&self, mission_id: &str) -> StorageResult<()> {
        let mission_id_owned = mission_id.to_string();
        let affected = self.with_client(move |client| {
            Box::pin(async move {
                client
                    .execute("DELETE FROM missions WHERE mission_id = $1", &[&mission_id_owned])
                    .await
            })
        })?;
        if affected == 0 {
            return Err(StorageError::NotFound(format!("mission '{}' not found", mission_id)));
        }
        Ok(())
    }

    fn mission_exists(&self, mission_id: &str) -> StorageResult<bool> {
        let mission_id_owned = mission_id.to_string();
        let row = self.with_client(move |client| {
            Box::pin(async move {
                client
                    .query_opt("SELECT 1 FROM missions WHERE mission_id = $1", &[&mission_id_owned])
                    .await
            })
        })?;
        Ok(row.is_some())
    }

    fn get_stats(&self) -> StorageResult<StorageStats> {
        let (missions, events, reports) = self.with_client(move |client| {
            Box::pin(async move {
                let missions: i64 = client.query_one("SELECT COUNT(*) FROM missions", &[]).await?.get(0);
                let events: i64 = client.query_one("SELECT COUNT(*) FROM events", &[]).await?.get(0);
                let reports: i64 = client.query_one("SELECT COUNT(*) FROM reports", &[]).await?.get(0);
                Ok((missions, events, reports))
            })
        })?;
        Ok(StorageStats {
            total_missions: missions as u64,
            total_events: events as u64,
            total_reports: reports as u64,
            storage_size_bytes: None,
            connected: true,
        })
    }

    fn close(&self) -> StorageResult<()> {
        let client = self.client.lock().unwrap().take();
        if client.is_none() {
            return Err(Self::not_connected());
        }
        // Dropping the client (and, next, the runtime) tears down the
        // background connection task and closes the socket.
        drop(client);
        self.runtime.lock().unwrap().take();
        Ok(())
    }
}

// ============================================================================
// BigQuery backend
//
// Left intentionally stubbed. Unlike PostgreSQL (available locally via
// Docker) and S3 (available locally via MinIO), there is no realistic local
// BigQuery emulator in this environment, and real BigQuery requires GCP
// project credentials this environment does not have. Implementing this for
// real would mean shipping untested code against a live cloud API, which is
// worse than an honest stub. The error message below says so explicitly
// instead of implying feature parity with the other two backends.
// ============================================================================

pub struct BigQueryBackend {
    #[allow(dead_code)]
    connection_string: String,
}

impl BigQueryBackend {
    pub fn new(connection_string: &str) -> Self {
        BigQueryBackend {
            connection_string: connection_string.to_string(),
        }
    }
}

const BIGQUERY_NOT_IMPLEMENTED: &str =
    "BigQuery backend is not implemented. Unlike the PostgreSQL and S3 backends, there is no \
     practical local BigQuery emulator, and real BigQuery access requires Google Cloud Platform \
     project credentials (a service account key or workload identity) that are not available in \
     every deployment environment. This backend was deliberately left unimplemented rather than \
     shipping untested code against a live cloud API with real billing implications. To \
     implement it: add `google-cloud-bigquery` (or call the BigQuery REST API directly), \
     authenticate via `GOOGLE_APPLICATION_CREDENTIALS`, and mirror the schema used by \
     PostgresBackend (missions/events/reports tables, e.g. via a partitioned table keyed on \
     mission_id).";

impl StorageBackend for BigQueryBackend {
    fn connect(&mut self) -> StorageResult<()> {
        Err(StorageError::ConnectionFailed(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn store_mission(&self, _mission_id: &str, _data: &str) -> StorageResult<()> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn retrieve_mission(&self, _mission_id: &str) -> StorageResult<String> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn store_event(&self, _mission_id: &str, _event_id: &str, _data: &str) -> StorageResult<()> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn retrieve_event(&self, _mission_id: &str, _event_id: &str) -> StorageResult<String> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn store_report(&self, _mission_id: &str, _report: &str) -> StorageResult<()> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn retrieve_report(&self, _mission_id: &str) -> StorageResult<String> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn list_missions(&self, _limit: Option<usize>) -> StorageResult<Vec<String>> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn delete_mission(&self, _mission_id: &str) -> StorageResult<()> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn mission_exists(&self, _mission_id: &str) -> StorageResult<bool> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn get_stats(&self) -> StorageResult<StorageStats> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }

    fn close(&self) -> StorageResult<()> {
        Err(StorageError::BackendError(BIGQUERY_NOT_IMPLEMENTED.to_string()))
    }
}

// ============================================================================
// S3 backend
//
// Real implementation using `aws-sdk-s3`. Like PostgresBackend, it owns a
// dedicated Tokio runtime and drives the async AWS SDK calls synchronously
// via `block_on`, since the `StorageBackend` trait is sync.
//
// Connection string format: `s3://bucket[/key-prefix][?endpoint=URL][&path_style=true]`
//   - `endpoint` overrides the S3 endpoint (used to point at a local
//     S3-compatible service such as MinIO instead of real AWS S3).
//   - `path_style=true` forces path-style addressing (required by MinIO and
//     most self-hosted S3-compatible services).
// Credentials and region are resolved the standard AWS way (environment
// variables `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`/`AWS_REGION`, shared
// config/credentials files, etc.) via `aws-config`.
//
// Object layout under the (optional) key prefix:
//   {prefix}/missions/{mission_id}/mission.json
//   {prefix}/missions/{mission_id}/events/{event_id}.json
//   {prefix}/missions/{mission_id}/report.json
// Grouping everything for a mission under one key prefix makes
// `delete_mission` a single list+bulk-delete instead of requiring a
// secondary index.
// ============================================================================

#[derive(Debug, Clone)]
struct S3Target {
    bucket: String,
    prefix: String, // no leading/trailing slash; may be empty
    endpoint_url: Option<String>,
    force_path_style: bool,
}

fn parse_s3_connection_string(s: &str) -> StorageResult<S3Target> {
    let rest = s.strip_prefix("s3://").ok_or_else(|| {
        StorageError::ConnectionFailed(format!(
            "invalid S3 connection string (expected s3://bucket[/prefix][?endpoint=URL]): {}",
            s
        ))
    })?;

    let (path_part, query_part) = match rest.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (rest, None),
    };

    let mut parts = path_part.splitn(2, '/');
    let bucket = parts.next().unwrap_or("").to_string();
    if bucket.is_empty() {
        return Err(StorageError::ConnectionFailed(format!(
            "invalid S3 connection string, missing bucket name: {}",
            s
        )));
    }
    let prefix = parts
        .next()
        .unwrap_or("")
        .trim_matches('/')
        .to_string();

    let mut endpoint_url = None;
    let mut force_path_style = false;
    if let Some(query) = query_part {
        for kv in query.split('&') {
            if kv.is_empty() {
                continue;
            }
            match kv.split_once('=') {
                Some(("endpoint", v)) => endpoint_url = Some(v.to_string()),
                Some(("path_style", v)) | Some(("force_path_style", v)) => {
                    force_path_style = v == "true" || v == "1";
                }
                _ => {}
            }
        }
    }
    // Local/self-hosted S3-compatible endpoints (MinIO, etc.) virtually
    // always require path-style addressing since they don't support
    // per-bucket DNS subdomains.
    if endpoint_url.is_some() {
        force_path_style = true;
    }

    Ok(S3Target {
        bucket,
        prefix,
        endpoint_url,
        force_path_style,
    })
}

impl S3Target {
    fn mission_key(&self, mission_id: &str) -> String {
        if self.prefix.is_empty() {
            format!("missions/{}/mission.json", mission_id)
        } else {
            format!("{}/missions/{}/mission.json", self.prefix, mission_id)
        }
    }

    fn event_key(&self, mission_id: &str, event_id: &str) -> String {
        if self.prefix.is_empty() {
            format!("missions/{}/events/{}.json", mission_id, event_id)
        } else {
            format!("{}/missions/{}/events/{}.json", self.prefix, mission_id, event_id)
        }
    }

    fn report_key(&self, mission_id: &str) -> String {
        if self.prefix.is_empty() {
            format!("missions/{}/report.json", mission_id)
        } else {
            format!("{}/missions/{}/report.json", self.prefix, mission_id)
        }
    }

    fn missions_prefix(&self) -> String {
        if self.prefix.is_empty() {
            "missions/".to_string()
        } else {
            format!("{}/missions/", self.prefix)
        }
    }

    fn mission_object_prefix(&self, mission_id: &str) -> String {
        if self.prefix.is_empty() {
            format!("missions/{}/", mission_id)
        } else {
            format!("{}/missions/{}/", self.prefix, mission_id)
        }
    }
}

pub struct S3Backend {
    connection_string: String,
    runtime: Mutex<Option<Runtime>>,
    client: Mutex<Option<aws_sdk_s3::Client>>,
    target: Mutex<Option<S3Target>>,
}

impl S3Backend {
    pub fn new(connection_string: &str) -> Self {
        S3Backend {
            connection_string: connection_string.to_string(),
            runtime: Mutex::new(None),
            client: Mutex::new(None),
            target: Mutex::new(None),
        }
    }

    fn not_connected() -> StorageError {
        StorageError::ConnectionFailed("S3 backend not connected; call connect() first".to_string())
    }
}

impl StorageBackend for S3Backend {
    fn connect(&mut self) -> StorageResult<()> {
        let target = parse_s3_connection_string(&self.connection_string)?;

        let runtime = Runtime::new().map_err(|e| {
            StorageError::ConnectionFailed(format!("failed to start async runtime: {}", e))
        })?;

        let target_for_setup = target.clone();
        let client = runtime.block_on(async move {
            let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
            if let Some(endpoint) = &target_for_setup.endpoint_url {
                loader = loader.endpoint_url(endpoint.clone());
            }
            let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
            loader = loader.region(aws_sdk_s3::config::Region::new(region));

            let shared_config = loader.load().await;
            let mut s3_builder = aws_sdk_s3::config::Builder::from(&shared_config);
            if target_for_setup.force_path_style {
                s3_builder = s3_builder.force_path_style(true);
            }
            aws_sdk_s3::Client::from_conf(s3_builder.build())
        });

        // Verify the bucket is reachable, creating it if it doesn't exist
        // yet (convenient for fresh local/test buckets; a real AWS deployment
        // would normally pre-provision the bucket, but this keeps
        // `connect()` self-sufficient either way).
        let bucket = target.bucket.clone();
        let head_result = runtime.block_on(client.head_bucket().bucket(&bucket).send());
        if head_result.is_err() {
            let _ = runtime.block_on(client.create_bucket().bucket(&bucket).send());
            runtime
                .block_on(client.head_bucket().bucket(&bucket).send())
                .map_err(|e| {
                    StorageError::ConnectionFailed(format!(
                        "bucket '{}' not accessible: {}",
                        bucket, e
                    ))
                })?;
        }

        *self.client.lock().unwrap() = Some(client);
        *self.target.lock().unwrap() = Some(target);
        *self.runtime.lock().unwrap() = Some(runtime);
        Ok(())
    }

    fn store_mission(&self, mission_id: &str, data: &str) -> StorageResult<()> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let key = target.mission_key(mission_id);
        runtime
            .block_on(
                client
                    .put_object()
                    .bucket(&target.bucket)
                    .key(&key)
                    .body(aws_sdk_s3::primitives::ByteStream::from(data.as_bytes().to_vec()))
                    .content_type("application/json")
                    .send(),
            )
            .map_err(|e| StorageError::WriteFailed(e.to_string()))?;
        Ok(())
    }

    fn retrieve_mission(&self, mission_id: &str) -> StorageResult<String> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let key = target.mission_key(mission_id);
        let result = runtime.block_on(client.get_object().bucket(&target.bucket).key(&key).send());
        let output = match result {
            Ok(output) => output,
            Err(e) => {
                if is_not_found(&e) {
                    return Err(StorageError::NotFound(format!("mission '{}' not found", mission_id)));
                }
                return Err(StorageError::ReadFailed(e.to_string()));
            }
        };
        let bytes = runtime
            .block_on(output.body.collect())
            .map_err(|e| StorageError::ReadFailed(e.to_string()))?
            .into_bytes();
        String::from_utf8(bytes.to_vec()).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn store_event(&self, mission_id: &str, event_id: &str, data: &str) -> StorageResult<()> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let key = target.event_key(mission_id, event_id);
        runtime
            .block_on(
                client
                    .put_object()
                    .bucket(&target.bucket)
                    .key(&key)
                    .body(aws_sdk_s3::primitives::ByteStream::from(data.as_bytes().to_vec()))
                    .content_type("application/json")
                    .send(),
            )
            .map_err(|e| StorageError::WriteFailed(e.to_string()))?;
        Ok(())
    }

    fn retrieve_event(&self, mission_id: &str, event_id: &str) -> StorageResult<String> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let key = target.event_key(mission_id, event_id);
        let result = runtime.block_on(client.get_object().bucket(&target.bucket).key(&key).send());
        let output = match result {
            Ok(output) => output,
            Err(e) => {
                if is_not_found(&e) {
                    return Err(StorageError::NotFound(format!(
                        "event '{}' not found for mission '{}'",
                        event_id, mission_id
                    )));
                }
                return Err(StorageError::ReadFailed(e.to_string()));
            }
        };
        let bytes = runtime
            .block_on(output.body.collect())
            .map_err(|e| StorageError::ReadFailed(e.to_string()))?
            .into_bytes();
        String::from_utf8(bytes.to_vec()).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn store_report(&self, mission_id: &str, report: &str) -> StorageResult<()> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let key = target.report_key(mission_id);
        runtime
            .block_on(
                client
                    .put_object()
                    .bucket(&target.bucket)
                    .key(&key)
                    .body(aws_sdk_s3::primitives::ByteStream::from(report.as_bytes().to_vec()))
                    .content_type("application/json")
                    .send(),
            )
            .map_err(|e| StorageError::WriteFailed(e.to_string()))?;
        Ok(())
    }

    fn retrieve_report(&self, mission_id: &str) -> StorageResult<String> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let key = target.report_key(mission_id);
        let result = runtime.block_on(client.get_object().bucket(&target.bucket).key(&key).send());
        let output = match result {
            Ok(output) => output,
            Err(e) => {
                if is_not_found(&e) {
                    return Err(StorageError::NotFound(format!(
                        "report for mission '{}' not found",
                        mission_id
                    )));
                }
                return Err(StorageError::ReadFailed(e.to_string()));
            }
        };
        let bytes = runtime
            .block_on(output.body.collect())
            .map_err(|e| StorageError::ReadFailed(e.to_string()))?
            .into_bytes();
        String::from_utf8(bytes.to_vec()).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn list_missions(&self, limit: Option<usize>) -> StorageResult<Vec<String>> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let missions_prefix = target.missions_prefix();
        let mut mission_ids = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = client
                .list_objects_v2()
                .bucket(&target.bucket)
                .prefix(&missions_prefix)
                .delimiter("/");
            if let Some(token) = &continuation_token {
                req = req.continuation_token(token);
            }
            let output = runtime
                .block_on(req.send())
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

            for common_prefix in output.common_prefixes() {
                if let Some(p) = common_prefix.prefix() {
                    // p looks like "{missions_prefix}{mission_id}/"
                    if let Some(rest) = p.strip_prefix(&missions_prefix) {
                        let mission_id = rest.trim_end_matches('/');
                        if !mission_id.is_empty() {
                            mission_ids.push(mission_id.to_string());
                        }
                    }
                }
            }

            if let Some(limit) = limit {
                if mission_ids.len() >= limit {
                    mission_ids.truncate(limit);
                    break;
                }
            }

            if output.is_truncated().unwrap_or(false) {
                continuation_token = output.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(mission_ids)
    }

    fn delete_mission(&self, mission_id: &str) -> StorageResult<()> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let object_prefix = target.mission_object_prefix(mission_id);

        // List every object under this mission's prefix (mission.json,
        // events/*.json, report.json) and batch-delete them, since S3 has no
        // native "delete by prefix" or foreign-key cascade.
        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;
        loop {
            let mut req = client
                .list_objects_v2()
                .bucket(&target.bucket)
                .prefix(&object_prefix);
            if let Some(token) = &continuation_token {
                req = req.continuation_token(token);
            }
            let output = runtime
                .block_on(req.send())
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            for obj in output.contents() {
                if let Some(k) = obj.key() {
                    keys.push(k.to_string());
                }
            }
            if output.is_truncated().unwrap_or(false) {
                continuation_token = output.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        if keys.is_empty() {
            return Err(StorageError::NotFound(format!("mission '{}' not found", mission_id)));
        }

        for chunk in keys.chunks(1000) {
            // S3 DeleteObjects supports up to 1000 keys per request.
            let object_ids: Result<Vec<_>, _> = chunk
                .iter()
                .map(|k| {
                    aws_sdk_s3::types::ObjectIdentifier::builder()
                        .key(k.clone())
                        .build()
                })
                .collect();
            let object_ids = object_ids.map_err(|e| StorageError::WriteFailed(e.to_string()))?;
            let delete = aws_sdk_s3::types::Delete::builder()
                .set_objects(Some(object_ids))
                .build()
                .map_err(|e| StorageError::WriteFailed(e.to_string()))?;
            runtime
                .block_on(
                    client
                        .delete_objects()
                        .bucket(&target.bucket)
                        .delete(delete)
                        .send(),
                )
                .map_err(|e| StorageError::WriteFailed(e.to_string()))?;
        }

        Ok(())
    }

    fn mission_exists(&self, mission_id: &str) -> StorageResult<bool> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let key = target.mission_key(mission_id);
        let result = runtime.block_on(client.head_object().bucket(&target.bucket).key(&key).send());
        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                if is_not_found(&e) {
                    Ok(false)
                } else {
                    Err(StorageError::ReadFailed(e.to_string()))
                }
            }
        }
    }

    fn get_stats(&self) -> StorageResult<StorageStats> {
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().ok_or_else(Self::not_connected)?;
        let client_guard = self.client.lock().unwrap();
        let client = client_guard.as_ref().ok_or_else(Self::not_connected)?;
        let target_guard = self.target.lock().unwrap();
        let target = target_guard.as_ref().ok_or_else(Self::not_connected)?;

        let missions_prefix = target.missions_prefix();
        let (mut missions, mut events, mut reports, mut total_bytes) = (0u64, 0u64, 0u64, 0u64);
        let mut continuation_token: Option<String> = None;
        loop {
            let mut req = client
                .list_objects_v2()
                .bucket(&target.bucket)
                .prefix(&missions_prefix);
            if let Some(token) = &continuation_token {
                req = req.continuation_token(token);
            }
            let output = runtime
                .block_on(req.send())
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            for obj in output.contents() {
                let key = obj.key().unwrap_or("");
                total_bytes += obj.size().unwrap_or(0).max(0) as u64;
                if key.ends_with("/mission.json") {
                    missions += 1;
                } else if key.ends_with("/report.json") {
                    reports += 1;
                } else if key.contains("/events/") {
                    events += 1;
                }
            }
            if output.is_truncated().unwrap_or(false) {
                continuation_token = output.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(StorageStats {
            total_missions: missions,
            total_events: events,
            total_reports: reports,
            storage_size_bytes: Some(total_bytes),
            connected: true,
        })
    }

    fn close(&self) -> StorageResult<()> {
        let client = self.client.lock().unwrap().take();
        if client.is_none() {
            return Err(Self::not_connected());
        }
        drop(client);
        self.target.lock().unwrap().take();
        self.runtime.lock().unwrap().take();
        Ok(())
    }
}

/// Best-effort classification of an AWS SDK error as "object/bucket not
/// found" (S3 returns this as a service error with a 404 status / specific
/// error code depending on the operation), so callers can map it onto
/// `StorageError::NotFound` instead of a generic read failure.
fn is_not_found<E, R>(err: &aws_sdk_s3::error::SdkError<E, R>) -> bool
where
    E: std::error::Error + 'static,
{
    if let Some(service_err) = err.as_service_error() {
        let msg = service_err.to_string().to_lowercase();
        if msg.contains("nosuchkey") || msg.contains("notfound") || msg.contains("not found") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_stub_reports_not_connected_before_connect() {
        let backend = PostgresBackend::new("postgresql://localhost/pyroboreplay");
        match backend.store_mission("m1", "{}") {
            Err(StorageError::ConnectionFailed(msg)) => {
                assert!(msg.contains("not connected"));
            }
            other => panic!("Expected ConnectionFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_bigquery_stub_returns_error() {
        let mut backend = BigQueryBackend::new("bigquery://project/dataset");
        match backend.connect() {
            Err(StorageError::ConnectionFailed(msg)) => {
                assert!(msg.contains("not implemented"));
                assert!(msg.contains("credentials"));
            }
            _ => panic!("Expected ConnectionFailed"),
        }
    }

    #[test]
    fn test_s3_stub_reports_not_connected_before_connect() {
        let backend = S3Backend::new("s3://bucket/path");
        match backend.store_mission("m1", "{}") {
            Err(StorageError::ConnectionFailed(msg)) => {
                assert!(msg.contains("not connected"));
            }
            other => panic!("Expected ConnectionFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_s3_connection_string_basic() {
        let target = parse_s3_connection_string("s3://my-bucket").unwrap();
        assert_eq!(target.bucket, "my-bucket");
        assert_eq!(target.prefix, "");
        assert!(target.endpoint_url.is_none());
        assert!(!target.force_path_style);
    }

    #[test]
    fn test_parse_s3_connection_string_with_prefix_and_endpoint() {
        let target =
            parse_s3_connection_string("s3://my-bucket/some/prefix?endpoint=http://localhost:9000")
                .unwrap();
        assert_eq!(target.bucket, "my-bucket");
        assert_eq!(target.prefix, "some/prefix");
        assert_eq!(target.endpoint_url.as_deref(), Some("http://localhost:9000"));
        // endpoint override implies path-style addressing.
        assert!(target.force_path_style);
    }

    #[test]
    fn test_parse_s3_connection_string_rejects_missing_bucket() {
        assert!(parse_s3_connection_string("s3://").is_err());
        assert!(parse_s3_connection_string("not-s3://bucket").is_err());
    }

    #[test]
    fn test_s3_target_key_layout() {
        let target = S3Target {
            bucket: "b".to_string(),
            prefix: "env".to_string(),
            endpoint_url: None,
            force_path_style: false,
        };
        assert_eq!(target.mission_key("m1"), "env/missions/m1/mission.json");
        assert_eq!(target.event_key("m1", "e1"), "env/missions/m1/events/e1.json");
        assert_eq!(target.report_key("m1"), "env/missions/m1/report.json");
        assert_eq!(target.missions_prefix(), "env/missions/");
        assert_eq!(target.mission_object_prefix("m1"), "env/missions/m1/");
    }
}
