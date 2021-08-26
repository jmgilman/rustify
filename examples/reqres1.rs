use std::collections::HashMap;

use derive_builder::Builder;
use rustify::{Client, Endpoint, Wrapper};
use rustify_derive::Endpoint;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// Endpoints must derive `serde::Serialize` as rustify uses the fields of the
// endpoint struct to form the contents of the request body. If the struct
// contains zero fields then no request body will be sent. It's important to
// tag fields that shouldn't show up in the request body with `#[serde(skip)]`.
//
// While using a builder archetype for requests is not required, it's often the
// cleanest way for building requests. For this endpoint it doesn't bring too
// much benefit, however, for consistency it's implemented anyways.
//
// Setting the `builder` attribute to true adds a `builder()` method to the
// struct for easily getting a default instance of the request builder.
//
// Rustify supports a few more parameters in the endpoint definition than are
// shown here. It resorts to sane defaults in most cases. For our example they
// are as follows:
// * method: defaults to GET
// * request_type: defaults to JSON
// * response_type: defaults to JSON
#[derive(Builder, Endpoint, Serialize)]
#[endpoint(path = "/api/users", response = "Vec<User>", builder = "true")]
struct ListUsersRequest {
    // Tagging this field with #[endpoint(query)] informs rustify that this
    // field should be appended as a query parameter to the request URL.
    #[endpoint(query)]
    #[serde(skip)]
    pub page: usize,
}

// Some responses from the API are paginated and contain a common wrapper around
// the actual resulting data. Since this is so prevalent in APIs, rustify offers
// a `Wrapper` which can be used to define this behavior.
//
// Below we define the details of the wrapper that appears around paginated
// responses. The form of the resulting data field is specified with a generic
// and will be supplied when we call the endpoint. Endpoints have a special
// `exec_wrap()` method which will automatically wrap the response from the
// endpoint in the given wrapper.
#[derive(Debug, Deserialize)]
pub struct PaginationWrapper<T> {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
    pub data: T,
    pub support: HashMap<String, String>,
}

// This is almost always the form that the implementation will take.
// Unforunately, Rust does not support associated types having a default
// type set to a generic, so we must define it when we use it.
impl<T: DeserializeOwned> Wrapper for PaginationWrapper<T> {
    type Value = T;
}

// Our endpoint returns a JSON array of objects which each contain information
// about a user. We represent this by creating a `User` struct and then using
// `Vec<User>` in the `response` parameter of the endpoint to inform rustify on
// how it should deserialize the response. We don't need to worry about the
// wrapper because it's handled for us!
#[derive(Debug, Deserialize)]
struct User {
    pub id: usize,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar: String,
}

#[tokio::main]
async fn main() {
    // In order to execute endpoints, we must first create a client configured
    // with the base URL of our HTTP API server. In this case we're using the
    // popular reqres.in for our example.
    // Asynchronous clients can be found in rustify::clients and synchronous
    // clients in rustify::blocking::clients.
    let client = Client::default("https://reqres.in/");

    // We use the builder archetype here for constructing an instance of the
    // endpoint that we can then execute. It's safe to unwrap because we know
    // that all required fields have been specified.
    let endpoint = ListUsersRequest::builder().page(1).build().unwrap();

    // Here is where the magic of rustify happens. We call `exec_wrap()` which
    // takes two arguments: an instance of a `Client` and a generic type
    // parameter which specifies what the response should be wrapped in. Behind
    // the scenes rustify will initiate a connection to the API server and send
    // a HTTP request as defined by the endpoint. In this case, it sends a GET
    // request to https://reqres.in/api/users?page=1 and automatically
    // deserializes the response into a PaginationWrapper<ListUsersResponse>.
    //
    // The response type is wrapped in Option<> since it's possible for the API
    // to return an empty response.
    let result: Result<Option<PaginationWrapper<_>>, _> = endpoint.exec_wrap(&client).await;

    // Executing an endpoint can fail for a number of reasons: there was a
    // problem building the request, an underlying network issue, the server
    // returned a non-200 response, the response could not be properly
    // deserialized, etc. Rustify uses a common error enum which contains a
    // number of variants for identifying the root cause.
    match result {
        Ok(r) => match r {
            Some(d) => {
                d.data.iter().for_each(print_user);
            }
            None => println!("Error: The server returned an empty response!"),
        },
        Err(e) => println!("Error: {:#?}", e),
    };
}

fn print_user(user: &User) {
    println!(
        "ID: {}\nEmail: {}\nFirst Name: {}\nLast Name: {}\n\n",
        user.id, user.email, user.first_name, user.last_name
    );
}
