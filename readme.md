# Axum tools

## Axum minijinja

**axum-minijinja** is a wrapper around the [minijijnja](https://github.com/mitsuhiko/minijinja) crate.

It offers several functionalities:

- Providing `View` extractor for response: `view.render(name)`
- Accessing **minijinja** under the hood: `view.env.get_template(name)`
- Rendering of a specific block from a template: `template_name#block_name.html`
- Automatic reloading for the development profile using **minijinja-autoreload**

### Usage

```toml
# Cargo.toml
[dependencies]
axum-minijinja = { git = "https://github.com/borolgs/axum-tools.git" }
```

```rs
// main.rs
use axum_minijinja::{minijinja::Environment, ViewEngine, View};

#[tokio::main]
async fn main() -> Result<()> {

    let env = Environment::new();
    let engine = ViewEngine::from_dir(env, "views");

    let state = AppState {
        engine,
    };

    let app = ApiRouter::new()
        .route("/", index)
        .with_state(state);
}

async fn index(view: View) -> impl IntoResponse {
    view.response("index.html", json!({}))
}
```

### Render block

```html
<!-- some template -->
{% extends "page.html" %} {% block main %}
<div>
  <h1>Hello</h1>
</div>
{% endblock %}
```

```rs
async fn block(view: View) -> impl IntoResponse {
    view.response("index.html#main", json!({})) // -> <div><h1>Hello</h1></div>
}
```
