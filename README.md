# :sparkles: Star Custer Calculator :star2:

This project calculates all combinations and permutations of distances between stars in a cluster. The original was written in Python, which was slow, so this was translated into Rust both for speed, and the ability to handle larger clusters. Preliminary performance tests show a *6,000-fold increase* in speed for the Rust v 0.0.1 port compared to the original Python script.

## Usage

Input files require fields for `designation`, `ra`, `dec`, and `parallax`, and must be in CSV format. This tool is currently specified for GAIA catalogues, so `ra` and `dec` are in **decimal degrees**, and `parallax` is in **mas**.

From your CLI enter:
```bash
cncalc <FILENAME>
```

Once it is done generating a table it will spit out a binary Parquet file. This is much smaller than a CSV file which only a masochist would manually read through anyway.

## License

This project is released under the GNU GPL-3.0 license. Check out the [LICENSE](LICENSE) file for more information.
