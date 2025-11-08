//! BuildKit solve operation implementation

use crate::builder::{BuildConfig, DockerfileSource};
use crate::client::BuildKitClient;
use crate::progress::ProgressHandler;
use anyhow::{Context, Result};
use crate::proto::moby::buildkit::v1::{
    Exporter, SolveRequest, StatusRequest, CacheOptions, CacheOptionsEntry,
};
use std::collections::HashMap;
use tokio_stream::StreamExt;
use uuid::Uuid;

/// Build result containing the image digest and metadata
#[derive(Debug)]
pub struct BuildResult {
    /// Container image digest
    pub digest: Option<String>,
    /// Export metadata
    pub metadata: HashMap<String, String>,
}

impl BuildKitClient {
    /// Execute a build operation with the given configuration
    ///
    /// # Arguments
    /// * `config` - Build configuration
    /// * `progress_handler` - Optional progress handler for real-time updates
    ///
    /// # Returns
    /// Build result containing digest and metadata
    pub async fn build(
        &mut self,
        config: BuildConfig,
        mut progress_handler: Option<Box<dyn ProgressHandler>>,
    ) -> Result<BuildResult> {
        // Generate unique build reference
        let build_ref = format!("build-{}", Uuid::new_v4());
        tracing::info!("Starting build with ref: {}", build_ref);

        // Prepare frontend attributes
        let mut frontend_attrs = HashMap::new();

        // Set dockerfile filename
        match &config.source {
            DockerfileSource::Local { dockerfile_path, .. } => {
                if let Some(path) = dockerfile_path {
                    frontend_attrs.insert(
                        "filename".to_string(),
                        path.to_string_lossy().to_string(),
                    );
                }
            }
            DockerfileSource::GitHub { dockerfile_path, .. } => {
                if let Some(path) = dockerfile_path {
                    frontend_attrs.insert("filename".to_string(), path.clone());
                }
            }
        }

        // Add build args
        for (key, value) in &config.build_args {
            frontend_attrs.insert(format!("build-arg:{}", key), value.clone());
        }

        // Set target stage
        if let Some(target) = &config.target {
            frontend_attrs.insert("target".to_string(), target.clone());
        }

        // Set platforms
        if !config.platforms.is_empty() {
            let platforms_str = config
                .platforms
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",");
            frontend_attrs.insert("platform".to_string(), platforms_str);
        }

        // Set no-cache
        if config.no_cache {
            frontend_attrs.insert("no-cache".to_string(), "true".to_string());
        }

        // Set pull
        if config.pull {
            frontend_attrs.insert("image-resolve-mode".to_string(), "pull".to_string());
        }

        // Prepare context source
        let context = self.prepare_context(&config).await?;
        frontend_attrs.insert("context".to_string(), context);

        // Prepare exports (push to registry)
        let mut exports = Vec::new();
        if !config.tags.is_empty() {
            let mut export_attrs = HashMap::new();
            export_attrs.insert("name".to_string(), config.tags.join(","));
            export_attrs.insert("push".to_string(), "true".to_string());

            // Add registry authentication if provided
            if let Some(auth) = &config.registry_auth {
                export_attrs.insert(
                    "registry.insecure".to_string(),
                    if auth.host.starts_with("localhost") {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    },
                );
            }

            exports.push(Exporter {
                r#type: "image".to_string(),
                attrs: export_attrs,
            });
        }

        // Prepare cache imports
        let cache_imports = config
            .cache_from
            .iter()
            .map(|source| {
                let mut attrs = HashMap::new();
                attrs.insert("ref".to_string(), source.clone());
                CacheOptionsEntry {
                    r#type: "registry".to_string(),
                    attrs,
                }
            })
            .collect();

        // Prepare cache exports
        let cache_exports = config
            .cache_to
            .iter()
            .map(|dest| {
                let mut attrs = HashMap::new();
                attrs.insert("ref".to_string(), dest.clone());
                attrs.insert("mode".to_string(), "max".to_string());
                CacheOptionsEntry {
                    r#type: "registry".to_string(),
                    attrs,
                }
            })
            .collect();

        // Create solve request
        let request = SolveRequest {
            r#ref: build_ref.clone(),
            definition: None,
            exporter_deprecated: String::new(),
            exporter_attrs_deprecated: HashMap::new(),
            session: String::new(),
            frontend: "dockerfile.v0".to_string(),
            frontend_attrs,
            cache: Some(CacheOptions {
                export_ref_deprecated: String::new(),
                import_refs_deprecated: vec![],
                export_attrs_deprecated: HashMap::new(),
                exports: cache_exports,
                imports: cache_imports,
            }),
            entitlements: vec![],
            frontend_inputs: HashMap::new(),
            internal: false,
            source_policy: None,
            exporters: exports,
            enable_session_exporter: false,
            source_policy_session: String::new(),
        };

        // Start the build
        tracing::info!("Sending solve request to buildkit");
        let response = self
            .control()
            .solve(request)
            .await
            .context("Failed to execute solve")?;

        let solve_response = response.into_inner();

        // Monitor build progress if handler is provided
        if let Some(ref mut handler) = progress_handler {
            self.monitor_progress(&build_ref, handler).await?;
        }

        // Extract digest and metadata
        let digest = solve_response
            .exporter_response
            .get("containerimage.digest")
            .cloned();

        tracing::info!("Build completed successfully");
        if let Some(ref d) = digest {
            tracing::info!("Image digest: {}", d);
        }

        Ok(BuildResult {
            digest,
            metadata: solve_response.exporter_response,
        })
    }

    /// Prepare build context based on source type
    async fn prepare_context(&self, config: &BuildConfig) -> Result<String> {
        match &config.source {
            DockerfileSource::Local { context_path, .. } => {
                let abs_path = std::fs::canonicalize(context_path)
                    .context("Failed to resolve context path")?;
                Ok(format!("local://{}", abs_path.display()))
            }
            DockerfileSource::GitHub {
                repo_url,
                git_ref,
                token,
                ..
            } => {
                let mut url = repo_url.clone();

                // Add authentication token if provided
                if let Some(token) = token {
                    // Format: https://token@github.com/user/repo.git
                    url = url.replace("https://", &format!("https://{}@", token));
                }

                // Add git reference
                if let Some(git_ref) = git_ref {
                    url = format!("{}#{}", url, git_ref);
                }

                Ok(url)
            }
        }
    }

    /// Monitor build progress and send updates to the handler
    async fn monitor_progress(
        &mut self,
        build_ref: &str,
        handler: &mut Box<dyn ProgressHandler>,
    ) -> Result<()> {
        let status_request = StatusRequest {
            r#ref: build_ref.to_string(),
        };

        let mut stream = self
            .control()
            .status(status_request)
            .await
            .context("Failed to get status stream")?
            .into_inner();

        handler.on_start()?;

        while let Some(response) = stream.next().await {
            match response {
                Ok(status) => {
                    handler.on_status(status)?;
                }
                Err(e) => {
                    tracing::error!("Status stream error: {}", e);
                    handler.on_error(&e.to_string())?;
                    break;
                }
            }
        }

        handler.on_complete()?;
        Ok(())
    }
}
