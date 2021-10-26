use crate::structs::*;
use crate::structure::*;

use rand::seq::SliceRandom;
use rand::thread_rng;
use svg::node::element::*;
use svg::Document;

const MARGIN: f64 = 5.0;

pub fn plot(item: Item, path: &str) {
    let size = 1024.0;
    let margin = 20.0;

    let mut entities = plot_item(
        &item,
        Area::new(0.0, 0.0, size, size),
        (item.files(), item.size()),
    );
    improve_positions(&mut entities);
    entities = shrink_folder_sizes(entities);
    improve_positions(&mut entities);
    entities = shrink_folder_sizes(entities);
    println!("{:?}", entities);
    let (plot, _) = plot_entities(&entities, Group::new().set("id", "view-root"), &entities);

    let root = Document::new()
        .set("viewBox", (-margin, -margin, size + margin, size + margin))
        .set("xmlns:xlink", "http://www.w3.org/1999/xlink")
        .set("onload", "load()")
        .add(Style::new(std::include_str!("style.css")))
        .add(Script::new(std::include_str!("script.js")).set("type", "text/javascript"))
        .add(plot)
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
        ))
        .add(make_button(
            "Toggle ref lines",
            "toggle-references-button",
            Point(10.0, 90.0),
            "toggle_references_button()",
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

fn get_radius(size: f64, total: (i32, f64)) -> f64 {
    (size.log2() / total.1.log2()) * 1024.0 * 0.5 * 1.0005_f64.powi(total.0)
}

fn plot_entities(node: &EntityNode, group: Group, root: &EntityNode) -> (Group, Group) {
    match node {
        EntityNode::File(entity, item) => {
            let circle = Circle::new()
                .set("cx", entity.pos.0)
                .set("cy", entity.pos.1)
                .set("r", entity.radius)
                .set("fill", item.colour());
            let text = Text::new()
                .set("x", entity.pos.0)
                .set("y", entity.pos.1)
                .add(svg::node::Text::new(item.name()));
            let mut line_group = Group::new();
            if let Item::File { refs, .. } = item {
                for reference in refs {
                    if let Some(Point(x, y)) = find_ref(reference, root) {
                        line_group = line_group.add(
                            Line::new()
                                .set("x1", entity.pos.0)
                                .set("y1", entity.pos.1)
                                .set("x2", x)
                                .set("y2", y)
                                .set("class", "ref"),
                        );
                    }
                }
            }
            (
                group.add(Group::new().add(circle).add(text).set("class", "file")),
                line_group,
            )
        }
        EntityNode::Folder(entity, name, items) => {
            let circle = Circle::new()
                .set("cx", entity.pos.0)
                .set("cy", entity.pos.1)
                .set("r", entity.radius);
            let text = Text::new()
                .set("x", entity.pos.0)
                .set("y", entity.pos.1 - entity.radius)
                .add(svg::node::Text::new(name));
            let (transform, text_scale) = get_transform(&entity);
            let mut folder_group = Group::new()
                .add(circle)
                .add(text)
                .set("class", "folder")
                .set("data-transform", transform)
                .set("data-text-scale", text_scale);

            let mut lines = Group::new();
            for item in items {
                let res = plot_entities(item, folder_group, root);
                folder_group = res.0;
                lines = lines.add(res.1);
            }

            (group.add(folder_group).add(lines), Group::new())
        }
    }
}

fn find_ref(reference: &str, entity: &EntityNode) -> Option<Point> {
    match entity {
        EntityNode::File(place, Item::File { name, .. }) => {
            if name == reference {
                return Some(place.pos);
            } else {
                None
            }
        }
        EntityNode::Folder(_, _, items) => {
            for item in items {
                if let Some(p) = find_ref(reference, item) {
                    return Some(p);
                }
            }
            None
        }
        _ => panic!("Not possible"),
    }
}

#[derive(Debug, Clone, Copy)]
struct Entity {
    pos: Point,
    radius: f64,
    speed: Point,
}

impl Entity {
    pub fn bounding_box(&self) -> Area {
        Area {
            start_x: self.pos.0 - self.radius,
            start_y: self.pos.1 - self.radius,
            end_x: self.pos.0 + self.radius,
            end_y: self.pos.1 + self.radius,
        }
    }
    pub fn from_bounding_box(area: Area) -> Self {
        Entity {
            pos: area.center(),
            radius: (area.end_x - area.start_x).max(area.end_y - area.start_y) / 2.0,
            speed: Point(0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
enum EntityNode<'a> {
    File(Entity, &'a Item),
    Folder(Entity, String, Vec<EntityNode<'a>>),
}

impl<'a> EntityNode<'a> {
    pub fn entity(&self) -> &Entity {
        match self {
            EntityNode::File(e, _) => e,
            EntityNode::Folder(e, _, _) => e,
        }
    }
    pub fn entity_mut(&mut self) -> &mut Entity {
        match self {
            EntityNode::File(e, _) => e,
            EntityNode::Folder(e, _, _) => e,
        }
    }
    pub fn set_entity(self, entity: Entity) -> Self {
        match self {
            EntityNode::File(_, i) => EntityNode::File(entity, i),
            EntityNode::Folder(_, n, i) => EntityNode::Folder(entity, n, i),
        }
    }
}

fn plot_item(item: &Item, area: Area, total: (i32, f64)) -> EntityNode {
    match item {
        Item::File { .. } => EntityNode::File(
            Entity {
                pos: area.center(),
                radius: get_radius(item.size(), total),
                speed: Point(0.0, 0.0),
            },
            item,
        ),
        Item::Folder { name, items } => {
            let base = (items.len() as f64).sqrt().ceil() as usize;
            EntityNode::Folder(
                Entity {
                    pos: area.center(),
                    radius: get_radius(item.size(), total),
                    speed: Point(0.0, 0.0),
                },
                name.to_string(),
                items
                    .iter()
                    .zip(area.split_evenly((base, base)))
                    .map(|(i, a)| plot_item(i, a, total))
                    .collect(),
            )
        }
    }
}

fn improve_positions(entity: &mut EntityNode) {
    match entity {
        EntityNode::Folder(folder_entity, _, items) => {
            improve_folder_positions(folder_entity, items);
            for item in items {
                improve_positions(item)
            }
        }
        _ => (),
    }
}

fn improve_folder_positions(entity: &mut Entity, items: &mut Vec<EntityNode>) {
    let bounds = entity.bounding_box();
    let center = bounds.center();
    //println!("Gravitate towards: {:?}", center);
    for _ in 0..100 {
        let mut vec = (0..items.len()).collect::<Vec<_>>();
        vec.shuffle(&mut thread_rng());
        for index in vec {
            let mut item = items[index].entity().clone();
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
                let other = items[other_index].entity();
                let min_dis = item.radius + other.radius + MARGIN;
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
            if item.pos.0 < bounds.start_x && item.speed.0 < 0.0 {
                item.speed.0 += 0.5;
            } else if item.pos.0 > bounds.end_x && item.speed.0 > 0.0 {
                item.speed.0 -= 0.5;
            }
            if item.pos.1 < bounds.start_y && item.speed.1 < 0.0 {
                item.speed.1 += 0.5;
            } else if item.pos.1 > bounds.end_y && item.speed.1 > 0.0 {
                item.speed.1 -= 0.5;
            }
            //println!(
            //    "\tto: {:?} with speed {:?} collision {}",
            //    item.0, item.2, collision
            //);
            items[index] = items[index].clone().set_entity(item);
        }
    }
    if !items.is_empty() {
        let first = items[0].entity();
        let mut bounding_box = Area::new(first.pos.0, first.pos.1, first.pos.0, first.pos.1);
        for node in items.iter_mut() {
            let item = node.entity();
            if item.pos.0 - item.radius < bounding_box.start_x {
                bounding_box.start_x = item.pos.0 - item.radius
            }
            if item.pos.0 + item.radius > bounding_box.end_x {
                bounding_box.end_x = item.pos.0 + item.radius
            }
            if item.pos.1 - item.radius < bounding_box.start_y {
                bounding_box.start_y = item.pos.1 - item.radius
            }
            if item.pos.1 + item.radius > bounding_box.end_y {
                bounding_box.end_y = item.pos.1 + item.radius
            }
        }
        let re_center = bounding_box.center() - center;
        for node in items.iter_mut() {
            let item = node.entity_mut();
            item.pos = item.pos - re_center;
        }
    }
}

fn shrink_folder_sizes(entity: EntityNode) -> EntityNode {
    match entity {
        EntityNode::Folder(mut folder_entity, name, mut items) => {
            for n in 0..items.len() {
                items[n] = shrink_folder_sizes(items[n].clone())
            }
            if !items.is_empty() {
                folder_entity.radius = 0.0;
                for node in &*items {
                    let item = node.entity();
                    let distance = item.pos.distance(folder_entity.pos) + item.radius
                        - folder_entity.radius
                        + MARGIN;
                    if distance > 0.0 {
                        folder_entity.radius += distance
                    }
                }
            }
            EntityNode::Folder(folder_entity, name, items)
        }
        e => e,
    }
}

fn get_transform(entity: &Entity) -> (String, String) {
    let area = entity.bounding_box();
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
