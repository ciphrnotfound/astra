package index

type QueryResult string

const (
    messageQueryResult    QueryResult = "message"
    statsQueryResult      QueryResult = "stats"
)

type IndexStats struct {
    // Add fields for IndexStats here
}