use std::fmt;

use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub(crate) struct HealthCheckRequest {
    #[prost(string, tag = "1")]
    pub service: String,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct HealthCheckResponse {
    #[prost(int32, tag = "1")]
    pub status: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ServingStatus {
    Unknown,
    Serving,
    NotServing,
    ServiceUnknown,
    Unrecognized(i32),
}

impl ServingStatus {
    pub(crate) fn from_i32(value: i32) -> Self {
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
