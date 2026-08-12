use cncalc::*;
use clap::Parser;
use std::time::SystemTime;

mod args;
use crate::args::CalcArgs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CalcArgs::parse();
    let time_start = SystemTime::now();

    // Hard coded query for now
    let load_cols = vec![
        "designation".to_string(),
        "ra".to_string(),
        "dec".to_string(),
        "parallax".to_string(),
        "g_abs".to_string()
    ];

    let mut df1 = load_data(&args.filename, load_cols)?;
    calc_rho(&mut df1)?;

    let lap_time1 = SystemTime::now();
    let time_elapsed = lap_time1.duration_since(time_start).unwrap();

    // Calculate the Gini coefficient of the star cluster
    // let gini = gini_coefficient_lf(&df1)?;
    // println!("Gini coefficient of the star cluster is: {}", gini);

    let source = restructure_data(&df1)?;
    let mut df2 = calc_distances(&df1, &source)?;
    // let mut df2 = calc_distances_lazy(df1)?;

    let lap_time2 = SystemTime::now();
    let time_end = lap_time2.duration_since(lap_time1).unwrap();

    // Save the dataframe to a file
    write_df(&mut df2);

    let lap_time3 = SystemTime::now();
    let time_total = lap_time3.duration_since(time_start).unwrap();

    println!("Radial distances completed in {:?}", time_elapsed);
    println!("Distances calculated in {:?}", time_end);
    println!("Operation completed in {:?}", time_total);

    // Calculate the Gini coefficient of the star cluster
    let gini = gini_coefficient(&source);
    println!("Gini coefficient of the star cluster is: {}", gini);

    Ok(())
}
