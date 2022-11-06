use std::io;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::ops::Deref;
use chrono::prelude::*;

struct Pull {
    special: bool,
    pities: i32,
    time: NaiveDateTime,
}

#[derive(Copy, Clone)]
struct Avg {
    avg: f64,
    quantity: i32,
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
        for line in iter {
            let unwraped_line = line.unwrap();
            let strings = unwraped_line.split(',').collect::<Vec<&str>>();

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

            // println!("Collected 5-star {} with pity {}", strings[0], pities);
            array.push(Pull {
                special: non_special.contains(&strings[0]),
                pities,
                time: NaiveDateTime::parse_from_str(strings[5].as_ref(), "%Y-%m-%d %H:%M:%S").unwrap(),
            });

            pities = 0;
        }
    }

    let mut time_stats = [Avg { avg: 0.0, quantity: 0 }; 48];
    let mut out = File::create("D:\\result.txt").unwrap();
    for entry in array.iter() {
        // out.write(format!("{},{},{}\n", entry.special, entry.pities, entry.time.to_string()).as_ref()).expect("TODO: panic message");

        let time = entry.time.time();
        let index: usize = (2 * time.hour() + if time.minute() >= 30 { 1 } else { 0 }) as usize;
        let mut avg_ref: &mut Avg = time_stats.get_mut(index).unwrap();
        avg_ref.avg = (avg_ref.avg * avg_ref.quantity as f64 + entry.pities as f64) / (avg_ref.quantity as f64 + 1.0);
        avg_ref.quantity += 1;
    }

    for (i, el) in time_stats.iter().enumerate() {
        let hour = i / 2;
        let minute_start = (i % 2) * 30;
        let minute_end = (i % 2) * 30 + 30;
        let str = format!("{0:>0width$}:{1:>0width$} to {2:>0width$}:{3:>0width$} q={4} avg = {5}\n",
                          hour, minute_start,
                          if minute_end == 60 { hour + 1 } else { hour }, minute_end % 60,
                          el.quantity, el.avg, width=2
        );
        out.write(str.as_bytes()).expect("TODO: panic message");
    }
}
