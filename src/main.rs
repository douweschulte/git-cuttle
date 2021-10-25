use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use svg::node::element::*;
use svg::Document;

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
    let root = Document::new()
        .set("viewBox", (0, 0, 1024, 1024))
        .add(plot_item(
            &item,
            Area::new(0.0, 0.0, 1024.0, 1024.0),
            item.size(),
        ));
    svg::save(path, &root).unwrap();
}

fn plot_item(item: &Item, area: Area, total: f64) -> Group {
    match item {
        Item::File { name, .. } => {
            let (x, y) = area.center();
            let circle = Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", item.size() / total * FACTOR)
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
            let (x, y) = area.center();
            let circle = Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", item.size() / total * FACTOR)
                .set("fill", "none")
                .set("stroke", item.colour());
            let text = Text::new()
                .set("x", x)
                .set("y", y - (item.size() / total * FACTOR))
                .set("text-anchor", "middle")
                .set("font-family", "sans-serif")
                .set("font-size", "1em")
                .set("fill", "black")
                .add(svg::node::Text::new(name));
            let mut group = Group::new().add(circle).add(text);

            let length = items.len();
            match length {
                0 => group,
                1 => group.add(plot_item(&items[0], area.shrink(0.1), total)),
                2 => {
                    let ratio = items[0].size() / (items[0].size() + items[1].size());
                    let chunks = area.split_horizontally(ratio);
                    group
                        .add(plot_item(&items[0], chunks.0, total))
                        .add(plot_item(&items[1], chunks.1, total))
                }
                n => {
                    let base = (n as f64).sqrt().ceil() as usize;
                    for (chunk, item) in area.split_evenly((base, base)).into_iter().zip(items) {
                        group = group.add(plot_item(item, chunk, total));
                    }
                    group
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
        self.get_size(1.0)
    }

    fn get_size(&self, level: f64) -> f64 {
        match self {
            Item::File { size: s, .. } => (*s as f64).log2(),
            Item::Folder { items, .. } => {
                1.1 * items
                    .iter()
                    .fold(0.0, |acc, item| acc + item.get_size(level + 1.0))
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
    pub fn center(&self) -> (f64, f64) {
        (
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
