# ZipDiff

A differential fuzzer for ZIP parsers.

This is the source code for the USENIX Security '25 paper *My ZIP isn’t your ZIP: Identifying and Exploiting Semantic Gaps Between ZIP Parsers*.

Permanent link and Docker image files: https://doi.org/10.5281/zenodo.15526863

## Environment

-   Linux
-   [Rust](https://www.rust-lang.org/tools/install) (tested on 1.86, any version is fine as long as the code compiles successfully)
-   Docker and [Docker Compose](https://docs.docker.com/compose/install/)
-   The evaluation process is resource-intensive, as it runs many ZIP parsers in parallel. It is recommended to have at least 128 GB of RAM and 300 GB of disk space. While it can also run on systems with fewer RAM, you may encounter significant performance degration, primarily due to uncached disk I/O, since the unzipped outputs can be quite large.

## Preparation

Build ZIP parser Docker images:

```console
cd parsers
./prepare.sh
sudo docker compose build
```

Alternatively, if you want to save some time or make sure the versions match the evaluation in the paper, you can load the images from files:

```console
for i in *.tar.bz2; do
    docker load -i "$i"
done
```

Build the fuzzer:

```console
cd zip-diff
cargo build --release
```

## Running the Fuzzer

```console
cd zip-diff
# clear samples and results from previous evaluations
sudo rm -rf ../evaluation
sudo target/release/fuzz
```

Here root permission is required because the outputs are written inside Docker and are owned by root. Sometimes the outputs have incorrect permission bits and cannot be read by regular users even if the user is the file owner.

You can use command-line options to set output locations and other parameters:

```console
cargo run --release -- --help
# or target/release/fuzz --help
```
