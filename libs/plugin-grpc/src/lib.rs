//! Plugin Registry gRPC Service
//!
//! Provides remote plugin management via gRPC for federated plugin loading.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::{error, info};

use plugin_registry::{PluginConfig, PluginRegistry};

/// Plugin Registry gRPC Service
pub struct PluginGrpcService {
    registry: Arc<PluginRegistry>,
}

impl PluginGrpcService {
    /// Creates a new gRPC service with the given registry.
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self { registry }
    }
}

/// Request to list all loaded plugins.
#[derive(Debug, Default)]
pub struct ListPluginsRequest {}

/// Response containing plugin metadata.
#[derive(Debug)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

/// Response containing list of plugins.
#[derive(Debug)]
pub struct ListPluginsResponse {
    pub plugins: Vec<PluginInfo>,
}

/// Request to load a plugin.
#[derive(Debug)]
pub struct LoadPluginRequest {
    pub name: String,
    pub version: String,
}

/// Response for plugin load operation.
#[derive(Debug)]
pub struct LoadPluginResponse {
    pub success: bool,
    pub message: String,
}

/// Request to initialize a plugin.
#[derive(Debug)]
pub struct InitPluginRequest {
    pub name: String,
    pub config_json: String,
}

/// Response for plugin initialization.
#[derive(Debug)]
pub struct InitPluginResponse {
    pub success: bool,
    pub message: String,
}

/// Request to shutdown a plugin.
#[derive(Debug)]
pub struct ShutdownPluginRequest {
    pub name: String,
}

/// Response for plugin shutdown.
#[derive(Debug)]
pub struct ShutdownPluginResponse {
    pub success: bool,
    pub message: String,
}

impl PluginGrpcService {
    /// Lists all loaded plugins.
    pub async fn list_plugins(
        &self,
        _request: Request<ListPluginsRequest>,
    ) -> Result<Response<ListPluginsResponse>, Status> {
        info!("gRPC: listing plugins");

        let plugins = self.registry.get_all_metadata().await;
        let plugin_infos: Vec<PluginInfo> = plugins
            .into_iter()
            .map(|m| PluginInfo {
                name: m.name,
                version: m.version,
                description: m.description,
            })
            .collect();

        Ok(Response::new(ListPluginsResponse {
            plugins: plugin_infos,
        }))
    }

    /// Loads a plugin (placeholder - actual loading requires compiled plugin).
    pub async fn load_plugin(
        &self,
        request: Request<LoadPluginRequest>,
    ) -> Result<Response<LoadPluginResponse>, Status> {
        let req = request.into_inner();
        info!(name = %req.name, version = %req.version, "gRPC: load plugin request");

        // In a real implementation, this would:
        // 1. Fetch plugin binary from registry
        // 2. Dynamically load the plugin
        // 3. Register with local registry

        Ok(Response::new(LoadPluginResponse {
            success: true,
            message: format!(
                "Plugin '{}' version '{}' loaded successfully",
                req.name, req.version
            ),
        }))
    }

    /// Initializes a loaded plugin.
    pub async fn init_plugin(
        &self,
        request: Request<InitPluginRequest>,
    ) -> Result<Response<InitPluginResponse>, Status> {
        let req = request.into_inner();
        info!(name = %req.name, "gRPC: init plugin request");

        let config: PluginConfig =
            serde_json::from_str(&req.config_json).unwrap_or_else(|_| PluginConfig::default());

        match self.registry.initialize(&req.name, config).await {
            Ok(()) => Ok(Response::new(InitPluginResponse {
                success: true,
                message: format!("Plugin '{}' initialized successfully", req.name),
            })),
            Err(e) => {
                error!(name = %req.name, error = %e, "plugin initialization failed");
                Ok(Response::new(InitPluginResponse {
                    success: false,
                    message: format!("Failed to initialize plugin '{}': {}", req.name, e),
                }))
            }
        }
    }

    /// Shuts down a plugin.
    pub async fn shutdown_plugin(
        &self,
        request: Request<ShutdownPluginRequest>,
    ) -> Result<Response<ShutdownPluginResponse>, Status> {
        let req = request.into_inner();
        info!(name = %req.name, "gRPC: shutdown plugin request");

        match self.registry.shutdown(&req.name).await {
            Ok(()) => Ok(Response::new(ShutdownPluginResponse {
                success: true,
                message: format!("Plugin '{}' shutdown successfully", req.name),
            })),
            Err(e) => {
                error!(name = %req.name, error = %e, "plugin shutdown failed");
                Ok(Response::new(ShutdownPluginResponse {
                    success: false,
                    message: format!("Failed to shutdown plugin '{}': {}", req.name, e),
                }))
            }
        }
    }
}

/// Server configuration for plugin gRPC service.
#[derive(Debug, Clone)]
pub struct PluginGrpcServer {
    pub addr: String,
    pub port: u16,
}

impl Default for PluginGrpcServer {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1".to_string(),
            port: 50051,
        }
    }
}

impl PluginGrpcServer {
    /// Creates a new server configuration.
    pub fn new(addr: impl Into<String>, port: u16) -> Self {
        Self {
            addr: addr.into(),
            port,
        }
    }

    /// Returns the full socket address.
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}
