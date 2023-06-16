use decorum::R64;
use indicatif::ProgressBar;
use ndarray::Array2;
use polars::prelude::*;

#[derive(Debug)]
pub struct Source {
    id:    Vec<String>,
    ra:    Vec<f64>,
    dec:   Vec<f64>,
    rho:   Vec<f64>,
    g_abs: Vec<f64>
}

impl Source {
    // Calculates luminosity from absolute magnitude relative to Sol
    // Solar absolute magnitude in Gaia G band = 4.68
    // Solar luminosity = 382.8 x10^24 W or 382.8 yottawatts (YW)
    fn calc_rel_lum(&self) -> Vec<f64> {
        let m_sol = 4.68;

        self.g_abs.iter().map(|star| 100.0_f64.powf((m_sol - star)/5.0)).collect()
    }
}

// Create dataframe from CSV file
// I did not see a way to filter columns when reading with LazyFrames
pub fn load_data(path: &str, load_cols: Vec<String>) -> PolarsResult<DataFrame> {
    CsvReader::from_path(path)?
        .infer_schema(None)
        .has_header(true)
        .with_delimiter(b',')
        .with_columns(Some(load_cols))
        .finish()
}

pub fn lazy_load(path: &str) -> PolarsResult<LazyFrame> {
    LazyCsvReader::new(path)
        .has_header(true)
        .with_delimiter(b',')
        .finish()
}

pub fn restructure_data(df: &DataFrame) -> Source {
    let messy_desc = df.column("designation").unwrap()
        .iter().map(|s| s.to_string()).collect::<Vec<_>>();

    let mut des_vec: Vec<String> = Vec::new();
    for line in messy_desc {
        let newline = line.split('"').collect::<Vec<_>>();
        des_vec.push(newline[1].to_string());
    }

    let ra_vec: Vec<f64> = df.column("ra")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|r| match r {
            Some(r) => r,
            _ => panic!("Not a number")
        })
        .collect();

    let dec_vec: Vec<f64> = df.column("dec")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|d| match d {
            Some(d) => d,
            _ => panic!("Not a number")
        })
        .collect();

    let rho_vec: Vec<f64> = df.column("distance")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|rho| match rho {
            Some(rho) => rho,
            _ => panic!("Not a number")
        })
        .collect();

    let g_abs_vec: Vec<f64> = df.column("g_abs")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|g| match g {
            Some(g) => g,
            _ => panic!("Not a number")
        })
        .collect();
    
    Source {
        id: des_vec,
        ra: ra_vec,
        dec: dec_vec,
        rho: rho_vec,
        g_abs: g_abs_vec
    }
}

// Calculate distance(pc) from parallax(mas)
fn distance(parallax: f64) -> f64 {
    let d = 1. / (parallax / 1000.);

    d
}

// Add distance(pc) column
pub fn calc_rho(df: &mut DataFrame) -> DataFrame {
    let dist = Series::new(
        "distance",
        df.column("parallax")
            .unwrap()
            .f64()
            .unwrap()
            .into_iter()
            .map(|s| match s {
                Some(s) => distance(s),
                _ => panic!("Empty cell"),
            })
            .collect::<Vec<f64>>(),
    );

    df.with_column(dist).unwrap().clone()
}

// Calculate angular distance between two points (need to convert to radians)
fn calc_ang_dist(ra1: f64, ra2: f64, dec1: f64, dec2: f64) -> f64 {
    ((dec1.to_radians().sin() * dec2.to_radians().sin())
        + (dec1.to_radians().cos() * dec2.to_radians().cos() * (ra1 - ra2).to_radians().cos()))
    .acos()
}

// Calculate linear distance between to spherical points
// Gamma = angle between to position vectors with common origin in radians
// Rho = radial distance from common origin to point (vector lengths)
fn calc_lin_dist(gamma: f64, rho1: f64, rho2: f64) -> f64 {
    (rho1.powf(2.0) + rho2.powf(2.0) - (2.0 * rho1 * rho2 * gamma.cos())).sqrt()
}

// Calculate distances between spherical coordinates
pub fn calc_distances(df1: &DataFrame, source: &Source) -> DataFrame {
    let mut df2 = df1
        .clone()
        .lazy()
        .select([cols(["designation"])])
        .collect()
        .unwrap();

    let size = df1.height();
    let calculations = ((size * (size - 1)) / 2).try_into().unwrap();

    let mut table = Array2::<f64>::zeros((size, size));

    let bar = ProgressBar::new(calculations);
    for col in 0..size {
        for row in 0..size {
            if row <= col {
                continue;
            } else {
                let gamma = calc_ang_dist(source.ra[col], source.ra[row], source.dec[col], source.dec[row]);
                let lin_dist = calc_lin_dist(gamma, source.rho[col], source.rho[row]);
                [table[[row, col]], table[[col, row]]] = [lin_dist; 2];

                bar.inc(1);
            }
        }
    }
    bar.finish();

    // Translate data from ndarray to Polars dataframe
    for idx in 0..size {
        let designation = &source.id[idx];
        let col_data = table.column(idx).to_vec();
        let series_data = Series::new(designation, col_data);
        df2.with_column(series_data).unwrap();
    }

    df2
}

