# Derpisync
## About
**Derpisync** is intended to synchronize tags of images from *derpibooru* with your localy stored images (using **tmsu** as local tagging backend).
## Usage
To use **derpisync** you need **tmsu** to be installed.
After tmsu is installed and it's database initialized with `tmsu init` you can pipe desired filenames to be synced into `derpisync`
```bash
cd dir-with-your-images
tmsu init
ls | derpisync
```
or
```bash
find -type f | derpisync
```
`derpisync` will create **.derpisync-index** file in working directory to keep track for files which already have synced tags.
> Note: It always uses **.derpisync-index** in working directory therefore it's reccomended to always launch it from the same dir to avoid duplicating info
## Build and installation
```bash
git clone https://github.com/shybu8/derpisync.git
cd derpisync
cargo build --release
sudo cp target/release/derpisync /usr/local/bin
```

