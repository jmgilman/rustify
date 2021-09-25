use std::str::FromStr;

use bytes::Bytes;
use derive_builder::Builder;
use rustify::{errors::ClientError, Client, Endpoint, MiddleWare};
use rustify_derive::Endpoint;
use serde::{Deserialize, Serialize};

// With this endpoint we are actually giving the struct some fields that will be
// used to construct the JSON body of the request. When building a request body,
// rustify performs a few checks to determine how the body should be
// constructed. You can tag a field with `#[endpoint(raw)]` to use that field
// directly as the request body (it must be a `Vec<u8>`), you can tag one or
// more fields with #[endpoint(body)] to serialize them together into the
// request body, or as in the case below, if neither of the above tags are found
// then rustify automatically serializes all "untagged" fields as the request
// body.
//
// The actual API doesn't include an `opt` argument, however, it's included here
// for demonstration purposes. Using `setter(into)` here makes creating the
// request easier since we can pass string slices, as an example. Using
// `setter(strip_option)` allows passing in optional arguments without wrapping
// them in `Some`. By default, when rustify serializes the request body, any
// `Option` fields that have their value set to `None` will be skipped. This
// prevents sending something like {"opt": ""} which in some cases could
// actually overwrite an existing value.
//
// The reqres API doesn't specify which arguments are required, however, for the
// sake of this example we assume `name` and `job` are required and we therefore
// do not wrap them in an `Option`.
#[derive(Builder, Default, Endpoint, Serialize)]
#[endpoint(
    path = "users",
    method = "POST",
    response = "CreateUserResponse",
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
        req: &mut http::Request<Vec<u8>>,
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
