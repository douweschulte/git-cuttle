use regex::Regex;
use std::fs;
use std::io::Read;
use std::path::Path;

pub fn get_structure(path: &Path, ignore: &[&str]) -> Option<Item> {
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
            let patterns = Regex::new(
                r"(?:use crate::([^;]*?)(?:::\*)?(?:::\{[^;]\})?;)|(?:include_str!\(([^\)]*)\))|(?:mod ([^;]*);)",
            )
            .unwrap();
            let mut s = String::new();
            let _ = fs::File::open(path).ok()?.read_to_string(&mut s).ok()?;
            //let matches = patterns.find_iter(&s);
            let name = path.file_name().map(|s| s.to_str()).flatten()?.to_string();
            Some(Item::File {
                name,
                full_name: path.to_str()?.to_string(),
                size: meta.len(),
                class: find_class(path),
                refs: patterns
                    .captures_iter(&s)
                    .filter_map(|c| {
                        if let Some(r) = c.get(1) {
                            Some(r.as_str().replace("::", "/") + ".rs")
                        } else if let Some(r) = c.get(2) {
                            Some(r.as_str().replace('"', ""))
                        } else if let Some(r) = c.get(3) {
                            Some(r.as_str().to_string() + ".rs")
                        } else {
                            None
                        }
                    })
                    .collect(),
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
            "csv" | "tsv" | "xlsx" | "xls" | "fasta" => FileType::Data,
            "yaml" | "toml" | "lock" => FileType::Configuration,
            _ => FileType::Unknown,
        })
    })
}

#[derive(Debug)]
pub enum Item {
    File {
        name: String,
        full_name: String,
        size: u64,
        class: FileType,
        refs: Vec<String>,
    },
    Folder {
        name: String,
        items: Vec<Item>,
    },
}

#[derive(Debug)]
pub enum FileType {
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
                let sum = items.iter().fold(0.0, |acc, item| acc + item.size());
                let len = items.len() as f64;
                25.0_f64.powf(len * 1.10) * sum * (sum / len)
            }
        }
    }

    pub fn colour(&self) -> &str {
        match self {
            Item::File { class, .. } => match class {
                FileType::Code => "var(--color-primary)",       //purple
                FileType::Data => "var(--color-primary-shade)", // green
                FileType::Configuration => "var(--color-secondary)", // yellow
                FileType::Unknown => "var(--color-tertiary)",   // blue
            },
            Item::Folder { .. } => "var(--color-light)", // grey
        }
    }
}
