# Yew Ethers D3
Simpleton mainnet ethereum explorer.

![image](https://user-images.githubusercontent.com/18542003/191859599-d389bbb2-8795-4c3a-8418-4d3f84726e9e.png)


TOPLAY
//      fetch previous data
//      add other networks (?)
//      make variable in plot selectable (handle different types)
//      cache?
//      add filters (listeners & for queries (tokens, NFTs (ethers has a module for it)))      
//      download csv (what data & how many tx's)
//      play with stream (instead of interval) [to continue..]


### Installation

If you don't already have it installed, it's time to install Rust: <https://www.rust-lang.org/tools/install>.
The rest of this guide assumes a typical Rust installation which contains both `rustup` and Cargo.

To compile Rust to WASM, we need to have the `wasm32-unknown-unknown` target installed.
If you don't already have it, install it with the following command:

```bash
rustup target add wasm32-unknown-unknown
```

Now that we have our basics covered, it's time to install the star of the show: [Trunk].
Simply run the following command to install it:

```bash
cargo install trunk wasm-bindgen-cli
```

That's it, we're done!

### Running

You need to create an .env file with the following information
WSS_KEY_MAINNET = "Your WSS mainnet key"

// it does not have to be infura

```bash
trunk serve
```

Rebuilds the app whenever a change is detected and runs a local server to host it.

There's also the `trunk watch` command which does the same thing but without hosting it.

### Release

```bash
trunk build --release
```

This builds the app in release mode similar to `cargo build --release`.
You can also pass the `--release` flag to `trunk serve` if you need to get every last drop of performance.

Unless overwritten, the output will be located in the `dist` directory.

## Using this template

There are a few things you have to adjust when adopting this template.

### Remove example code

The code in [src/main.rs](src/main.rs) specific to the example is limited to only the `view` method.
There is, however, a fair bit of Sass in [index.scss](index.scss) you can remove.

### Update metadata

Update the `name`, `version`, `description` and `repository` fields in the [Cargo.toml](Cargo.toml) file.
The [index.html](index.html) file also contains a `<title>` tag that needs updating.

Finally, you should update this very `README` file to be about your app.

### License
This example was made with Trunk templates.

The template ships with both the Apache and MIT license.
If you don't want to have your app dual licensed, just remove one (or both) of the files and update the `license` field in `Cargo.toml`.

There are two empty spaces in the MIT license you need to fill out: `{{year}}` and `{{authors}}`.

[trunk]: https://github.com/thedodd/trunk
