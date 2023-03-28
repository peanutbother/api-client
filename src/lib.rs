#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::pedantic)]

use serde::Serialize;

#[cfg(not(feature = "middleware"))]
/// Type of the reqwest client, depending on the features
pub type ClientType = reqwest::Client;

#[cfg(feature = "middleware")]
/// Type of the reqwest client, depending on the features
pub type ClientType = reqwest_middleware::ClientWithMiddleware;

#[cfg(not(feature = "middleware"))]
/// Type of the reqwest client, depending on the features
pub type ClientError = reqwest::Error;

#[cfg(feature = "middleware")]
/// Type of the reqwest client, depending on the features
pub type ClientError = reqwest_middleware::Error;

#[cfg(not(feature = "middleware"))]
/// Type of the reqwest `Result`type, depending on the features
pub type ResultType<T> = reqwest::Result<T>;

#[cfg(feature = "middleware")]
/// Type of the reqwest `Result`type, depending on the features
pub type ResultType<T> = reqwest_middleware::Result<T>;

#[cfg(not(feature = "middleware"))]
/// Type of the reqwest request builder, depending on the features
pub type RequestBuilder = reqwest::RequestBuilder;

#[cfg(feature = "middleware")]
/// Type of the reqwest request builder, depending on the features
pub type RequestBuilder = reqwest_middleware::RequestBuilder;

/// Used internally to the api! macro.
#[doc(hidden)]
pub enum Body<'a, T: Serialize + ?Sized = ()> {
    /// No body.
    None,
    /// JSON body.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    Json(&'a T),
    /// Form body.
    Form(&'a T),
    #[cfg(feature = "multipart")]
    #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
    /// Multipart body.
    Multipart(reqwest::multipart::Form),
}

/// The main API trait.
///
/// If you need custom behavior, such as authentication, you should implement this trait on your custom struct. See the [Api::pre_request] method for more details.
///
/// Otherwise, you can use the [api] macro to generate a struct with a proper implementation of this trait.
#[async_trait::async_trait(?Send)]
pub trait Api {
    /// Returns a reference to a reqwest Client to create requests.
    fn client(&self) -> &ClientType;

    /// You can use this method to modify the request before sending it.
    ///
    /// Some good examples of usage are:
    ///  - Authentication
    ///  - Custom headers (can also be done with a method on Client)
    ///
    /// # Authentication
    /// ```rust
    /// use api_client::{api, Api};
    /// use reqwest::{Client, RequestBuilder};
    ///
    /// struct ExampleApi {
    ///     client: Client,
    ///     username: String,
    ///     password: String
    /// }
    ///
    /// impl Api for ExampleApi {
    ///     fn client(&self) -> &Client {
    ///         &self.client
    ///     }
    ///
    ///     fn pre_request(&self, request: RequestBuilder) -> reqwest::Result<RequestBuilder> {
    ///         Ok(request.basic_auth(&self.username, Some(&self.password)))
    ///     }
    /// }
    ///
    /// impl ExampleApi {
    ///     api! {
    ///         fn example() -> String {
    ///            GET "https://example.com"
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn pre_request(&self, request: RequestBuilder) -> ResultType<RequestBuilder> {
        Ok(request)
    }

    /// You can use this method to modify the response before parsing it.
    ///
    /// Some good examples of usage are:
    ///  - Authentication
    ///  - Updating client fields
    ///
    /// # Authentication
    /// ```rust
    /// use api_client::{api, Api};
    /// use reqwest::{Client, RequestBuilder};
    ///
    /// struct ExampleApi {
    ///     client: Client,
    ///     username: String,
    ///     password: String
    /// }
    ///
    /// impl Api for ExampleApi {
    ///     fn client(&self) -> &Client {
    ///         &self.client
    ///     }
    ///
    ///     fn post_response(&mut self, response: Response) -> Response {
    ///         for cookie in self.cookies() {
    ///             // do something with cookie
    ///         }
    ///         response
    ///     }
    /// }
    ///
    /// impl ExampleApi {
    ///     api! {
    ///         fn example() -> String {
    ///            GET "https://example.com"
    ///         }
    ///     }
    /// }
    /// ```
    fn post_response(&mut self, response: reqwest::Response) -> reqwest::Response {
        response
    }

