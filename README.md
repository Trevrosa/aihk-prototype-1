## Usage

Ensure .apikey is not empty and exists in the root directory of the repository.

Run the dev version (auto-reloads server & client on file change) with `./dev.sh`.

Run the pre-compiled version with `./prod.sh`.

The app will start at http://localhost:8082 by default. You can modify that by changing the flags passed to the server binary:

```
Usage: server [OPTIONS]

Options:
  -l, --log <LOG_LEVEL>          set the log level [default: debug]
  -p, --port <PORT>              set the listen port [default: 8080]
      --static-dir <STATIC_DIR>  set the directory where static files are to be found [default: ../dist]
  -h, --help                     Print help
```
