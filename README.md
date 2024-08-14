# Corrosion Web Server

This is a simple web server implemented in Rust. It listens on `127.0.0.1:8080` and serves a specified file in response to HTTP GET requests.

## Features

- Multi-threaded request handling using a thread pool
- Serves a specified file for HTTP GET requests
- Returns a 404 Not Found response for other requests

## Usage

### Installing and running the Server

    git clone https://github.com/Mandrew0822/corrosion
    cd corosion
    chmod +x build
    build
    
### Example

To serve an `index.html` file:

you can find example html files in '/demos'
```sh
corrosion index.html
