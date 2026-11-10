use std::fmt;

use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub struct HealthCheckRequest {
    #[prost(string, tag = "1")]
    pub service: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct HealthCheckResponse {
    #[prost(int32, tag = "1")]
    pub status: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServingStatus {
    Unknown,
    Serving,
    NotServing,
    ServiceUnknown,
    Unrecognized(i32),
}

impl ServingStatus {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Serving,
            2 => Self::NotServing,
            3 => Self::ServiceUnknown,
            other => Self::Unrecognized(other),
        }
    }

    pub(crate) fn is_serving(self) -> bool {
        self == Self::Serving
    }
}

impl fmt::Display for ServingStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => formatter.write_str("UNKNOWN"),
            Self::Serving => formatter.write_str("SERVING"),
            Self::NotServing => formatter.write_str("NOT_SERVING"),
            Self::ServiceUnknown => formatter.write_str("SERVICE_UNKNOWN"),
            Self::Unrecognized(value) => write!(formatter, "UNRECOGNIZED({value})"),
        }
    }
}

/// Standard `/grpc.health.v1.Health` gRPC service for embedded server liveness & readiness checks.
#[derive(Clone, Debug)]
pub struct HealthService {
    max_decoding_message_size: Option<usize>,
    max_encoding_message_size: Option<usize>,
}

impl Default for HealthService {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthService {
    pub const KNOWN_SERVICES: &'static [&'static str] = &[
        "",
        "grpc.health.v1.Health",
        "oscal.services.v1.OscalService",
        "oscal.services.v1.GovernanceService",
        "oscal.services.v1.TransparencyExchangeService",
        "oscal.services.v1.TransparencyGraphService",
    ];

    pub fn new() -> Self {
        Self {
            max_decoding_message_size: Some(crate::transport::MAX_MESSAGE_BYTES),
            max_encoding_message_size: Some(crate::transport::MAX_MESSAGE_BYTES),
        }
    }

    #[must_use]
    pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
        self.max_decoding_message_size = Some(limit);
        self
    }

    #[must_use]
    pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
        self.max_encoding_message_size = Some(limit);
        self
    }
}

impl tonic::server::NamedService for HealthService {
    const NAME: &'static str = "grpc.health.v1.Health";
}

impl<B> tonic::codegen::Service<tonic::codegen::http::Request<B>> for HealthService
where
    B: tonic::codegen::Body + Send + 'static,
    B::Error: Into<tonic::codegen::StdError> + Send + 'static,
{
    type Response = tonic::codegen::http::Response<tonic::body::Body>;
    type Error = std::convert::Infallible;
    type Future = tonic::codegen::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: tonic::codegen::http::Request<B>) -> Self::Future {
        match req.uri().path() {
            "/grpc.health.v1.Health/Check" => {
                struct CheckSvc;
                impl tonic::server::UnaryService<HealthCheckRequest> for CheckSvc {
                    type Response = HealthCheckResponse;
                    type Future =
                        tonic::codegen::BoxFuture<tonic::Response<Self::Response>, tonic::Status>;

                    fn call(
                        &mut self,
                        request: tonic::Request<HealthCheckRequest>,
                    ) -> Self::Future {
                        let service = request.into_inner().service;
                        Box::pin(async move {
                            if HealthService::KNOWN_SERVICES.contains(&service.as_str()) {
                                Ok(tonic::Response::new(HealthCheckResponse { status: 1 }))
                            } else {
                                Err(tonic::Status::not_found(format!(
                                    "unknown service: {service}"
                                )))
                            }
                        })
                    }
                }

                let max_decoding = self.max_decoding_message_size;
                let max_encoding = self.max_encoding_message_size;
                Box::pin(async move {
                    let codec = tonic_prost::ProstCodec::default();
                    let mut grpc = tonic::server::Grpc::new(codec)
                        .apply_max_message_size_config(max_decoding, max_encoding);
                    let res = grpc.unary(CheckSvc, req).await;
                    Ok(res)
                })
            }
            _ => Box::pin(async move {
                let mut response =
                    tonic::codegen::http::Response::new(tonic::body::Body::default());
                let headers = response.headers_mut();
                headers.insert(
                    tonic::Status::GRPC_STATUS,
                    (tonic::Code::Unimplemented as i32).into(),
                );
                headers.insert(
                    tonic::codegen::http::header::CONTENT_TYPE,
                    tonic::metadata::GRPC_CONTENT_TYPE,
                );
                Ok(response)
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HealthCheckRequest, ServingStatus};

    #[test]
    fn standard_health_status_values_are_mapped_without_claiming_unknown_as_healthy() {
        assert_eq!(ServingStatus::from_i32(0), ServingStatus::Unknown);
        assert_eq!(ServingStatus::from_i32(1), ServingStatus::Serving);
        assert_eq!(ServingStatus::from_i32(2), ServingStatus::NotServing);
        assert_eq!(ServingStatus::from_i32(3), ServingStatus::ServiceUnknown);
        assert_eq!(ServingStatus::from_i32(99), ServingStatus::Unrecognized(99));
        assert!(ServingStatus::Serving.is_serving());
        assert!(!ServingStatus::Unknown.is_serving());
        assert_eq!(HealthCheckRequest::default().service, "");
    }
}
