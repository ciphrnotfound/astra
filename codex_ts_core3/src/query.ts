type IndexStats = {
    // assuming IndexStats is an object with properties
    // this will be filled in automatically by the compiler
};

enum QueryResult {
    Message(string: string),
    Stats(indexStats: IndexStats),
}