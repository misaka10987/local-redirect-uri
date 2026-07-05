use anyhow::anyhow;
use axum::{Form, Router, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::broadcast::{self, Receiver, Sender},
};
use tracing::error;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Data {
    pub code: String,
    pub state: String,
}

/// A simple HTTP server for receiving the authorization code from OAuth2 redirect.
///
/// # Example
///
/// ``` rust
/// // redirect_uri = "http://localhost:8080"
/// let redirect = local_redirect_uri::Server::new(8080, "<CSRF_TOKEN>".into());
///
/// // user opens authorization URL in the browser and proceeds OAuth2 flow
/// // ...
///
/// let code = redirect.wait_code().await?;
/// ```
pub struct Server {
    pub port: u16,
    pub csrf_token: String,
    send: Sender<Result<String, String>>,
    recv: Receiver<Result<String, String>>,
}

impl Server {
    /// Create a new server.
    ///
    /// The server does nothing until [`Self::wait_code`] is called.
    pub fn new(port: u16, csrf_token: String) -> Self {
        let (send, recv) = broadcast::channel(1);

        Self {
            port,
            csrf_token,
            send,
            recv,
        }
    }

    /// Wait for the authorization code from the redirect.
    ///
    /// This function will internally check the CSRF token before return and fails if it mismatches.
    pub async fn wait_code(self) -> anyhow::Result<String> {
        let tcp = TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;

        let send = self.send.clone();

        let app = Router::new().route(
            "/",
            get(async move |Form(data): Form<Data>| {
                if data.state != self.csrf_token {
                    error!("blocking request with invalid CSRF token");

                    send.send(Err("CSRF protection".into())).unwrap();

                    return (
                        StatusCode::FORBIDDEN,
                        "Authorization failed. Redirection is blocked by Cross-Site Request Forgery (CSRF) protection.",
                    );
                }

                 send.send(Ok(data.code)).unwrap();

                (StatusCode::OK, "Authorization successful. You can close this window now.")
            }),
        );

        let mut recv = self.send.subscribe();

        axum::serve(tcp, app)
            .with_graceful_shutdown(async move {
                if let Err(e) = recv.recv().await {
                    error!("{e}");
                }
            })
            .await?;

        let mut recv = self.recv;

        let data = recv.recv().await?.map_err(|e| anyhow!("{e}"))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
