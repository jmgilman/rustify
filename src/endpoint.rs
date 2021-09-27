//! Contains the [Endpoint] trait and supporting traits/functions.

use std::marker::PhantomData;

#[cfg(feature = "blocking")]
use crate::blocking::client::Client as BlockingClient;
use crate::{
    client::Client,
    enums::{RequestMethod, RequestType, ResponseType},
    errors::ClientError,
};
use async_trait::async_trait;
use http::{Request, Response};
use serde::de::DeserializeOwned;

/// Represents a generic wrapper that can be applied to [Endpoint] results.
///
/// Some APIs use a generic wrapper when returning responses that contains
/// information about the response and the actual response data in a subfield.
/// This trait allows implementing a generic wrapper which can be used with
/// [EndpointResult::wrap] to automatically wrap the [Endpoint::Response] in the
/// wrapper. The only requirement is that the [Wrapper::Value] must enclose
/// the [Endpoint::Response].
pub trait Wrapper: DeserializeOwned + Send + Sync {
    type Value;
}

/// Represents an [Endpoint] that has had [MiddleWare] applied to it.
///
/// This type wraps [Endpoint] by implementng it. The primary difference is
/// when `exec` is called the request and response will potentially be mutated
/// before processing. Only one [MiddleWare] can be applied to a single
/// [Endpoint].
pub struct MutatedEndpoint<'a, E: Endpoint, M: MiddleWare> {
    endpoint: E,
    middleware: &'a M,
}

impl<'a, E: Endpoint, M: MiddleWare> MutatedEndpoint<'a, E, M> {
    /// Returns a new [MutatedEndpoint].
    pub fn new(endpoint: E, middleware: &'a M) -> Self {
        MutatedEndpoint {
            endpoint,
            middleware,
        }
    }
}

#[async_trait]
impl<E: Endpoint, M: MiddleWare> Endpoint for MutatedEndpoint<'_, E, M> {
    type Response = E::Response;
    const REQUEST_BODY_TYPE: RequestType = E::REQUEST_BODY_TYPE;
    const RESPONSE_BODY_TYPE: ResponseType = E::RESPONSE_BODY_TYPE;

    fn path(&self) -> String {
        self.endpoint.path()
    }

    fn method(&self) -> RequestMethod {
        self.endpoint.method()
    }

    fn query(&self) -> Result<Option<String>, ClientError> {
        self.endpoint.query()
    }

    fn body(&self) -> Result<Option<Vec<u8>>, ClientError> {
        self.endpoint.body()
    }

    #[instrument(skip(self), err)]
    fn url(&self, base: &str) -> Result<http::Uri, ClientError> {
        self.endpoint.url(base)
    }

    #[instrument(skip(self), err)]
    fn request(&self, base: &str) -> Result<Request<Vec<u8>>, ClientError> {
        let mut req = crate::http::build_request(
            base,
            &self.path(),
            self.method(),
            self.query()?,
            self.body()?,
        )?;

        self.middleware.request(self, &mut req)?;
        Ok(req)
    }

    #[instrument(skip(self, client), err)]
    async fn exec(
        &self,
        client: &impl Client,
    ) -> Result<EndpointResult<Self::Response>, ClientError> {
        info!("Executing endpoint");

        let req = self.request(client.base())?;
        let resp = exec_mut(client, self, req, self.middleware).await?;
        Ok(EndpointResult::new(resp, Self::RESPONSE_BODY_TYPE))
    }

    #[cfg(feature = "blocking")]
    fn exec_block(
        &self,
        client: &impl BlockingClient,
    ) -> Result<EndpointResult<Self::Response>, ClientError> {
        info!("Executing endpoint");

        let req = self.request(client.base())?;
        let resp = exec_block_mut(client, self, req, self.middleware)?;
        Ok(EndpointResult::new(resp, Self::RESPONSE_BODY_TYPE))
    }
}

