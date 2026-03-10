 use crate::index::IndexStats;
 
 pub enum QueryResult {
     Message(String),
     Stats(IndexStats),
 }
