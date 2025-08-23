/// OpenStreetMap module for geographic data access
pub mod osm;

// Re-export key types
pub use osm::{BoundingBox, Node, OsmClient, OsmQueryResult, Point, Relation, Way};
