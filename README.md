# aihk-prototype-1

This is a frontend & backend for AIxHK 2023 written in Rust.

## Structure

### The Cargo.toml at the root directory describes a Cargo workspace with the sub-crates `common`, `frontend`, and `server`.
---

`common` contains common types that are shared between `frontend` and `server`, that can be compiled in WASM and non-WASM environments.

`frontend` is the frontend written with the Yew framework. It includes utility functions only the frontend uses and the frontend itself.

`server` is the backend. It has api endpoints that facilitate creating accounts, creating posts, etc. Only it interacts with the database.  

## API Endpoints

### `/api/get_posts`
Only accepts GET requests. 

Returns a `(StatusCode, Json<Option<Vec<Post>>>)`. Response body will be `None` when no posts are found in database.

### `/api/submit_post`
Only accepts POST requests. 

Requires a String request body, and a valid session id as a bearer authentication header.

Returns a `(StatusCode, String)`. Response body will contain either a success message or error message.

### `/api/add_comment`
Only accepts POST requests.

Requires a valid `Json<InputComment>` in request body, and a valid session id as a bearer authentication header.

Returns a `(StatusCode, String)`. Response body will contain either a success message or error message.

### `/api/create_account`
Only accepts POST requests.

Requires a valid `Json<User>` in request body.

Returns a `(StatusCode, Json<Option<String>>)`. Response body will be `None` when storing new user fails somehow. Otherwise, the response body will be a new session id.

### `/api/login`
Only accepts POST requests.

Requires a valid `Json<User>` in request body.

Returns a `(StatusCode, Json<Option<String>>)`. Response body will be `None` when the user doesn't exist or password does not match. Otherwise, the response body will be a new session id.

### `/api/validate_session`
Only accepts GET requests.

Requires a valid session id as a bearer authentication header.

Returns a `(StatusCode, Json<Option<String>>)`. Response body will be `None` when session is invalid, and will be the username of the session id if valid.

---

To see some documentation, open the `/doc/common/index.html`, `/doc/frontend/index.html`, and `/doc/server/index.html` files respectively for each crate with a web browser.

To see more, look through the source `.rs` files.

## Compilation

This project compiles with the latest version of Rust.

To compile the frontend, in the frontend directory, run: `CARGO_TARGET_DIR=../target-trunk trunk build --release --public-url /`

To compile the server, at the root directory, run: `cargo build --bin server -r`

# Where can I see the website?

Go to the website here! https://aihk1.trevrosa.dev

# Usage

Run the dev version (auto-reloads server & client on file change) with `./dev.sh`.

Run the pre-compiled version with `./prod.sh`.

The app will start at http://localhost:8080 by default. You can modify that by changing the flags passed to the server binary:

```
Usage: server [OPTIONS]

Options:
  -l, --log <LOG_LEVEL>          set the log level [default: debug]
  -p, --port <PORT>              set the listen port [default: 8080]
      --static-dir <STATIC_DIR>  set the directory where static files are to be found [default: ../dist]
  -h, --help                     Print help
```
