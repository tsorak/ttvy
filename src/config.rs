use std::{env, path::PathBuf, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::{fs, sync::Mutex};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
    pub channel: Option<String>,
    pub oauth: Option<String>,
    pub nick: Option<String>,
}

impl Config {
    pub async fn new() -> Self {
        let mut save_dir = env::var("HOME").expect("Failed to get HOME");
        save_dir.push_str("/.ttvy/state.json");
        let save_dir = PathBuf::from_str(&save_dir).unwrap();

        match fs::read_to_string(&save_dir).await {
            Ok(c) => serde_json::from_str(&c).expect("Bad config"),
            Err(_) => Self {
                channel: None,
                oauth: None,
                nick: None,
            },
        }
    }

    pub async fn init(&mut self) -> &mut Self {
        let args: Vec<String> = env::args().collect();

        if let Some(channel) = args.get(1).cloned() {
            self.channel = Some(channel);
        }
        self.oauth.get_or_insert(http::get_ttv_token().await);
        // self.ttv_nick = None;

        self
    }

    pub async fn set_initial_channel(config: &Arc<Mutex<Self>>) {
        let config = config.clone();
        let args: Vec<String> = env::args().collect();

        if let Some(initial_channel) = args.get(1) {
            let mut c = config.lock().await;
            let _ = c.channel.insert(initial_channel.clone());
        }
    }

    pub fn fetch_auth_token(config: &Arc<Mutex<Self>>) {
        let config = config.clone();
        tokio::spawn(async move {
            let mut c = config.lock().await;
            let _ = c.oauth.insert(http::get_ttv_token().await);
        });
    }
}

mod http {
    use std::sync::Arc;

    use axum::{
        http::{header, HeaderValue},
        response::IntoResponse,
        routing::{get, post},
        Extension, Json, Router,
    };

    use tokio::{
        fs::File,
        io::AsyncReadExt,
        process::Command,
        sync::mpsc::{channel, Receiver, Sender},
        task::JoinHandle,
    };

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    struct TokenBody {
        pub token: String,
    }

    pub async fn get_ttv_token() -> String {
        let api_url: String = "https://id.twitch.tv/oauth2/authorize?\
            response_type=token\
            &client_id=m0y30jcckwn2a7m7hh0djrg47wvbuk\
            &scope=chat%3Aread%20chat%3Aedit\
            &redirect_uri=http://localhost:4537"
            .to_string();

        if Command::new("open").arg(&api_url).spawn().is_err() {
            println!("Complete authentication at\n{api_url}");
        }

        println!("Waiting for token...");
        let (token_tx, mut token_rx) = channel::<String>(1);
        let (shutdown_tx, shutdown_rx) = channel::<()>(1);

        let _handle = start_webserver(token_tx, shutdown_rx);

        let msg = token_rx.recv().await.unwrap();
        shutdown_tx.send(()).await.unwrap();
        msg
    }

    fn start_webserver(token_tx: Sender<String>, mut shutdown_rx: Receiver<()>) -> JoinHandle<()> {
        let state = Arc::new(token_tx);
        tokio::spawn(async move {
            // build our application with a single route
            let app = Router::new()
                .route("/token", post(handle_token_route))
                .route("/", get(serve_index))
                .route("/script.js", get(serve_script))
                .layer(Extension(state));

            let listener = tokio::net::TcpListener::bind("0.0.0.0:4537").await.unwrap();

            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    shutdown_rx.recv().await;
                })
                .await
                .unwrap()
        })
    }

    async fn serve_static_file(path: &str) -> String {
        let mut file = File::open(path).await.unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).await.unwrap();
        contents
    }

    async fn serve_index() -> impl IntoResponse {
        let mut res = serve_static_file("web/index.html").await.into_response();

        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str("text/html").unwrap(),
        );

        res
    }

    async fn serve_script() -> impl IntoResponse {
        let mut res = serve_static_file("web/script.js").await.into_response();

        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str("application/javascript").unwrap(),
        );

        res
    }

    async fn handle_token_route(
        state: Extension<Arc<Sender<String>>>,
        Json(payload): Json<TokenBody>,
    ) {
        state.0.send(payload.token).await.unwrap();
        "OK".to_string();
    }
}
