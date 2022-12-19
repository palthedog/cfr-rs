use std::{
    error::Error,
    fs::{
        self,
        DirEntry,
    },
    path::{
        Path,
        PathBuf,
    },
};

use log::{
    error,
    info,
    warn,
};
use plotters::{
    self,
    prelude::*,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct LogRecord {
    #[allow(dead_code)]
    epoch: u32,

    elapsed_seconds: u32,
    exploitability: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let paths = fs::read_dir("./logs")?;
    for path in paths {
        let path = path?;
        if !path.file_type()?.is_dir() {
            continue;
        }
        plot_dir(&path)?;
    }
    Ok(())
}

fn plot_dir(dir: &DirEntry) -> Result<(), Box<dyn std::error::Error>> {
    let img_path = Path::new("graphs/").join(dir.file_name()).with_extension("svg");
    let root_area = SVGBackend::new(&img_path, (1000, 800)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let logs = load_logs(dir);
    if logs.is_empty() {
        warn!("no log files in {}", dir.path().display());
        return Ok(());
    }
    let (xr, yr) = logs_to_range(&logs);

    let mut chart = ChartBuilder::on(&root_area)
        .caption(dir.path().to_str().unwrap(), ("sans-serif", 20).into_font())
        .margin(5)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(xr.0..xr.1, (yr.0..yr.1).log_scale())?;

    chart
        .configure_mesh()
        .y_desc("Exploitability")
        .y_label_formatter(&|y| format!("{:.1e}", y))
        .y_label_style(("sans-serif", 18).into_font())
        .x_desc("Elapsed Time (mins)")
        .x_label_formatter(&|x| format!("{}", x / 60))
        .x_label_style(("sans-serif", 18).into_font())
        .draw()?;

    for (i, (name, log)) in logs.iter().enumerate() {
        let color = Palette99::pick(i).mix(0.8);
        info!("plotting: {}", name);
        chart
            .draw_series(LineSeries::new(
                log.iter().map(|r| (r.elapsed_seconds, r.exploitability)),
                color,
            ))?
            .label(name)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(1))
            });
    }

    chart
        .configure_series_labels()
        .background_style(WHITE)
        .border_style(BLACK)
        .label_font(("sans-serif", 18).into_font())
        .draw()?;

    root_area.present()?;
    info!("{} created", img_path.display());

    Ok(())
}

fn logs_to_range(logs: &Vec<(String, Vec<LogRecord>)>) -> ((u32, u32), (f64, f64)) {
    let mut xmin = u32::MAX;
    let mut xmax = u32::MIN;
    let mut ymin = f64::MAX;
    let mut ymax = f64::MIN;
    for (_name, log) in logs.iter() {
        let (xr, yr) = log_to_range(log);
        xmin = xmin.min(xr.0);
        ymin = ymin.min(yr.0);
        xmax = xmax.max(xr.1);
        ymax = ymax.max(yr.1);
    }
    ((xmin, xmax), (ymin, ymax))
}

fn log_to_range(v: &Vec<LogRecord>) -> ((u32, u32), (f64, f64)) {
    let xs = v.iter().map(|r| r.elapsed_seconds).collect::<Vec<_>>();
    let ys = v.iter().map(|r| r.exploitability).collect::<Vec<_>>();
    let xr = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap());
    let yr = (
        *ys.iter().min_by(|a, b| a.total_cmp(b)).unwrap(),
        *ys.iter().max_by(|a, b| a.total_cmp(b)).unwrap(),
    );
    (xr, yr)
}

fn load_logs(dir: &DirEntry) -> Vec<(String, Vec<LogRecord>)> {
    let mut v = vec![];
    let paths = match fs::read_dir(dir.path()) {
        Ok(p) => p,
        Err(err) => {
            error!("Failed to read dir: {}", err);
            return v;
        }
    };
    for path in paths {
        let path = path.unwrap();
        if let Ok(data) = load_log(&path.path()) {
            let name = path.file_name().to_string_lossy().to_string();
            v.push((name, data));
        }
    }
    v
}

fn load_log(path: &PathBuf) -> Result<Vec<LogRecord>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut v = vec![];
    for r in reader.deserialize() {
        match r {
            Ok(r) => v.push(r),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(limit_len(v, 800))
}

fn limit_len(v: Vec<LogRecord>, max: usize) -> Vec<LogRecord> {
    if v.len() < max {
        return v;
    }

    let mut new_v = Vec::with_capacity(max);
    let step = v.len() as f64 / max as f64;
    let mut next = 0.0f64;
    for (i, elem) in v.into_iter().enumerate() {
        if (i + 1) > next as usize {
            new_v.push(elem);
            next += step;
        }
    }
    assert_eq!(max, new_v.len());
    new_v
}
