use std::io;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use chrono::prelude::*;
use plotters::prelude::*;

const DAY_SLICES: usize = 48;

struct Pull {
    special: bool,
    win_fifty: bool,
    pities: i32,
    time: NaiveDateTime,
}

#[derive(Copy, Clone)]
struct Avg {
    avg: f64,
    win_chance: f64,
    special_count: i32,
    count: i32,
}

fn main() {
    let mut array: Vec<Pull> = vec![];
    let non_special = ["刻晴", "迪卢克", "七七", "莫娜", "琴", "提纳里"];

    let data_dir = "D:\\Code\\player_data";
    let dir = fs::read_dir(data_dir).unwrap();
    for path in dir {
        let unwraped_path = path.unwrap();
        println!("Parsing file {}", unwraped_path.file_name().to_str().unwrap());
        let file = File::open(unwraped_path.path()).unwrap();

        let reader = BufReader::new(file);
        let mut iter = reader.lines();
        iter.next();
        let mut pities = 0;
        let mut win_fifty_flag = true;

        for line in iter {
            let unwrapped_line = line.unwrap();
            let strings = unwrapped_line.split(',').collect::<Vec<&str>>();

            match strings[1].as_ref() {
                "100" | "200" | "302" => continue,
                _ => ()
            }

            // println!("Collected {} with {}", strings[0], strings[3]);
            pities += 1;
            match strings[3].as_ref() {
                "3" | "4" => continue,
                _ => ()
            }

            let special = !non_special.contains(&strings[0]);
            array.push(Pull {
                special,
                win_fifty: win_fifty_flag,
                pities,
                time: NaiveDateTime::parse_from_str(strings[5].as_ref(), "%Y-%m-%d %H:%M:%S").unwrap(),
            });

            win_fifty_flag = special;
            pities = 0;
        }
    }

    let mut time_stats = [Avg { avg: 0.0, win_chance: 0.0, special_count: 0, count: 0 }; DAY_SLICES + 1];
    let mut out = File::create("D:\\result.txt").unwrap();
    for entry in array.iter() {
        let time = entry.time.time();
        let index: usize = (2 * time.hour() + if time.minute() >= 30 { 1 } else { 0 }) as usize;
        let mut avg_ref: &mut Avg = time_stats.get_mut(index).unwrap();
        avg_ref.avg = (avg_ref.avg * avg_ref.count as f64 + entry.pities as f64) / (avg_ref.count as f64 + 1.0);
        avg_ref.count += 1;

        if entry.special {
            avg_ref.win_chance = (avg_ref.win_chance * avg_ref.special_count as f64 + if entry.win_fifty { 1.0 } else { 0.0 }) / (avg_ref.special_count as f64 + 1.0);
            avg_ref.special_count += 1;
        }
    }

    time_stats[DAY_SLICES] = time_stats[0];

    let root_area = BitMapBackend::new("D:\\test.png", (1000, 500)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40.0)
        .set_label_area_size(LabelAreaPosition::Bottom, 40.0)
        .set_label_area_size(LabelAreaPosition::Right, 40.0)
        .set_label_area_size(LabelAreaPosition::Top, 40.0)
        .caption("Genshin Avg Pulls", ("sans-serif", 40.0))
        .build_cartesian_2d(0.0..24.0, 55.0..65.0).unwrap()
        .set_secondary_coord(0.0..24.0, 0.4..0.6);

    ctx
        .configure_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Pulls")
        .x_desc("Time")
        .x_labels(25)
        .axis_desc_style(("sans-serif", 15))
        .draw().expect("TODO: panic message");

    ctx
        .configure_secondary_axes()
        .y_desc("Chance of winning fifty")
        .draw().expect("TODO: panic message");

    /*
    ctx.draw_series(
        LineSeries::new((0..DAY_SLICES + 1).zip(time_stats.iter()).map(|(x, y)| {
            ((x as f64) * 24.0 / (DAY_SLICES as f64), y.avg)
        }), &BLUE)
    ).unwrap();
    */

    ctx.draw_series((0..DAY_SLICES).zip(time_stats.iter()).map(|(x, y)| {
        let x0 = (x as f64) * 24.0 / (DAY_SLICES as f64);
        let x1 = x0 + 24.0 / (DAY_SLICES as f64);
        let mut bar = Rectangle::new([(x0, 0.0), (x1, y.avg)], BLUE.filled());
        bar.set_margin(0, 0, 2, 2);
        bar
    })).unwrap();

    ctx.draw_secondary_series(
        LineSeries::new((0..DAY_SLICES + 1).zip(time_stats.iter()).map(|(x, y)| {
            ((x as f64) * 24.0 / (DAY_SLICES as f64), y.win_chance)
        }), &RED)
    ).unwrap();

    for (i, el) in time_stats.iter().enumerate() {
        let start_minute = i * (24 * 60 / DAY_SLICES);
        let end_minute = (i + 1) * (24 * 60 / DAY_SLICES);
        let str = format!("{0:>0width$}:{1:>0width$} to {2:>0width$}:{3:>0width$} q={4} avg = {5}\n",
                          start_minute / 60, start_minute % 60,
                          end_minute / 60, end_minute % 60,
                          el.count, el.avg, width = 2
        );
        out.write(str.as_bytes()).expect("TODO: panic message");
    }
}
