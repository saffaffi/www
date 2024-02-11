# index page??

Here's my own handler function!

```rust
pub async fn index(state: AppState) -> Markup {
    let raw_content = fs::read_to_string("saffi-wtf/content/_index.md").unwrap();

    let syntect_adapter = SyntectAdapter::new(None);

    let plugins = {
        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&syntect_adapter);
        plugins
    };
    let options = ComrakOptions::default();
    let html_content = markdown_to_html_with_plugins(&raw_content, &options, &plugins);

    wrappers::base(state, PreEscaped(html_content)).await
}
```

And here's some Nix stuff:

```nix
{
  packages = rec {
    saffi = (rustPkgs.workspace.www-saffi { }).out;
    saffi-dev = (rustPkgs.workspace.www-saffi-dev { }).bin;
    saffi-wtf = (rustPkgs.workspace.www-saffi-wtf { }).bin;
  };
}
```
