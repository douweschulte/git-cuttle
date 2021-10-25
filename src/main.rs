#![allow(dead_code)]

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use svg::node::element::*;
use svg::Document;

fn main() {
    let mut s = String::new();
    print!(">");
    let _ = stdout().flush();
    stdin().read_line(&mut s).expect("Incorrect input");
    s = s.trim().to_string();
    let structure = get_structure(Path::new(&s), &[".vscode", "target", ".git"]);
    println!("{:?}", structure);
    if let Some(item) = structure {
        plot(item, "plot.svg");
    }
}

fn plot(item: Item, path: &str) {
    let size = 1024.0;
    let margin = 10.0;
    let root = Document::new()
        .set("viewBox", (-margin, -margin, size + margin, size + margin))
        .add(plot_item(
            &item,
            Area::new(0.0, 0.0, size, size),
            item.size(),
        ));
    svg::save(path, &root).unwrap();
}

fn get_radius(size: f64, total: f64) -> f64 {
    (size.log2() / total.log2()) * 1024.0 * 0.5
}

fn plot_item(item: &Item, area: Area, total: f64) -> Group {
    match item {
        Item::File { name, .. } => {
            let Point(x, y) = area.center();
            let circle = Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", get_radius(item.size(), total))
                .set("fill", item.colour());
            let text = Text::new()
                .set("x", x)
                .set("y", y)
                .set("text-anchor", "middle")
                .set("font-family", "sans-serif")
                .set("font-size", "1em")
                .set("fill", "black")
                .add(svg::node::Text::new(name));
            Group::new().add(circle).add(text)
        }
        Item::Folder { name, items } => {
            let Point(x, y) = area.center();
            let radius = get_radius(item.size(), total);
            let circle = Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", radius)
                .set("fill", "none")
                .set("stroke", item.colour());
            let text = Text::new()
                .set("x", x)
                .set("y", y - radius)
                .set("text-anchor", "middle")
                .set("font-family", "sans-serif")
                .set("font-size", "1em")
                .set("fill", "black")
                .add(svg::node::Text::new(name));
            let mut group = Group::new().add(circle).add(text);

            let base = (items.len() as f64).sqrt().ceil() as usize;
            let positions: Vec<_> = area
                .split_evenly((base, base))
                .into_iter()
                .zip(items)
                .map(|(a, i)| (a, get_radius(i.size(), total), i))
                .collect();
            for (chunk, item) in improve_positions(positions, area) {
                group = group.add(plot_item(item, chunk, total));
            }
            group
        }
    }
}

fn improve_positions<'a>(
    positions: Vec<(Area, f64, &'a Item)>,
    bounds: Area,
) -> Vec<(Area, &'a Item)> {
    let center = bounds.center();
    let mut items: Vec<_> = positions
        .iter()
        .map(|(a, s, i)| (a.center(), s, Point(0.0, 0.0), i))
        .collect();
    //println!("Gravitate towards: {:?}", center);
    for _ in 0..100 {
        let mut vec = (0..items.len()).collect::<Vec<_>>();
        vec.shuffle(&mut thread_rng());
        for index in vec {
            let mut item = items[index];
            // update speed
            item.2 = (center - item.0).normalize() * 10.0 + item.2;
            //println!(
            //    "Item {} updated from {:?} with speed {:?}",
            //    index, item.0, item.2
            //);
            // update position
            item.0 = item.0 + item.2;
            // handle collisions
            for other_index in (0..items.len()).filter(|i| *i != index) {
                let other = items[other_index];
                let min_dis = item.1 + other.1 + 5.0;
                if item.0.distance(other.0) < min_dis {
                    // 'Bounce' away from the other ball, could maybe break on multiple collisions in a single frame
                    item.2 = (item.0 - other.0).normalize() - other.2 * 0.75;
                    items[other_index].2 = (other.0 - item.0).normalize() - item.2 * 0.75; // Push the other item a bit
                    for _ in 0..100 {
                        item.0 = item.0 + item.2 * 0.1;
                        if item.0.distance(other.0) >= min_dis {
                            break;
                        }
                    }
                }
            }
            if item.0 .0 < bounds.start_x && item.2 .0 < 0.0
                || item.0 .0 > bounds.end_x && item.2 .0 > 0.0
            {
                item.2 .0 *= -0.75;
            }
            if item.0 .1 < bounds.start_y && item.2 .1 < 0.0
                || item.0 .1 > bounds.end_y && item.2 .1 > 0.0
            {
                item.2 .1 *= -0.75;
            }
            //println!(
            //    "\tto: {:?} with speed {:?} collision {}",
            //    item.0, item.2, collision
            //);
            items[index] = item;
        }
    }
    items
        .iter()
        .map(|(Point(x, y), s, _, i)| {
            (
                Area::new(x - **s / 2.0, y - **s / 2.0, x + **s / 2.0, y + **s / 2.0),
                **i,
            )
        })
        .collect()
}

