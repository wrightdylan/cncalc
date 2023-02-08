use cncalc::*;
use clap::Parser;
use std::time::SystemTime;

mod args;
use crate::args::CalcArgs;

fn main() {
    let args = CalcArgs::parse();
    let time_start = SystemTime::now();

    // Hard coded query for now
    let load_cols = vec![
        "designation".to_string(),
        "ra".to_string(),
        "dec".to_string(),
        "parallax".to_string(),
    ];
    let mut df1 = load_data(&args.filename, load_cols).unwrap();

    df1 = calc_rho(&mut df1);

    let lap_time1 = SystemTime::now();
    let time_elapsed = lap_time1.duration_since(time_start).unwrap();

    // Using Polars has proven to be an such an extreme pain in the arse.
    // It's imperative to use indexing, which Polars seems unable to do even though docs say you can.
    // Examples in blogs and official docs do not work, and some functions listed don't work either.
    // Best to just restructure.
    let source = restructure_data(&df1);

    // Generate a new dataframe containing all distances
    let mut df2 = calc_distances(df1, source);

    let lap_time2 = SystemTime::now();
    let time_end = lap_time2.duration_since(lap_time1).unwrap();

    // Save the dataframe to a file
    write_df(&mut df2);

    let lap_time3 = SystemTime::now();
    let time_total = lap_time3.duration_since(time_start).unwrap();

    println!("Radial distances completed in {:?}", time_elapsed);
    println!("Distances calculated in {:?}", time_end);
    println!("Operation completed in {:?}", time_total);
}
