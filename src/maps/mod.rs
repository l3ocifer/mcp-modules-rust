/// OpenStreetMap module for geographic data access
pub mod osm;

// Re-export key types
pub use osm::{OsmClient, Point, BoundingBox, Node, Way, Relation, OsmQueryResult}; 