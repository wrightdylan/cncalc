use minmax::*;
use clap::Parser;
use std::time::SystemTime;

mod args;
use crate::args::CalcArgs;

fn main() {
    let args = CalcArgs::parse();
    let time_start = SystemTime::now();

    let df = read_parquet(&args.filename).unwrap();
    // let lf = scan_parq(path).unwrap();


    // Generate a new dataframe containing all mins and maxes
    let mut df2 = sort_dist(df);

    // Save the DataFrame to CSV
    write_df(&mut df2);

    let lap_time1 = SystemTime::now();
    let time_elapsed = lap_time1.duration_since(time_start).unwrap();

    println!("Task completed in {:?}", time_elapsed);
}