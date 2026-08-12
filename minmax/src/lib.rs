use polars::prelude::*;

pub fn read_parquet(path: &str) -> PolarsResult<DataFrame> {
    let mut file = std::fs::File::open(path).unwrap();
    ParquetReader::new(&mut file).finish()
}

pub fn scan_parq(path: &str) -> PolarsResult<LazyFrame> {
    let args = ScanArgsParquet::default();
    
    LazyFrame::scan_parquet(PlRefPath::new(path), args)
}

pub fn vectorise_designations(df: &DataFrame) -> PolarsResult<Vec<String>> {
    let designation_col = df.column("designation")?;
    let designation_series = designation_col.as_materialized_series();
    let str_chunked = designation_series.str()?;

    let des_vec: Vec<String> = str_chunked
        .iter() 
        .map(|opt_s| {
            let s = opt_s.unwrap_or("");
            let newline: Vec<&str> = s.split('"').collect();
            newline.get(1).unwrap_or(&"").to_string()
        })
        .collect();

    Ok(des_vec)
}

pub fn sort_dist(df: DataFrame) -> PolarsResult<DataFrame> {
    let mut df2 = df
        .clone()
        .lazy()
        .select([col(PlSmallStr::from("designation"))])
        .collect()?;

    let size = df.height();

    let mut min_val: Vec<f64> = Vec::new();
    let mut max_val: Vec<f64> = Vec::new();
    let mut avg_val: Vec<f64> = Vec::new();

    for col in df.columns().iter().skip(1) {
        let series = col.as_materialized_series();
        let chunked_array = series.f64()?;
        
        let nz_min = chunked_array
            .iter()
            .flatten()
            .filter(|&val| val > 0.0)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        let max = series.max::<f64>()?.unwrap_or(0.0);
        let sum = series.sum::<f64>()?;

        min_val.push(nz_min);
        max_val.push(max);
        avg_val.push(sum / (size as f64));
    }

    df2.with_column(Column::from(Series::new(PlSmallStr::from("Min"), min_val)))?;
    df2.with_column(Column::from(Series::new(PlSmallStr::from("Max"), max_val)))?;
    df2.with_column(Column::from(Series::new(PlSmallStr::from("Avg"), avg_val)))?;

    let get_stat = |df: &DataFrame, name: &str, stat_type: &str| -> f64 {
        df.column(name)
            .map(|c| c.as_materialized_series().clone())
            .and_then(|s| {
                let chunked = s.f64()?;
                match stat_type {
                    "nz_min" => Ok(chunked.iter().flatten().filter(|&v| v > 0.0).min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0.0)),
                    "min" => s.min::<f64>().map(|o| o.unwrap_or(0.0)),
                    "max" => s.max::<f64>().map(|o| o.unwrap_or(0.0)),
                    "sum" => s.sum::<f64>(),
                    _ => Ok(0.0),
                }
            })
            .unwrap_or(0.0)
    };

    println!("Shortest distance to closest neighbour is {:?} pc", get_stat(&df2, "Min", "nz_min"));
    println!("Longest distance to closest neighbour is {:?} pc", get_stat(&df2, "Min", "max"));
    println!("Average distance to closest neighbour is {:?} pc", get_stat(&df2, "Min", "sum") / (size as f64));
    println!("Longest distance between stars is {:?} pc", get_stat(&df2, "Max", "max"));
    println!("Average distance between stars is {:?} pc", get_stat(&df2, "Avg", "sum") / (size as f64));

    Ok(df2)
}

pub fn write_df(df: &mut DataFrame) {
    let mut file_out = std::fs::File::create("MinMaxPairs.csv").unwrap();
    CsvWriter::new(&mut file_out).finish(df).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_second_lowest() -> PolarsResult<()> {
        let s = Series::new(PlSmallStr::from("days"), [0.0, 0.1, 0.2, 0.3].as_ref());

        let chunked_array = s.f64()?;
        
        let nz_min = chunked_array
            .iter()
            .flatten()
            .filter(|&val| val > 0.0)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        assert_eq!(nz_min, Some(0.1));
        
        Ok(())
    }
}
