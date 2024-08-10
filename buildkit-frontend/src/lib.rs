#![deny(warnings)]
#![deny(clippy::all)]

use anyhow::{Context, Result};
use log::*;
use serde::de::DeserializeOwned;
use tonic::transport::Endpoint;
use tower::service_fn;

mod bridge;
mod error;
mod stdio;
mod utils;

pub mod oci;
pub mod options;

use oci::ImageSpecification;
use utils::OutputRef;

pub use self::bridge::Bridge;
pub use self::error::ErrorCode;
pub use self::options::Options;
pub use self::stdio::{stdio_connector, StdioSocket};

#[tonic::async_trait]
pub trait Frontend<O = Options>
where
    O: DeserializeOwned,
{
    async fn run(self, bridge: Bridge, options: O) -> Result<FrontendOutput>;
}

pub struct FrontendOutput {
    output: OutputRef,
    image_spec: Option<ImageSpecification>,
}

impl FrontendOutput {
    pub fn with_ref(output: OutputRef) -> Self {
        Self {
            output,
            image_spec: None,
        }
    }

    pub fn with_spec_and_ref(spec: ImageSpecification, output: OutputRef) -> Self {
        Self {
            output,
            image_spec: Some(spec),
        }
    }
}

pub async fn run_frontend<F, O>(frontend: F) -> Result<()>
where
    F: Frontend<O>,
    O: DeserializeOwned,
{
    let service = service_fn(stdio_connector);
    let channel = {
        Endpoint::from_static("http://[::]:50051")
            .connect_with_connector(service)
            .await?
    };

    let bridge = Bridge::new(channel);

    match frontend_entrypoint(&bridge, frontend).await {
        Ok(output) => {
            bridge
                .finish_with_success(output.output, output.image_spec)
                .await
                .context("Unable to send a success result")?;
        }

        Err(error) => {
            error!("Frontend entrypoint failed: {}", error);

            // https://godoc.org/google.golang.org/grpc/codes#Code
            bridge
                .finish_with_error(ErrorCode::Unknown, error.to_string())
                .await
                .context("Unable to send an error result")?;
        }
    }

    // TODO: gracefully shutdown the HTTP/2 connection

    Ok(())
}

async fn frontend_entrypoint<F, O>(bridge: &Bridge, frontend: F) -> Result<FrontendOutput>
where
    F: Frontend<O>,
    O: DeserializeOwned,
{
    let options = options::from_env(std::env::vars()).context("Unable to parse options")?;

    debug!("running a frontend entrypoint");
    frontend.run(bridge.clone(), options).await
}
