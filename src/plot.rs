use crate::structs::*;
use crate::structure::*;

use rand::seq::SliceRandom;
use rand::thread_rng;
use svg::node::element::*;
use svg::Document;

pub fn plot(item: Item, path: &str) {
    let size = 1024.0;
    let margin = 20.0;
    let root = Document::new()
        .set("viewBox", (-margin, -margin, size + margin, size + margin))
        .set("xmlns:xlink", "http://www.w3.org/1999/xlink")
        .set("onload", "load()")
        .add(Style::new(std::include_str!("style.css")))
        .add(Script::new(std::include_str!("script.js")).set("type", "text/javascript"))
        .add(plot_item(&item, Area::new(0.0, 0.0, size, size), item.size()).set("id", "view-root"))
        .add(make_button(
            "Toggle file text",
            "toggle-file-text-button",
            Point(10.0, 10.0),
            "toggle_file_text_button()",
        ))
        .add(make_button(
            "Reset view",
            "reset-view-button",
            Point(10.0, 50.0),
            "reset_view_button()",
        ));
    svg::save(path, &root).unwrap();
}

fn make_button(text: &str, id: &str, pos: Point, call_back: &str) -> Group {
    Group::new()
        .add(
            Rectangle::new()
                .set("x", pos.0)
                .set("y", pos.1)
                .set("width", 120)
                .set("height", 30),
        )
        .add(
            Text::new()
                .set("x", pos.0 + 5.0)
                .set("y", pos.1 + 20.0)
                .add(svg::node::Text::new(text)),
        )
        .set("class", "btn")
        .set("id", id)
        .set("onclick", call_back)
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
                .set("opacity", "var(--file-text-opacity)")
                .add(svg::node::Text::new(name));
            Group::new().add(circle).add(text).set("class", "file")
        }
        Item::Folder { name, items } => {
            let Point(x, y) = area.center();
            let radius = get_radius(item.size(), total);
            let circle = Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", radius)
                .set("stroke", item.colour());
            let text = Text::new()
                .set("x", x)
                .set("y", y - radius)
                .set("opacity", "var(--folder-text-opacity)")
                .add(svg::node::Text::new(name));
            let (transform, text_scale) = get_transform(&area);
            let mut group = Group::new()
                .add(circle)
                .add(text)
                .set("class", "folder")
                .set("data-transform", transform)
                .set("data-text-scale", text_scale);
            //.set("onclick", "folder_click");

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
    #[derive(Debug, Clone, Copy)]
    struct Entity<'a> {
        pos: Point,
        size: f64,
        speed: Point,
        item: &'a Item,
    }
    let center = bounds.center();
    let mut items: Vec<_> = positions
        .iter()
        .map(|(a, s, i)| Entity {
            pos: a.center(),
            size: *s,
            speed: Point(0.0, 0.0),
            item: i,
        })
        .collect();
    //println!("Gravitate towards: {:?}", center);
    for _ in 0..100 {
        let mut vec = (0..items.len()).collect::<Vec<_>>();
        vec.shuffle(&mut thread_rng());
        for index in vec {
            let mut item = items[index];
            // update speed
            item.speed = (center - item.pos).normalize() * 0.5 + item.speed;
            //println!(
            //    "Item {} updated from {:?} with speed {:?}",
            //    index, item.0, item.2
            //);
            // update position
            item.pos = item.pos + item.speed;
            // handle collisions
            for other_index in (0..items.len()).filter(|i| *i != index) {
                let other = &items[other_index];
                let min_dis = item.size + other.size + 5.0;
                if item.pos.distance(other.pos) < min_dis {
                    // 'Bounce' away from the other ball, could maybe break on multiple collisions in a single frame
                    item.speed = (item.pos - other.pos).normalize() - other.speed * 0.75;
                    //items[other_index].speed =
                    //    (other.pos - item.pos).normalize() - item.speed * 0.75; // Push the other item a bit
                    for _ in 0..100 {
                        item.pos = item.pos + item.speed * 0.1;
                        if item.pos.distance(other.pos) >= min_dis {
                            break;
                        }
                    }
                }
            }
            if item.pos.0 < bounds.start_x && item.speed.0 < 0.0
                || item.pos.0 > bounds.end_x && item.speed.0 > 0.0
            {
                item.speed.0 *= -0.75;
            }
            if item.pos.1 < bounds.start_y && item.speed.1 < 0.0
                || item.pos.1 > bounds.end_y && item.speed.1 > 0.0
            {
                item.speed.1 *= -0.75;
            }
            //println!(
            //    "\tto: {:?} with speed {:?} collision {}",
            //    item.0, item.2, collision
            //);
            items[index] = item;
        }
    }
    if items.len() > 0 {
        let mut bounding_box = Area::new(
            items[0].pos.0,
            items[0].pos.1,
            items[0].pos.0,
            items[0].pos.1,
        );
        for item in &mut items {
            if item.pos.0 - item.size < bounding_box.start_x {
                bounding_box.start_x = item.pos.0 - item.size
            }
            if item.pos.0 + item.size > bounding_box.end_x {
                bounding_box.end_x = item.pos.0 + item.size
            }
            if item.pos.1 - item.size < bounding_box.start_y {
                bounding_box.start_y = item.pos.1 - item.size
            }
            if item.pos.1 + item.size > bounding_box.end_y {
                bounding_box.end_y = item.pos.1 + item.size
            }
        }
        let re_center = bounding_box.center() - center;
        for item in &mut items {
            item.pos = item.pos - re_center;
        }
    }
    items
        .iter()
        .map(
            |Entity {
                 pos: Point(x, y),
                 size,
                 item,
                 ..
             }| {
                (
                    Area::new(
                        x - *size / 2.0,
                        y - *size / 2.0,
                        x + *size / 2.0,
                        y + *size / 2.0,
                    ),
                    *item,
                )
            },
        )
        .collect()
}

fn get_transform(area: &Area) -> (String, String) {
    let size = (area.end_x - area.start_x).min(area.end_y - area.start_y) * 1.5;
    let scale = 1024.0 / size;
    let transform_x = area.start_x + (area.end_x - area.start_x) / 2.0 - size / 2.0;
    let transform_y = area.start_y + (area.end_y - area.start_y) / 2.0 - size / 2.0;
    (
        format!(
            "scale({}) translate(-{}px, -{}px)",
            scale, transform_x, transform_y
        ),
        (1.0 / scale).to_string(),
    )
}
