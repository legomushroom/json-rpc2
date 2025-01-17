//! Non-blocking implementation, requires the `async` feature.

use crate::{Error, Request, Response, Result};
use async_trait::async_trait;

#[async_trait]
/// Trait for async services that maybe handle a request.
///
/// Only available with the `async` feature.
pub trait Service: Send + Sync {
    /// Type of the user data for this service.
    type Data: Send + Sync;

    /// See [Service](crate::Service) for more information.
    async fn handle(
        &self,
        request: &Request,
        ctx: &Self::Data,
    ) -> Result<Option<Response>>;
}

/// Serve requests.
///
/// Requests are passed to each service in turn and the first service
/// that returns a response wins.
///
/// Only available with the `async` feature.
pub struct Server<T: Send + Sync> {
    /// Services that the server should invoke for every request.
    services: Vec<Box<dyn Service<Data = T>>>,
}

impl<T: Send + Sync> Server<T> {
    /// Create a new server.
    pub fn new(services: Vec<Box<dyn Service<Data = T>>>) -> Self {
        Self { services }
    }

    /// Call services in order and return the first response message.
    ///
    /// If no services match the incoming request this will
    /// return `Error::MethodNotFound`.
    pub(crate) async fn handle(
        &self,
        request: &Request,
        ctx: &T,
    ) -> Result<Response> {
        for service in self.services.iter() {
            if let Some(result) = service.handle(request, ctx).await? {
                return Ok(result);
            }
        }

        let err = Error::MethodNotFound {
            name: request.method().to_string(),
            id: request.id.clone(),
        };

        Ok((request, err).into())
    }

    /// Infallible service handler, errors are automatically converted to responses.
    ///
    /// If a request was a notification (no id field) this will yield `None`.
    pub async fn serve(&self, request: &Request, ctx: &T) -> Option<Response> {
        match self.handle(request, ctx).await {
            Ok(response) => {
                if response.error().is_some() || response.id().is_some() {
                    Some(response)
                } else {
                    None
                }
            }
            Err(e) => Some((request, e).into()),
        }
    }
}
