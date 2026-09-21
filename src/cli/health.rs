#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) async fn run_health(config: &AppConfig, args: HealthArgs) -> Result<()> {
    let mut client = ReadOnlyClient::connect(config).await?;
    let request = HealthCheckRequest {
        service: args.service,
    };
    let response = client.health_check(request.clone()).await?;
    capture_if_enabled(config, "grpc.health.v1.Health.Check", &request, &response)?;
    let status = ServingStatus::from_i32(response.status);
    if config.output == OutputFormat::Proto {
        output::emit_proto(&response)
    } else if config.output == OutputFormat::Table {
        output::table(
            &["SERVICE", "STATUS", "SERVING"],
            &[vec![
                request.service,
                status.to_string(),
                status.is_serving().to_string(),
            ]],
        );
        Ok(())
    } else {
        output::emit_json(
            config.output,
            &serde_json::json!({
                "service": request.service,
                "status": status.to_string(),
                "serving": status.is_serving(),
            }),
        )
    }
}