pub fn write_df(df: &mut DataFrame) {
    let mut file_out = std::fs::File::create("Distances.parquet").unwrap();
    ParquetWriter::new(&mut file_out).finish(df).unwrap();
}

//----Calculate Gini Coefficient---------------------------------------------->

fn sort_ascending(data: Vec<f64>) -> Vec<f64> {
    let mut ordered: Vec<R64> = data.into_iter().map(R64::from).collect();
    ordered.sort();
    ordered.into_iter().map(R64::into_inner).collect()
}

// Wow, sorting and dividing a vector of floats is SO messy
fn normalise_lums(source: &Source) -> Vec<f64> {
    let sorted = sort_ascending(source.calc_rel_lum());

    let total: f64 = sorted.iter().sum();
    sorted.iter().map(|lum| lum / total).collect::<Vec<f64>>()
}

fn lorenz_curve(data: Vec<f64>) -> Vec<f64> {
    let mut sigma = 0.0;
    let sigma_data: Vec<f64> = data
        .iter()
        .scan(0.0, |acc, &x| {
            *acc += x;
            Some(*acc)
        })
        .map(|x| {
            sigma = x;
            x
        })
        .collect();

    let step = 1.0 / (data.len() as f64);
    let sigma_range: Vec<f64> = (0..data.len())
        .step_by(1)
        .map(|i| (i as f64) * step)
        .scan(0.0, |acc, x| {
            *acc += x;
            Some(*acc)
        })
        .collect();

    sigma_data.iter().map(|x| x / sigma_range.last().unwrap()).collect()
}

pub fn gini_coefficient(source: &Source) -> f64 {
    let data = normalise_lums(source);
    let curve = lorenz_curve(data);

    let auc = curve
        .windows(2)
        .map(|x| (x[0] + x[1]) / 2.0)
        .sum::<f64>();

    1.0 - 2.0 * auc
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;

    #[test]
    fn load_test_data() {
        let test_file = "../Test.csv";
        let test_cols = vec![
            "designation".to_string(),
            "ra".to_string(),
            "dec".to_string(),
            "parallax".to_string(),
        ];

        assert_eq!(load_data(test_file, test_cols).unwrap().shape(), (100, 4));
    }

    #[test]
    fn test_ang_dist1() {
        let ra1: f64 = 56.117103;
        let ra2: f64 = 57.623953;
        let dec1: f64 = 19.184814;
        let dec2: f64 = 19.350312;
        //let gamma = calc_ang_dist(ra1, ra2, dec1, dec2).to_degrees();

        assert_float_absolute_eq!(
            calc_ang_dist(ra1, ra2, dec1, dec2).to_degrees(),
            1.4320,
            0.0001
        );
    }

    #[test]
    fn test_ang_dist2() {
        let ra1: f64 = 57.623953;
        let ra2: f64 = 58.806865;
        let dec1: f64 = 19.350312;
        let dec2: f64 = 19.504493;
        //let gamma = calc_ang_dist(ra1, ra2, dec1, dec2).to_degrees();

        assert_float_absolute_eq!(
            calc_ang_dist(ra1, ra2, dec1, dec2).to_degrees(),
            1.1262,
            0.0001
        );
    }

    #[test]
    fn test_lin_dist1() {
        let ra1: f64 = 56.117103;
        let ra2: f64 = 57.623953;
        let dec1: f64 = 19.184814;
        let dec2: f64 = 19.350312;
        let rho1 = 144.419884;
        let rho2 = 135.251336;
        let gamma = calc_ang_dist(ra1, ra2, dec1, dec2);

        assert_float_absolute_eq!(calc_lin_dist(gamma, rho1, rho2), 9.81141, 0.00001);
    }

    #[test]
    fn test_lin_dist2() {
        let ra1: f64 = 57.623953;
        let ra2: f64 = 58.806865;
        let dec1: f64 = 19.350312;
        let dec2: f64 = 19.504493;
        let rho1 = 135.251336;
        let rho2 = 131.867289;
        let gamma = calc_ang_dist(ra1, ra2, dec1, dec2);

        assert_float_absolute_eq!(calc_lin_dist(gamma, rho1, rho2), 4.28273, 0.00001);
    }

    #[test]
    fn test_calc_rel_lum() {
        let des_vec = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string()
        ];
        let ra_vec = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let dec_vec = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let rho_vec = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let g_abs_vec = vec![
            5.77120203581248,
            13.1477871804314,
            9.87512860703366,
            4.54148082591236,
            10.9815460996278
        ];
        let source = Source {
            id: des_vec,
            ra: ra_vec,
            dec: dec_vec,
            rho: rho_vec,
            g_abs: g_abs_vec
        };
        let lums = source.calc_rel_lum();

        assert_float_absolute_eq!(lums[0], 0.36603211,  0.00000001);
        assert_float_absolute_eq!(lums[1], 0.000410096, 0.000000001);
        assert_float_absolute_eq!(lums[2], 0.00835504,  0.00000001);
        assert_float_absolute_eq!(lums[3], 1.136076744, 0.000000001);
        assert_float_absolute_eq!(lums[4], 0.003015654, 0.000000001);
    }
}