/// Represents a remote HTTP endpoint which can be executed using a
/// [crate::client::Client].
///
/// This trait can be implemented directly, however, users should prefer using
/// the provided `rustify_derive` macro for generating implementations. An
/// Endpoint consists of:
///   * An `action` which is combined with the base URL of a Client to form a
///     fully qualified URL.
///   * A `method` of type [RequestType] which determines the HTTP method used
///     when a Client executes this endpoint.
///   * A `ResponseType` type which determines the type of response this
///     Endpoint will return when executed.
///
/// The fields of the struct act as a representation of data that will be
/// serialized and sent to the remote server. Where and how each field appears
/// in the final request is determined by how they are tagged with attributes.
/// For example, fields with `#[endpoint(query)]` will show up as a query
/// parameter and fields with `#[endpoint(body)]` will show up in the body in
/// the format specified by [Endpoint::REQUEST_BODY_TYPE]. By default, if no
/// fields are tagged with `#[endpoint(body)]` or `#[endpoint(raw)]` then any
/// untagged fields are assumed to be tagged with `#[endpoint(body)]` (this
/// reduces a large amount of boilerplate). Fields that should be excluded from
/// this behavior can be tagged with `#[endpoint(skip)]`.
///
/// It's worth noting that fields which have the [Option] type and whose value,
/// at runtime, is [Option::None] will not be serialized. This avoids defining
/// data parameters which were not specified when the endpoint was created.
///
/// A number of useful methods are provided for obtaining information about an
/// endpoint including its URL, HTTP method, and request data. The `request`
/// method can be used to produce a fully valid HTTP [Request] that can be used
/// for executing an endpoint without using a built-in [Client] provided by
/// rustify.
///
/// # Example
/// ```
/// use rustify::clients::reqwest::Client;
/// use rustify::endpoint::Endpoint;
/// use rustify_derive::Endpoint;
///
/// #[derive(Endpoint)]
/// #[endpoint(path = "my/endpoint")]
/// struct MyEndpoint {}
///
/// // Configure a client with a base URL of http://myapi.com
/// let client = Client::default("http://myapi.com");
///     
/// // Construct a new instance of our Endpoint
/// let endpoint = MyEndpoint {};
///
/// // Execute our Endpoint using the client
/// // This sends a GET request to http://myapi.com/my/endpoint
/// // It assumes an empty response
/// # tokio_test::block_on(async {
/// let result = endpoint.exec(&client).await;
/// # })
/// ```
#[async_trait]
pub trait Endpoint: Send + Sync + Sized {
    /// The type that the raw response from executing this endpoint will
    /// deserialized into. This type is passed on to the [EndpointResult] and is
    /// used to determine the type returned when the `parse()` method is called.
    type Response: DeserializeOwned + Send + Sync;

    /// The content type of the request body
    const REQUEST_BODY_TYPE: RequestType;

    /// The content type of the response body
    const RESPONSE_BODY_TYPE: ResponseType;

    /// The relative URL path that represents the location of this Endpoint.
    /// This is combined with the base URL from a
    /// [Client][crate::client::Client] instance to create the fully qualified
    /// URL.
    fn path(&self) -> String;

    /// The HTTP method to be used when executing this Endpoint.
    fn method(&self) -> RequestMethod;

    /// Optional query parameters to add to the request.
    fn query(&self) -> Result<Option<String>, ClientError> {
        Ok(None)
    }

    /// Optional data to add to the body of the request.
    fn body(&self) -> Result<Option<Vec<u8>>, ClientError> {
        Ok(None)
    }

    /// Returns the full URL address of the endpoint using the base address.
    #[instrument(skip(self), err)]
    fn url(&self, base: &str) -> Result<http::Uri, ClientError> {
        crate::http::build_url(base, &self.path(), self.query()?)
    }

    /// Returns a [Request] containing all data necessary to execute against
    /// this endpoint.
    #[instrument(skip(self), err)]
    fn request(&self, base: &str) -> Result<Request<Vec<u8>>, ClientError> {
        crate::http::build_request(
            base,
            &self.path(),
            self.method(),
            self.query()?,
            self.body()?,
        )
    }

    /// Executes the Endpoint using the given [Client].
    #[instrument(skip(self, client), err)]
    async fn exec(
        &self,
        client: &impl Client,
    ) -> Result<EndpointResult<Self::Response>, ClientError> {
        info!("Executing endpoint");

        let req = self.request(client.base())?;
        let resp = exec(client, req).await?;
        Ok(EndpointResult::new(resp, Self::RESPONSE_BODY_TYPE))
    }

