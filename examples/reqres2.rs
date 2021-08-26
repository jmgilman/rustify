use std::str::FromStr;

use bytes::Bytes;
use derive_builder::Builder;
use rustify::{errors::ClientError, Client, Endpoint, MiddleWare};
use rustify_derive::Endpoint;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// With this endpoint we are actually giving the struct some fields that will be
// used to construct the JSON body of the request. Additionally, we need to tell
// rustify that this should be a POST request.
//
// The actual API doesn't include an `opt` argument, however, it's included here
// for demonstration purposes. Using `setter(into)` here makes creating the
// request easier since we can pass string slices, as an example. Using
// `setter(strip_option)` allows passing in optional arguments without wrapping
// them in `Some`. The combination of `builder(default)` with
// `skip_serializing_none` means that any optional field that is not set when
// the endpoint is built will not be included in the request body. This prevents
// sending something like {"opt": ""} which in some cases could actually
// overwrite an existing value.
//
// The reqres API doesn't specify which arguments are required, however, for the
// sake of this example we assume `name` and `job` are required and we therefore
// do not wrap them in an Option<> enum.
#[skip_serializing_none]
#[derive(Builder, Default, Endpoint, Serialize)]
#[endpoint(
    path = "users",
    method = "POST",
    result = "CreateUserResponse",
    builder = "true"
)]
#[builder(setter(into, strip_option), default)]
struct CreateUserRequest {
    pub name: String,
    pub job: String,
    pub opt: Option<String>,
}

// The API returns an ID and timestamp.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateUserResponse {
    pub id: String,
    pub created_at: String,
}

// Rustify allows passing Middleware when executing an endpoint. Implementations
// of this trait contain two methods for operating on outgoing requests and
// incoming responses. In our case, all of the paths in our API calls share a
// common trait of needed to be prepended with "/api". Wouldn't it be nice to
// automatically do this instead of having to specify it for every endpoint?
//
// The below implementation modifies all outgoing requests by automatically
// prepending "/api" to the URL path.
struct Middle {}
impl MiddleWare for Middle {
    fn request<E: Endpoint>(
        &self,
        _: &E,
        req: &mut http::Request<Bytes>,
    ) -> Result<(), ClientError> {
        // Prepending to the path of a URL is not a trivial task. Here we use
        // the `url` crate which offers better support for mutating a URL. We
        // parse the final result back into an `http::Uri`.
        let url = url::Url::parse(req.uri().to_string().as_str()).unwrap();
        let mut url_c = url.clone();
        let mut segs: Vec<&str> = url.path_segments().unwrap().collect();
        segs.insert(0, "api");
        url_c.path_segments_mut().unwrap().clear().extend(segs);
        *req.uri_mut() = http::Uri::from_str(url_c.as_str()).unwrap();
        Ok(())
    }

    fn response<E: Endpoint>(
        &self,
        _: &E,
        _: &mut http::Response<Bytes>,
    ) -> Result<(), ClientError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // Just like in the first example we must first create a client.
    let client = Client::default("https://reqres.in/");

    // Then we can construct our endpoint
    let endpoint = CreateUserRequest::builder()
        .name("John")
        .job("Programmer")
        .build()
        .unwrap();

    // Here we use `exec_mut` which allows us to pass the Wrapper we created
    // earlier for mutating our outgoing requests.
    let result = endpoint
        .exec_mut(&client, &Middle {})
        .await
        .unwrap()
        .unwrap();
    println!(
        "Created user {} with ID {} at {}",
        endpoint.name, result.id, result.created_at
    );
}