    /// Used internally in the api! macro. Mostly for ergonmics.
    ///
    /// # Usage
    /// ```rust
    /// # use api_client::{api, Api};
    ///
    /// api!(pub struct Example);
    ///
    /// fn main() {
    ///     let example = Example::new();
    /// }
    /// ```
    #[doc(hidden)]
    #[inline]
    #[must_use]
    fn new() -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    /// Used internally in the api! macro to handle all requests.
    #[doc(hidden)]
    #[inline]
    async fn request<T: Serialize + ?Sized>(
        &mut self,
        method: reqwest::Method,
        url: &str,
        body: Body<'_, T>,
    ) -> ResultType<reqwest::Response> {
        let request = self.pre_request(self.client().request(method, url))?;
        let request = match body {
            Body::None => request,
            #[cfg(feature = "json")]
            Body::Json(body) => request.json(body),
            Body::Form(body) => request.form(body),
            #[cfg(feature = "multipart")]
            Body::Multipart(form) => request.multipart(form),
        };
        request.send().await.map(|r| self.post_response(r))
    }
}

/// Magic macro for API structs.
///
/// # Simple Usage (auto generated struct)
/// ```rust
/// use api_client::{api, Api};
/// use reqwest::{Client, RequestBuilder};
///
/// api!(pub struct ExampleApi);
///
/// impl ExampleApi {
///     api! {
///         fn example() -> String {
///            GET "https://example.com"
///         }
///     }
/// }
/// ```
///
/// # Advanced Usage (manually created struct and [Api] implementation)
/// ```rust
/// use api_client::{api, Api};
/// use reqwest::{Client, RequestBuilder};
///
/// struct ExampleApi {
///     client: Client,
///     username: String,
///     password: String
/// }
///
/// impl Api for ExampleApi {
///     fn client(&self) -> &Client {
///         &self.client
///     }
///
///     fn pre_request(&self, request: RequestBuilder) -> reqwest::Result<RequestBuilder> {
///         Ok(request.basic_auth(&self.username, Some(&self.password)))
///     }
/// }
///
/// impl ExampleApi {
///     api! {
///         fn example() -> String {
///            GET "https://example.com"
///         }
///     }
/// }
/// ```
#[macro_export]
#[cfg(not(feature = "middleware"))]
macro_rules! api {
    () => {};

    ($(#[$attr:meta])* $vis:vis struct $ident:ident) => {
        $(#[$attr])*
        $vis struct $ident(::reqwest::Client);

        impl $crate::Api for $ident {
            fn client(&self) -> &::reqwest::Client {
                &self.0
            }

            fn new() -> Self where Self: Sized {
                $ident(::reqwest::Client::new())
            }
        }
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Json<$req:ty>$(, $name:ident: $ty:ty)*) -> StatusCode { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name $ty),*) -> ::reqwest::Result<::reqwest::StatusCode> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Json(request)).await.map(|res| res.status())
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Json<$req:ty>$(, $name:ident: $ty:ty)*) -> String { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest::Result<String> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Json(request)).await?.text().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Json<$req:ty>$(, $name:ident: $ty:ty)*) -> Bytes { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest::Result<::bytes::Bytes> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Json(request)).await?.bytes().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Json<$req:ty>$(, $name:ident: $ty:ty)*) -> Json<$res:ty> { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest::Result<$res> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Json(request)).await?.json().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Form<$req:ty>$(, $name:ident: $ty:ty)*) -> StatusCode { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest::Result<::reqwest::StatusCode> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Form(request)).await.map(|res| res.status())
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Form<$req:ty>$(, $name:ident: $ty:ty)*) -> String { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest::Result<String> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Form(request)).await?.text().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Form<$req:ty>$(, $name:ident: $ty:ty)*) -> Bytes { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest::Result<::bytes::Bytes> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Form(request)).await?.bytes().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Form<$req:ty>$(, $name:ident: $ty:ty)*) -> Json<$res:ty> { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest::Result<$res> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Form(request)).await?.json().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident($($name:ident: $ty:ty),*) -> StatusCode { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, $($name: $ty),*) -> ::reqwest::Result<::reqwest::StatusCode> {
            use $crate::Api as _;
            self.request::<()>(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::None).await.map(|res| res.status())
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident($($name:ident: $ty:ty),*) -> String { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, $($name: $ty),*) -> ::reqwest::Result<String> {
            use $crate::Api as _;
            self.request::<()>(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::None).await?.text().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident($($name:ident: $ty:ty),*) -> Bytes { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, $($name: $ty),*) -> ::reqwest::Result<::bytes::Bytes> {
            use $crate::Api as _;
            self.request::<()>(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::None).await?.bytes().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident($($name:ident: $ty:ty),*) -> Json<$res:ty> { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, $($name: $ty),*) -> ::reqwest::Result<$res> {
            use $crate::Api as _;
            self.request::<()>(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::None).await?.json().await
        }
        api!($($rest)*);
    };
}

/// Magic macro for API structs.
///
/// # Simple Usage (auto generated struct)
/// ```rust
/// use api_client::{api, Api};
/// use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
///
/// api!(pub struct ExampleApi);
///
/// impl ExampleApi {
///     api! {
///         fn example() -> String {
///            GET "https://example.com"
///         }
///     }
/// }
/// ```
///
/// # Advanced Usage (manually created struct and [Api] implementation)
/// ```rust
/// use api_client::{api, Api};
/// use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
///
/// struct ExampleApi {
///     client: ClientWithMiddleware,
///     username: String,
///     password: String
/// }
///
/// impl Api for ExampleApi {
///     fn client(&self) -> &Client {
///         &self.client
///     }
///
///     fn pre_request(&self, request: RequestBuilder) -> reqwest_middleware::Result<RequestBuilder> {
///         Ok(request.basic_auth(&self.username, Some(&self.password)))
///     }
/// }
///
/// impl ExampleApi {
///     api! {
///         fn example() -> String {
///            GET "https://example.com"
///         }
///     }
/// }
/// ```
#[macro_export]
#[cfg(feature = "middleware")]
macro_rules! api {
    () => {};

    ($(#[$attr:meta])* $vis:vis struct $ident:ident) => {
        $(#[$attr])*
        $vis struct $ident(::reqwest_middleware::ClientWithMiddleware);

        impl $crate::Api for $ident {
            fn client(&self) -> &::reqwest_middleware::ClientWithMiddleware {
                &self.0
            }

            fn new() -> Self where Self: Sized {
                $ident(::reqwest_middleware::ClientBuilder::new(::reqwest::Client::new()).build())
            }
        }
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Json<$req:ty>$(, $name:ident: $ty:ty)*) -> StatusCode { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name $ty),*) -> ::reqwest_middleware::Result<::reqwest::StatusCode> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Json(request)).await.map(|res| res.status())
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Json<$req:ty>$(, $name:ident: $ty:ty)*) -> String { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest_middleware::Result<String> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Json(request)).await?.text().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Json<$req:ty>$(, $name:ident: $ty:ty)*) -> Bytes { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest_middleware::Result<::bytes::Bytes> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Json(request)).await?.bytes().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Json<$req:ty>$(, $name:ident: $ty:ty)*) -> Json<$res:ty> { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest_middleware::Result<$res> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Json(request)).await?.json().await.map_err(reqwest_middleware::Error::from)
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Form<$req:ty>$(, $name:ident: $ty:ty)*) -> StatusCode { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest_middleware::Result<::reqwest::StatusCode> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Form(request)).await.map(|res| res.status())
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Form<$req:ty>$(, $name:ident: $ty:ty)*) -> String { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest_middleware::Result<String> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Form(request)).await?.text().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Form<$req:ty>$(, $name:ident: $ty:ty)*) -> Bytes { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest_middleware::Result<::bytes::Bytes> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Form(request)).await?.bytes().await
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident(request: Form<$req:ty>$(, $name:ident: $ty:ty)*) -> Json<$res:ty> { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, request: &$req, $($name: $ty),*) -> ::reqwest_middleware::Result<$res> {
            use $crate::Api as _;
            self.request(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::Form(request)).await?.json().await.map_err(reqwest_middleware::Error::from)
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident($($name:ident: $ty:ty),*) -> StatusCode { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, $($name: $ty),*) -> ::reqwest_middleware::Result<::reqwest::StatusCode> {
            use $crate::Api as _;
            self.request::<()>(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::None).await.map(|res| res.status())
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident($($name:ident: $ty:ty),*) -> String { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, $($name: $ty),*) -> ::reqwest_middleware::Result<String> {
            use $crate::Api as _;
            self.request::<()>(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::None).await?.text().await.map_err(reqwest_middleware::Error::from)
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident($($name:ident: $ty:ty),*) -> Bytes { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, $($name: $ty),*) -> ::reqwest_middleware::Result<::bytes::Bytes> {
            use $crate::Api as _;
            self.request::<()>(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::None).await?.bytes().await.map_err(reqwest_middleware::Error::from)
        }
        api!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis fn $ident:ident($($name:ident: $ty:ty),*) -> Json<$res:ty> { $method:tt $url:literal } $($rest:tt)*) => {
        $(#[$attr])*
        #[inline]
        $vis async fn $ident(&mut self, $($name: $ty),*) -> ::reqwest_middleware::Result<$res> {
            use $crate::Api as _;
            self.request::<()>(::reqwest::Method::$method, format!($url).as_str(), $crate::Body::None).await?.json().await.map_err(reqwest_middleware::Error::from)
        }
        api!($($rest)*);
    };
}

#[cfg(test)]
mod tests {
    #![allow(unused)]

    use example::{CreateTodo, JsonPlaceholder, Todo, UpdateTodo};

    mod example {
        use crate::{api, Api};

        pub use models::*;

        mod models {
            use serde::{Deserialize, Serialize};

            #[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
            pub struct Todo {
                #[serde(rename = "userId")]
                pub user_id: u32,
                pub id: u32,
                pub title: String,
                pub completed: bool,
            }

            #[derive(Debug, Serialize)]
            pub struct CreateTodo {
                #[serde(rename = "userId")]
                pub user_id: u32,
                pub title: String,
                pub completed: bool,
            }

            #[derive(Debug, Default, Serialize)]
            pub struct UpdateTodo {
                #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
                pub user_id: Option<u32>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub title: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub completed: Option<bool>,
            }
        }

        api!(pub struct JsonPlaceholder);

        const BASE_URL: &str = "https://jsonplaceholder.typicode.com";

        impl JsonPlaceholder {
            pub fn new() -> Self {
                Api::new()
            }

            api! {
                pub fn todos() -> Json<Vec<Todo>> {
                    GET "{BASE_URL}/todos"
                }

                pub fn todo(id: u32) -> Json<Todo> {
                    GET "{BASE_URL}/todos/{id}"
                }

                pub fn create_todo(request: Json<CreateTodo>) -> Json<Todo> {
                    POST "{BASE_URL}/todos"
                }

                pub fn replace_todo(request: Json<Todo>, id: u32) -> Json<Todo> {
                    PUT "{BASE_URL}/todos/{id}"
                }

                pub fn update_todo(request: Json<UpdateTodo>, id: u32) -> Json<Todo> {
                    PATCH "{BASE_URL}/todos/{id}"
                }

                pub fn delete_todo(id: u32) -> StatusCode {
                    DELETE "{BASE_URL}/todos/{id}"
                }
            }
        }
    }

    #[test]
    fn json_placeholder() {
        tokio_test::block_on(async {
            let mut api = JsonPlaceholder::new();

            let all_todos = api.todos().await.unwrap();
            let todo_1 = api.todo(1).await.unwrap();
            assert_eq!(&all_todos[0], &todo_1);

            let new_todo = api
                .create_todo(&CreateTodo {
                    user_id: 1,
                    title: "test".to_string(),
                    completed: false,
                })
                .await
                .unwrap();
            assert_eq!(new_todo.id as usize, all_todos.len() + 1);

            let replaced_todo = api
                .replace_todo(
                    &Todo {
                        title: "test".to_string(),
                        completed: true,
                        ..todo_1
                    },
                    1,
                )
                .await
                .unwrap();
            assert_eq!(replaced_todo.title, "test");
            assert!(replaced_todo.completed);

            let updated_todo = api
                .update_todo(
                    &UpdateTodo {
                        title: Some("test".to_string()),
                        completed: Some(true),
                        ..Default::default()
                    },
                    1,
                )
                .await
                .unwrap();
            assert_eq!(updated_todo.title, "test");
            assert!(updated_todo.completed);

            assert!(api.delete_todo(1).await.unwrap().is_success());
        });
    }
}
