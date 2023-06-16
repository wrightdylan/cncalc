# :sparkles: Star Cluster Neighbour Calculator :star2:

This project calculates all combinations and permutations of distances between stars in a cluster. The original was written in Python, which was slow, so this was translated into Rust both for speed, and the ability to handle larger clusters. Preliminary performance tests show a *6,000-fold increase* in speed for the Rust v 0.0.1 port compared to the original Python script. Other than seeing what sort of speed increase I could get by writing this in a compiled language, I also thought this would be good practice in Rust, and I can get acquinted with another DataFrame package. As it turns out, the latter was just a bad idea.

The second version of this project allows me to get accustomed to using workspaces. It also brings in a port of another Python script which was the companion of the first distance calculator script. Due to a quirk in the way I had to build dataframes I couldn't use null values, so nulls were represented with decimal 0.0, which will obviously interfere with finding the minimum distances. Finding the second shortest distance was trivial, but rather than applying it as a typical function I wanted to make it a custom method in a trait to extend the Polars Series struct just to spice things up a little. Actually a weird flex. This second port has a speed increase of only 10x.

Since I'm on Linux, I thought I would also give cross compiling a go.

There was an attempt at parallelisation, but it turns out to run slower compared to a single thread. Maybe it would work better for globular clusters.

## Installation

Build all binaries with the command:
```bash
cargo build --release
```

Binaries are located in `target/release`.

## For Windows Users

For the convenience of non-Linux users, pre-built binaries for Windows systems can be found [here](bin_WinDohs/). The files are astonishingly large, but this is the nature of how Rust compiles. Perhaps that may change in the future when C is fully deprecated.

## Usage

Input files require fields for `designation`, `ra`, `dec`, and `parallax`, and must be in CSV format. This tool is currently specified for GAIA catalogues, so `ra` and `dec` are in **decimal degrees**, and `parallax` is in **mas**.

From your CLI enter:
```bash
cncalc <FILENAME>
```

Once it is done generating a table it will spit out a binary Parquet file. This is much smaller than a CSV file which only a masochist would manually read through anyway.

To find minimums and maximums for each star combination, use the command:
```bash
minmax <FILENAME>
```

This will generate a CSV file with all minimums and maximums, and a text summary.

## Features
- Calculates all distances between each and every star within a cluster.
- Summarises distances.
- Calculates Gini coefficient of the star cluster.

## Changelog

Changes logged [here](CHANGELOG.md).

## License

This project is released under the GNU GPL-3.0 license. Check out the [LICENSE](LICENSE) file for more information.