    fn with_middleware<M: MiddleWare>(self, middleware: &M) -> MutatedEndpoint<Self, M> {
        MutatedEndpoint::new(self, middleware)
    }

    /// Executes the Endpoint using the given [Client].
    #[cfg(feature = "blocking")]
    #[instrument(skip(self, client), err)]
    fn exec_block(
        &self,
        client: &impl BlockingClient,
    ) -> Result<EndpointResult<Self::Response>, ClientError> {
        info!("Executing endpoint");

        let req = self.request(client.base())?;
        let resp = exec_block(client, req)?;
        Ok(EndpointResult::new(resp, Self::RESPONSE_BODY_TYPE))
    }
}

/// A response from executing an [Endpoint].
///
/// All [Endpoint] executions will result in an [EndpointResult] which wraps
/// the actual HTTP [Response] and the final result type. The response can be
/// parsed into the final result type by calling `parse()` or optionally
/// wrapped by a [Wrapper] by calling `wrap()`.
pub struct EndpointResult<T: DeserializeOwned + Send + Sync> {
    pub response: Response<Vec<u8>>,
    pub ty: ResponseType,
    inner: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync> EndpointResult<T> {
    /// Returns a new [EndpointResult].
    pub fn new(response: Response<Vec<u8>>, ty: ResponseType) -> Self {
        EndpointResult {
            response,
            ty,
            inner: PhantomData,
        }
    }

    /// Parses the response into the final result type.
    #[instrument(skip(self), err)]
    pub fn parse(&self) -> Result<T, ClientError> {
        match self.ty {
            ResponseType::JSON => serde_json::from_slice(self.response.body()).map_err(|e| {
                ClientError::ResponseParseError {
                    source: e.into(),
                    content: String::from_utf8(self.response.body().to_vec()).ok(),
                }
            }),
        }
    }

    /// Returns the raw response body from the HTTP [Response].
    pub fn raw(&self) -> Vec<u8> {
        self.response.body().clone()
    }

    /// Parses the response into the final result type and then wraps it in the
    /// given [Wrapper].
    #[instrument(skip(self), err)]
    pub fn wrap<W>(&self) -> Result<W, ClientError>
    where
        W: Wrapper<Value = T>,
    {
        match self.ty {
            ResponseType::JSON => serde_json::from_slice(self.response.body()).map_err(|e| {
                ClientError::ResponseParseError {
                    source: e.into(),
                    content: String::from_utf8(self.response.body().to_vec()).ok(),
                }
            }),
        }
    }
}

/// Modifies an [Endpoint] request and/or response before final processing.
///
/// Types implementing this trait that do not desire to implement both methods
/// should instead return `OK(())` to bypass any processing of the [Request] or
/// [Response].
pub trait MiddleWare: Sync + Send {
    /// Modifies a [Request] from an [Endpoint] before it's executed.
    fn request<E: Endpoint>(
        &self,
        endpoint: &E,
        req: &mut Request<Vec<u8>>,
    ) -> Result<(), ClientError>;

    /// Modifies a [Response] from an [Endpoint] before being returned as an
    /// [EndpointResult].
    fn response<E: Endpoint>(
        &self,
        endpoint: &E,
        resp: &mut Response<Vec<u8>>,
    ) -> Result<(), ClientError>;
}

async fn exec(
    client: &impl Client,
    req: Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>, ClientError> {
    client.execute(req).await
}

async fn exec_mut(
    client: &impl Client,
    endpoint: &impl Endpoint,
    req: Request<Vec<u8>>,
    middle: &impl MiddleWare,
) -> Result<Response<Vec<u8>>, ClientError> {
    let mut resp = client.execute(req).await?;
    middle.response(endpoint, &mut resp)?;
    Ok(resp)
}

#[cfg(feature = "blocking")]
fn exec_block(
    client: &impl BlockingClient,
    req: Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>, ClientError> {
    client.execute(req)
}

#[cfg(feature = "blocking")]
fn exec_block_mut(
    client: &impl BlockingClient,
    endpoint: &impl Endpoint,
    req: Request<Vec<u8>>,
    middle: &impl MiddleWare,
) -> Result<Response<Vec<u8>>, ClientError> {
    let mut resp = client.execute(req)?;
    middle.response(endpoint, &mut resp)?;
    Ok(resp)
}
