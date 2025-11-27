# Baker Rust

Rust implementation of BakerCM.
Based on the original Python BakerCM (https://pypi.org/project/bakercm/). We hit install and environment issues with the PyPI package in some setups, so baker-rust reimplements the same behavior as a single self-contained binary to make installation easier anywhere.

## Why Use This?
- Same recipe and template behavior as BakerCM (INI recipes, `{{ VAR }}` templates, secrets, remote pulls).
- Lightweight: only AES/EAX crypto, HTTP, and JSON dependencies.
- Secrets use the identical `nonce\tag\cipher` hex format, so existing recipes/keys work.
- Remote recipes/templates with caching and optional `Authorization` header.

## Features
- Configure dynamic values in template files per environment.
- Encrypt/decrypt values and store them safely in recipes.
- Copy/move templates to targets with optional permission bits (mode on Unix).
- Manage recipe versions from common VCS-hosted locations via URL patterns.
- Customize Baker settings via `.bakerc`.

## Installation
Requires Rust 1.70+.

```console
cargo build
```

## Packages
Pre-built packages are published for Alpine and Wolfi to simplify installs.

### Alpine
```console
echo "https://arthurhoch.github.io/baker-rust/alpine/baker-rust" | sudo tee -a /etc/apk/repositories
ARCH=$(apk --print-arch)
sudo wget -O /etc/apk/keys/packager.rsa.pub https://arthurhoch.github.io/baker-rust/alpine/baker-rust/$ARCH/packager.rsa.pub
sudo apk update
sudo apk add baker-rust
```

### Wolfi (wolfi-dev)
```console
echo "https://arthurhoch.github.io/baker-rust/wolfi/baker-rust" | sudo tee -a /etc/apk/repositories
ARCH=$(apk --print-arch)
sudo wget -O /etc/apk/keys/melange.rsa.pub https://arthurhoch.github.io/baker-rust/wolfi/baker-rust/$ARCH/melange.rsa.pub
sudo apk update
sudo apk add baker-rust
```

## Using Baker (Rust)
1. Create a recipe such as `examples/dev.cfg`

```ini
[dev:app:template]
template = ./examples/templates/app.conf.tpl
path = ./examples/app.conf
[dev:app:variables]
HOST = dev-host.db
PORT = 9000
USER = dbuser
[dev:app:secrets]
PASSWORD = <encrypted or plain, see below>
```

2. Create the template `examples/templates/app.conf.tpl`

```ini
database:
 engine: 'postgres'
 host: '{{ HOST }}'
 port: '{{ PORT }}'
 user: '{{ USER }}'
 password: '{{ PASSWORD }}'
```

3. Run Baker

```console
cargo run -- run --path examples/dev.cfg
```

4. Done! File configured.

## Commands
- `configs [-a|--all]` — list settings (custom only or all defaults).
- `genkey <keypass>` — generate and store secret key.
- `encrypt [--file recipe] [values...]` — encrypt values or the `:secrets` section of a recipe.
- `pull <path:version> [-f|--force]` — download a recipe by version.
- `recipes [-a|--all]` — list cached recipes.
- `rm <recipe_id>` — remove a cached recipe.
- `run <path:version> | --path <file> [-f|--force]` — apply templates from a recipe; pulls remote templates if needed.
- Global: `--verbose` for debug logging, `-v/--version`, `-h/--help`.

## Secrets
- Generate a key: `cargo run -- genkey myKeyPass`
- Encrypt inline: `cargo run -- encrypt secretValue`
- Encrypt a recipe’s secrets section: `cargo run -- encrypt --file examples/dev.cfg`
- Templates read secrets like normal variables: `password: '{{ PASSWORD }}'`

## File System Operations
- `path` in `[name:template]` controls the output target (copy/rename behavior).
- `mode` (octal) is applied on Unix. `user/group` flags are parsed but not applied on Windows.
- `TEMPLATE_EXT` strips the extension from output (default `tpl`).

## Remote Recipes
Set repository settings in `~/.bakerc`:

```ini
REPOSITORY='https://raw.githubusercontent.com/lucasb/BakerCM/'
REPOSITORY_TYPE='github'   # or 'bitbucket' or 'custom'
REPOSITORY_AUTH='Basic YmFrZXI6YmFrZXJjbQ=='   # optional
REPOSITORY_CUSTOM_PATTERN='%(repository)s/%(path)s.%(ext)s/%(version)s'  # for custom
```

Use `pull` to fetch or `run <path:version>` to pull-and-run.

## Options / Settings
Defaults follow the Python Baker:

```
DEBUG=False
ENCODING=utf-8
RECIPE_CASE_SENSITIVE=False
REPOSITORY=None
REPOSITORY_TYPE=None
REPOSITORY_AUTH=None
REPOSITORY_CUSTOM_PATTERN=None
STORAGE_RECIPE=~/.baker/recipes/
STORAGE_RECIPE_INDEX=~/.baker/index
STORAGE_RECIPE_META=~/.baker/meta
STORAGE_KEY_PATH=~/.baker/baker.key
STORAGE_TEMPLATES=~/.baker/templates/
TEMPLATE_EXT=tpl
```

View them with `cargo run -- configs --all`.

## Examples
- `examples/dev.cfg` — local recipe using `examples/templates/app.conf.tpl`.

## Development
- Tests: `cargo test`
- Build: `cargo build`
- Logging: add `--verbose` to any command.

## Notes
- Remote template download supports caching and `Authorization` header (same behavior as Python Baker).
- Recipe parsing, template replacement, secret format, and command semantics mirror the original tool for compatibility.
