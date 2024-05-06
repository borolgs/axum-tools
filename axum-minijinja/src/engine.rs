use std::{convert::Infallible, sync::Arc};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{Html, IntoResponse, Response}
};
use minijinja::{path_loader, Environment, Error};
use minijinja_autoreload::AutoReloader;
use tokio::sync::OnceCell;

static TEMPLATE_WATCHER: OnceCell<AutoReloader> = OnceCell::const_new();

#[derive(Debug, Clone)]
pub struct ViewEngine {
    pub env: Arc<Environment<'static>>,
}

impl ViewEngine {
    pub fn new(env: Environment<'static>) -> Self {
        let env = Arc::new(env);
        Self { env }
    }

    pub fn from_dir(mut env: Environment<'static>, templates_dir: &'static str) -> Self {
        env.set_loader(path_loader(templates_dir));

        if cfg!(debug_assertions) {
            init_reloader(templates_dir);
        }

        Self::new(env)
    }
}

impl ViewEngine {
    pub fn response<D: serde::Serialize>(&self, key: &str, data: D) -> Response {
        match self.render(key.as_ref(), data) {
            Ok(x) => Html(x).into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }

    #[cfg(debug_assertions)]
    fn render<D: serde::Serialize>(&self, key: &str, data: D) -> Result<String, Error> {
        let reloader = TEMPLATE_WATCHER.get().unwrap();
        let env = reloader.acquire_env()?;

        if key.contains('#') {
            let parts = key.split_once("#");
            let template_name = parts.map(|p| p.0);
            let block_name = parts.map(|p| p.1);

            if let (Some(template_name), Some(block_name)) = (template_name, block_name) {
                let template = env.get_template(template_name)?;
                let rendered = template.eval_to_state(&data)?.render_block(&block_name)?;

                return Ok(rendered);
            }
        }

        let template = env.get_template(key)?;
        let rendered = template.render(&data)?;

        Ok(rendered)
    }

    #[cfg(not(debug_assertions))]
    fn render<D: serde::Serialize>(&self, key: &str, data: D) -> Result<String, Error> {
        if key.contains('#') {
            let parts = key.split("#").collect::<Vec<&str>>();
            let template_name = parts.first();
            let block_name = parts.last();

            if let (Some(template_name), Some(block_name)) = (template_name, block_name) {
                let template = self.env.get_template(template_name)?;
                let rendered = template.eval_to_state(&data)?.render_block(&block_name)?;

                return Ok(rendered);
            }
        }

        let template = self.env.get_template(key)?;
        let rendered = template.render(&data)?;

        Ok(rendered)
    }
}

#[async_trait]
impl<ApplicationState> FromRequestParts<ApplicationState> for ViewEngine
where
    Self: FromRef<ApplicationState>,
    ApplicationState: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        _: &mut Parts,
        state: &ApplicationState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}

pub type View = ViewEngine;

fn init_reloader(template_path: &'static str) {
    let reloader = AutoReloader::new(move |notifier| {
        let mut env = Environment::new();
        notifier.watch_path(template_path, true);
        env.set_loader(path_loader(template_path));
        Ok(env)
    });
    _ = TEMPLATE_WATCHER.set(reloader);
}
