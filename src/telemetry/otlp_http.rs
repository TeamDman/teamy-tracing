//! Explicitly started, IPv4 loopback-only OTLP/HTTP protobuf listener.
//!
//! The caller owns its runtime, shared store and shutdown handle. This module
//! never logs received events or installs a subscriber.
use super::RecordStore;
use super::otlp::IngestError;
use super::otlp::ingest_protobuf;
use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::post;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsPartialSuccess;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceResponse;
use prost::Message;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// Strict request budgets; the store has independent retention limits.
#[derive(Clone, Copy, Debug)]
pub struct ReceiverLimits {
    pub max_request_bytes: usize,
    pub max_in_flight: usize,
    pub body_timeout: Duration,
}

impl Default for ReceiverLimits {
    fn default() -> Self {
        Self {
            max_request_bytes: 1024 * 1024,
            max_in_flight: 8,
            body_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Clone, Debug)]
struct ReceiverState {
    store: Arc<Mutex<RecordStore>>,
    semaphore: Arc<Semaphore>,
    max_request_bytes: usize,
    body_timeout: Duration,
}

/// Bound listener. Call `shutdown` to request graceful shutdown and join it.
#[derive(Debug)]
pub struct Receiver {
    address: SocketAddr,
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<std::io::Result<()>>,
}

impl Receiver {
    /// Bind only `127.0.0.1`; port zero asks the OS for an available port.
    pub async fn bind_loopback(
        port: u16,
        limits: ReceiverLimits,
        store: Arc<Mutex<RecordStore>>,
    ) -> std::io::Result<Self> {
        if limits.max_request_bytes == 0
            || limits.max_in_flight == 0
            || limits.max_in_flight > Semaphore::MAX_PERMITS
            || limits.body_timeout.is_zero()
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "receiver limits must be nonzero",
            ));
        }
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await?;
        let address = listener.local_addr()?;
        let state = ReceiverState {
            store,
            semaphore: Arc::new(Semaphore::new(limits.max_in_flight)),
            max_request_bytes: limits.max_request_bytes,
            body_timeout: limits.body_timeout,
        };
        let router = Router::new()
            .route("/v1/logs", post(receive))
            .with_state(state);
        let (shutdown, signal) = oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = signal.await;
                })
                .await
        });
        Ok(Self {
            address,
            shutdown,
            task,
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Stops accepting connections and waits for in-flight work.
    pub async fn shutdown(self) -> std::io::Result<()> {
        let _ = self.shutdown.send(());
        self.task.await.map_err(std::io::Error::other)?
    }
}

async fn receive(State(state): State<ReceiverState>, request: Request<Body>) -> Response {
    let Ok(_permit) = state.semaphore.try_acquire() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let is_protobuf = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(';')
                .next()
                .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/x-protobuf"))
        });
    if !is_protobuf {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }
    let bytes = match tokio::time::timeout(
        state.body_timeout,
        to_bytes(request.into_body(), state.max_request_bytes),
    )
    .await
    {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(_)) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
        Err(_) => return StatusCode::REQUEST_TIMEOUT.into_response(),
    };
    let Ok(mut store) = state.store.lock() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let result = match ingest_protobuf(&bytes, state.max_request_bytes, &mut store) {
        Ok(result) => result,
        Err(IngestError::InvalidProtobuf) => return StatusCode::BAD_REQUEST.into_response(),
        Err(IngestError::RequestTooLarge) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let partial_success = (result.rejected != 0).then(|| ExportLogsPartialSuccess {
        rejected_log_records: i64::try_from(result.rejected).unwrap_or(i64::MAX),
        error_message: "records exceeded store limits".into(),
    });
    let body = ExportLogsServiceResponse { partial_success }.encode_to_vec();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-protobuf")],
        body,
    )
        .into_response()
}
