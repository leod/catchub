# Secret Project

## Dependencies
- `npm` (only needed for the `rollup` packer):
    Installation on Ubuntu (otherwise, follow https://github.com/nodesource/distributions/blob/master/README.md)

    ```
    curl -sL https://deb.nodesource.com/setup_14.x | sudo -E bash -
    sudo apt-get install -y nodejs
    ```
- `wasm-bindgen` to generate JavaScript/Wasm bindings:

    ```
    rustup target add wasm32-unknown-unknown
    cargo install wasm-bindgen-cli
    ```
- `rollup` to combine JavaScript files into a single file:

    ```
    sudo npm install --global rollup
    ```

## Running

Build the client:
```
cargo build --target wasm32-unknown-unknown
```
This will generate the following files in `clnt/static/`:
- `clnt_bg.wasm`
- `clnt.js`

Build and run the server:
```
cargo run -j8 --bin serv -- --clnt_dir clnt/static --http_address <your-ip>:8080
```

## Useful resources
- https://dev.to/dandyvica/wasm-in-rust-without-nodejs-2e0c