fn get_structure(path: &Path, ignore: &[&str]) -> Option<Item> {
    if path.is_dir()
        && !ignore
            .iter()
            .any(|d| Some(*d) == path.file_name().map(|s| s.to_str()).flatten())
    {
        Some(Item::Folder {
            name: path.file_name().map(|s| s.to_str()).flatten()?.to_string(),
            items: fs::read_dir(path).map_or(vec![], |r| {
                r.filter_map(|p| p.ok())
                    .map(|p| get_structure(&p.path(), ignore))
                    .flatten()
                    .collect()
            }),
        })
    } else if path.is_file() {
        if let Ok(meta) = path.metadata() {
            let name = path.file_name().map(|s| s.to_str()).flatten()?.to_string();
            Some(Item::File {
                name,
                size: meta.len(),
                class: find_class(path),
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn find_class(path: &Path) -> FileType {
    path.extension().map_or(FileType::Unknown, |n| {
        n.to_str().map_or(FileType::Unknown, |t| match t {
            "rs" | "cs" | "js" | "ts" | "r" | "cpp" | "py" => FileType::Code,
            "csv" | "tsv" | "xslx" | "xls" | "fasta" => FileType::Data,
            "yaml" | "toml" | "lock" => FileType::Configuration,
            _ => FileType::Unknown,
        })
    })
}

#[derive(Debug)]
enum Item {
    File {
        name: String,
        size: u64,
        class: FileType,
    },
    Folder {
        name: String,
        items: Vec<Item>,
    },
}

#[derive(Debug)]
enum FileType {
    Code,
    Data,
    Configuration,
    Unknown,
}

impl Item {
    pub fn size(&self) -> f64 {
        match self {
            Item::File { size, .. } => *size as f64,
            Item::Folder { items, .. } => {
                25.0_f64.powi(items.len() as i32)
                    * items.iter().fold(0.0, |acc, item| acc + item.size())
            }
        }
    }

    pub fn colour(&self) -> &str {
        match self {
            Item::File { class, .. } => match class {
                FileType::Code => "rgb(197, 134, 161)",          //purple
                FileType::Data => "rgb(78, 201, 176)",           // green
                FileType::Configuration => "rgb(220, 208, 143)", // yellow
                FileType::Unknown => "rgb(86, 154, 214)",        // blue
            },
            Item::Folder { .. } => "rgb(126, 126, 126)", // grey
        }
    }
}

struct Area {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
}

impl Area {
    pub fn new(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Self {
        Area {
            start_x,
            start_y,
            end_x,
            end_y,
        }
    }
    pub fn center(&self) -> Point {
        Point(
            (self.end_x - self.start_x) / 2.0 + self.start_x,
            (self.end_y - self.start_y) / 2.0 + self.start_y,
        )
    }
    pub fn shrink(&self, factor: f64) -> Self {
        let dx = self.end_x - self.start_x;
        let dy = self.end_y - self.start_y;
        Area {
            start_x: self.start_x + dx * factor,
            end_x: self.end_x - dx * factor,
            start_y: self.start_y + dy * factor,
            end_y: self.end_y - dy * factor,
        }
    }
    /// Split the area at the given point (in fractions: 0.5 is halfway)
    pub fn split_horizontally(&self, point: f64) -> (Self, Self) {
        let dx = self.end_x - self.start_x;
        (
            Area {
                start_x: self.start_x,
                end_x: self.start_x + dx * point,
                start_y: self.start_y,
                end_y: self.end_y,
            },
            Area {
                start_x: self.start_x + dx * point,
                end_x: self.end_x,
                start_y: self.start_y,
                end_y: self.end_y,
            },
        )
    }
    /// Split the area at the given point (in fractions: 0.5 is halfway)
    pub fn split_vertically(&self, point: f64) -> (Self, Self) {
        let dy = self.end_y - self.start_y;
        (
            Area {
                start_x: self.start_x,
                end_x: self.end_x,
                start_y: self.start_y,
                end_y: self.start_y + dy * point,
            },
            Area {
                start_x: self.start_x,
                end_x: self.end_x,
                start_y: self.start_y + dy * point,
                end_y: self.end_y,
            },
        )
    }
    pub fn split_evenly(&self, size: (usize, usize)) -> Vec<Self> {
        let step_x = (self.end_x - self.start_x) / (size.0 as f64);
        let step_y = (self.end_y - self.start_y) / (size.1 as f64);
        let mut output = Vec::with_capacity(size.0 * size.1);
        for x in 0..size.0 {
            for y in 0..size.1 {
                output.push(Area {
                    start_x: self.start_x + step_x * (x as f64),
                    end_x: self.start_x + step_x * ((x + 1) as f64),
                    start_y: self.start_y + step_y * (y as f64),
                    end_y: self.start_y + step_y * ((y + 1) as f64),
                })
            }
        }
        output
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point(f64, f64);

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Self;

    fn mul(self, other: f64) -> Self::Output {
        Self(self.0 * other, self.1 * other)
    }
}

impl Point {
    pub fn normalize(&self) -> Self {
        let sum = self.0.abs() + self.1.abs();
        if sum == 0.0 {
            Point(0.0, 0.0)
        } else {
            Point(self.0 / sum, self.1 / sum)
        }
    }

    pub fn distance(&self, other: Self) -> f64 {
        ((self.0 - other.0).powi(2) + (self.1 - other.1).powi(2)).sqrt()
    }
}
