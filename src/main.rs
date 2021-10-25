use plotters::prelude::*;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;

const MARGIN: f64 = 0.3;
const FACTOR: f64 = 500.0;

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
    let root = SVGBackend::new(path, (1024, 1024)).into_drawing_area();

    let total = item.size();
    plot_item(&item, &root, total)
}

fn plot_item(item: &Item, area: &DrawingArea<SVGBackend, plotters::coord::Shift>, total: f64) {
    match item {
        Item::File { name, .. } => {
            let range = area.get_pixel_range();
            println!("file {:?}", range);
            let x = (range.0.end - range.0.start) / 2 + range.0.start;
            let y = (range.1.end - range.1.start) / 2 + range.1.start;
            area.draw(&Circle::new(
                (x, y),
                item.size() / total * FACTOR,
                Into::<ShapeStyle>::into(item.colour()).filled(),
            ))
            .expect("Drawing of circle not working");
            area.draw(&Text::new(
                name.to_string(),
                (x, y),
                &"sans-serif".into_font().resize(20.0).color(&BLACK),
            ))
            .expect("Drawing of text not working");
            area.draw(&Rectangle::new(
                [(range.0.start, range.1.start), (range.0.end, range.1.end)],
                Into::<ShapeStyle>::into(item.colour()).stroke_width(2),
            ))
            .expect("Drawing of rect not working");
            println!(
                "rect {:?}",
                [(range.0.start, range.1.start), (range.0.end, range.1.end)]
            );
        }
        Item::Folder { name, items } => {
            // 3 items with 1000, 200, 500 size -> 10, 2, 5
            let range = area.get_pixel_range();
            let x = (range.0.end - range.0.start) / 2 + range.0.start;
            let y = (range.1.end - range.1.start) / 2 + range.1.start;
            let max_radius = (range.0.end - range.0.start).min(range.1.end - range.1.start);

            area.draw(&Circle::new(
                (x, y),
                max_radius,
                Into::<ShapeStyle>::into(item.colour()).stroke_width(2),
            ))
            .expect("Drawing of circle not working");
            area.draw(&Text::new(
                name.to_string(),
                (x, range.1.start),
                &"sans-serif".into_font().resize(20.0).color(&BLACK),
            ))
            .expect("Drawing of text not working");
            area.draw(&Rectangle::new(
                [(range.0.start, range.1.start), (range.0.end, range.1.end)],
                Into::<ShapeStyle>::into(item.colour()).stroke_width(2),
            ))
            .expect("Drawing of rect not working");

            let length = items.len();
            match length {
                0 => {}
                1 => plot_item(
                    &items[0],
                    &area.margin(MARGIN, MARGIN, MARGIN, MARGIN),
                    total,
                ),
                2 => {
                    let ratio = items[0].size() / (items[0].size() + items[1].size());
                    let chunks = area.split_horizontally(ratio);
                    plot_item(&items[0], &chunks.0, total);
                    plot_item(&items[1], &chunks.1, total);
                }
                n => {
                    let base = (n as f64).sqrt().ceil() as usize;
                    for (chunk, item) in area.split_evenly((base, base)).iter().zip(items) {
                        println!("chunk {:?}", chunk.get_pixel_range());
                        plot_item(item, chunk, total);
                    }
                }
            }
        }
    }
}

fn get_structure(path: &Path, ignore: &[&str]) -> Option<Item> {
    if path.is_dir()
        && !ignore
            .iter()
            .any(|d| Some(*d) == path.file_name().map(|s| s.to_str()).flatten())
    {
        Some(Item::Folder {
            name: path.to_str()?.to_string(),
            items: fs::read_dir(path).map_or(vec![], |r| {
                r.filter_map(|p| p.ok())
                    .map(|p| get_structure(&p.path(), ignore))
                    .flatten()
                    .collect()
            }),
        })
    } else if path.is_file() {
        if let Ok(meta) = path.metadata() {
            let name = path.to_str()?.to_string();
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
        self.get_size(1.0)
    }

    fn get_size(&self, level: f64) -> f64 {
        match self {
            Item::File { size: s, .. } => (level * MARGIN + 1.0) * (*s as f64).log2(),
            Item::Folder { items: i, .. } => i
                .iter()
                .fold(0.0, |acc, item| acc + item.get_size(level + 1.0)),
        }
    }

    pub fn colour(&self) -> RGBColor {
        match self {
            Item::File { class, .. } => match class {
                FileType::Code => RGBColor(197, 134, 161), //purple
                FileType::Data => RGBColor(78, 201, 176),  // green
                FileType::Configuration => RGBColor(220, 208, 143), // yellow
                FileType::Unknown => RGBColor(86, 154, 214), // blue
            },
            Item::Folder { .. } => RGBColor(126, 126, 126), // grey
        }
    }
}
