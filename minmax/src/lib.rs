#![allow(dead_code)]

use polars::{prelude::*, export::num::NumCast};

// Special method for finding minimum ignoring zeroes
pub trait MinMax {
    fn nz_min<T: NumCast>(&self) -> Option<T>;
}

impl MinMax for Series {
    /// Returns the next (non-zero) minimum value in the array, according to the natural order.
    /// Returns an option because the array is nullable.
    /// 
    /// # Example
    /// 
    /// ```
    /// # use polars::prelude::*;
    /// # use minmax::MinMax;
    /// let s = Series::new("days", [0.0, 0.1, 0.2, 0.3].as_ref());
    /// assert_eq!(s.nz_min(), Some(0.1));
    /// ```
    fn nz_min<T>(&self) -> Option<T>
    where
        T: NumCast
    {
        self.f64()
            .unwrap()
            .sort(false)
            .get(1).and_then(T::from)
    }
    /*{
        self.min_as_series()
            .cast(&DataType::Float64)
            .ok()
            .and_then(|s| s.f64().unwrap().get(0).and_then(T::from))
    }*/
}

pub fn read_parquet(path: &str) -> PolarsResult<DataFrame> {
    let mut file = std::fs::File::open(path).unwrap();
    ParquetReader::new(&mut file).finish()
}

pub fn scan_parq(path: &str) -> PolarsResult<LazyFrame> {
    let args = ScanArgsParquet::default();
    LazyFrame::scan_parquet(path, args)
}

fn non_zero_minimum(s: &Series) -> f64 {
    let sorted = s
        .f64()
        .unwrap()
        .sort(false);
    sorted.get(1).unwrap()
}

pub fn vectorise_designations(df: &DataFrame) -> Vec<String> {
    let messy_desc = df.column("designation").unwrap()
        .iter().map(|s| s.to_string()).collect::<Vec<_>>();

    let mut des_vec: Vec<String> = Vec::new();
    for line in messy_desc {
        let newline = line.split('"').collect::<Vec<_>>();
        des_vec.push(newline[1].to_string());
    }

    des_vec
}

pub fn sort_dist(df: DataFrame) -> DataFrame {
    let mut df2 = df
        .clone()
        .lazy()
        .select([cols(["designation"])])
        .collect()
        .unwrap();

    let size = df.height();

    // let mut table = Array2::<f64>::zeros((3, size));
    let mut min_val: Vec<f64> = Vec::new();
    let mut max_val: Vec<f64> = Vec::new();
    let mut avg_val: Vec<f64> = Vec::new();

    for col in df.get_columns().iter().skip(1) {
        min_val.push(col.nz_min::<f64>().unwrap());
        max_val.push(col.max::<f64>().unwrap());
        avg_val.push(col.sum::<f64>().unwrap()/(size as f64));
    }

    df2.with_column(Series::new("Min", min_val)).unwrap();
    df2.with_column(Series::new("Max", max_val)).unwrap();
    df2.with_column(Series::new("Avg", avg_val)).unwrap();

    println!("Shortest distance to closest neighbour is {:?} pc", df2.column("Min").unwrap().min::<f64>().unwrap());
    println!("Longest distance to closest neighbour is {:?} pc", df2.column("Min").unwrap().max::<f64>().unwrap());
    println!("Average distance to closest neighbour is {:?} pc", df2.column("Min").unwrap().sum::<f64>().unwrap()/(size as f64));
    println!("Longest distance between stars is {:?} pc", df2.column("Max").unwrap().max::<f64>().unwrap());
    println!("Average distance between stars is {:?} pc", df2.column("Avg").unwrap().sum::<f64>().unwrap()/(size as f64));

    df2
}

pub fn write_df(df: &mut DataFrame) {
    let mut file_out = std::fs::File::create("MinMaxPairs.csv").unwrap();
    CsvWriter::new(&mut file_out).finish(df).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_second_lowest() {
        let s = Series::new("days", [0.0, 0.1, 0.2, 0.3].as_ref());

        assert_eq!(s.nz_min(), Some(0.1));
    }
}