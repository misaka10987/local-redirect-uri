# local-redirect-uri

A simple HTTP server to provide a local redirect URI for OAuth2 and get the authorization code.

## Usage

```rust
// redirect_uri = "http://localhost:8080"
let redirect = local_redirect_uri::Server::new(8080, "<CSRF_TOKEN>".into());

// user opens authorization URL in the browser and proceeds OAuth2 flow
// ...

let code = redirect.wait_code().await?;
```

## Dependency

Note that a [tokio](https://tokio.rs/) runtime is required to run the server.
