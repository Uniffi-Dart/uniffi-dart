use std::collections::HashSet;

// UniFFI 0.32 types: `Box<T>` (transparent across the FFI, so `T` in Dart) and
// `HashSet<T>` (`Set<T>`: a sequence on the wire, a Dart `Set<T>`).

#[derive(uniffi::Enum, Clone, Debug, PartialEq)]
pub enum Shape {
    Circle { radius: f64 },
    Square { side: f64 },
}

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct Point {
    x: i32,
    y: i32,
}

// Boxed record and enum fields, plain and optional, and a boxed recursive field.
#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct Boxes {
    shape: Box<Shape>,
    origin: Box<Point>,
    target: Option<Box<Point>>,
}

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct TreeNode {
    label: String,
    child: Option<Box<TreeNode>>,
}

#[uniffi::export]
pub fn roundtrip_boxes(boxes: Boxes) -> Boxes {
    boxes
}

#[uniffi::export]
pub fn roundtrip_tree(tree: TreeNode) -> TreeNode {
    tree
}

#[uniffi::export]
pub fn tree_depth(tree: TreeNode) -> u32 {
    1 + tree.child.map_or(0, |child| tree_depth(*child))
}

#[uniffi::export]
pub fn boxed_point(x: i32, y: i32) -> Box<Point> {
    Box::new(Point { x, y })
}

// A custom type over a string, as a set element.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag(pub String);

uniffi::custom_newtype!(Tag, String);

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct Sets {
    ids: HashSet<u32>,
    tags: HashSet<Tag>,
    maybe_ids: Option<HashSet<u32>>,
}

#[uniffi::export]
pub fn roundtrip_sets(sets: Sets) -> Sets {
    sets
}

#[uniffi::export]
pub fn roundtrip_id_set(ids: HashSet<u32>) -> HashSet<u32> {
    ids
}

#[uniffi::export]
pub fn roundtrip_tag_set(tags: HashSet<Tag>) -> HashSet<Tag> {
    tags
}

// Builds a set in Rust from a list, so duplicates are dropped on the Rust side.
#[uniffi::export]
pub fn unique_tags(tags: Vec<Tag>) -> HashSet<Tag> {
    tags.into_iter().collect()
}

#[uniffi::export]
pub fn sum_ids(ids: HashSet<u32>) -> u32 {
    ids.iter().sum()
}

uniffi::include_scaffolding!("api");
