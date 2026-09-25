use k8s_openapi::api::core::v1::Pod;
use kube::{
    Client, Config,
    api::{Api, ListParams},
};
use serde_json::Value;

use crate::error::{AppError, Result};

pub struct KubeClusterClient {
    client: Option<Client>,
}

impl KubeClusterClient {
    pub async fn try_connect() -> Self {
        match Config::infer().await {
            Ok(config) => match Client::try_from(config) {
                Ok(client) => Self {
                    client: Some(client),
                },
                Err(_) => Self { client: None },
            },
            Err(_) => Self { client: None },
        }
    }

    #[cfg(test)]
    pub fn new_disconnected() -> Self {
        Self { client: None }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub async fn list_pods(&self, namespace: Option<&str>) -> Result<Vec<Value>> {
        let client = self.client.as_ref().ok_or_else(|| AppError::Connection {
            endpoint: "kubernetes".to_owned(),
            message: "not connected".to_owned(),
            hint: "ensure kubeconfig or in-cluster credentials are available",
        })?;

        let pods_api: Api<Pod> = if let Some(ns) = namespace {
            Api::namespaced(client.clone(), ns)
        } else {
            Api::all(client.clone())
        };

        let pod_list =
            pods_api
                .list(&ListParams::default())
                .await
                .map_err(|e| AppError::Negative {
                    class: "k8s",
                    status: e.to_string(),
                })?;

        let mut values = Vec::new();
        for pod in pod_list.items {
            let json_val = serde_json::to_value(&pod)
                .map_err(|err| AppError::Configuration(err.to_string()))?;
            values.push(json_val);
        }
        Ok(values)
    }
}
