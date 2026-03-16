package main

import (
    "encoding/json"
    "errors"
    "log"
    "os"
    "path/filepath"
    "sort"
    "strconv"
    "strings"
    "sync"
    "text/tabwriter"
    "time"

    "github.com/google/uuid"
)

type HealthScores struct {
	CodeQuality    uint32 `json:"code_quality"`
	TestHealth     uint32 `json:"test_health"`
	CrossLangDrift uint32 `json:"cross_lang_drift"`
	SecuritySurface uint32 `json:"security_surface"`
	GitHealth      uint32 `json:"git_health"`
	TeamVelocity   uint32 `json:"team_velocity"`
}

type MemoryEntry struct {
	Kind       string  `json:"kind"`
	Content    string  `json:"content"`
	Timestamp  uint64  `json:"timestamp"`
	Event      *MemoryEvent  `json:"event"`
}

type MemoryEvent struct {
	IndexSnapshot struct {
		FileCount  int `json:"file_count"`
		TotalLines int `json:"total_lines"`
		Languages  map[string]int `json:"languages"`
	} `json:"index_snapshot"`
	MigrationRun struct {
		From string `json:"from"`
		To   string `json:"to"`
		FileCount  int `json:"file_count"`
	} `json:"migration_run"`
	TeamSession struct {
		Developer   string `json:"developer"`
		TaskID      string `json:"task_id"`
		DurationSecs uint64  `json:"duration_secs"`
		 LinesAdded  int `json:"lines_added"`
		LinesDeleted int `json:"lines_deleted"`
	} `json:"team_session"`
	WorktreeSnapshot struct {
		ChangedFiles int `json:"changed_files"`
		Files        []string `json:"files"`
	} `json:"worktree_snapshot"`
	HealthSnapshot HealthScores `json:"health_snapshot"`
	GitCommit struct {
		ID      string `json:"id"`
		Summary string `json:"summary"`
		Author  string `json:"author"`
		Date    string `json:"date"`
	} `json:"git_commit"`
}

type MemoryStore struct {
	entries []*MemoryEntry
	path    string
	maxEntries int
	mu sync.Mutex
}

func GetNowSecs() uint64 {
    return uint64(time.Since(time.Unix(0, 0)).Seconds())
}

func DefaultMaxEntries() int {
    return 2000
}

func (m *MemoryStore) Load(path string) (*MemoryStore, error) {
    entries := make([]*MemoryEntry, 0)
    file, err := os.Open(path)
    if err != nil {
        return nil, err
    }
    defer file.Close()

    decoder := json.NewDecoder(file)
    err = decoder.Decode(&entries)
    if err != nil {
        return nil, err
    }
    m.mu.Lock()
    m.entries = entries
    m.path = path
    m.maxEntries = DefaultMaxEntries()
    m.trimToMax()
    m.mu.Unlock()
    return m, nil
}

func (m *MemoryStore) Add(kind string, content string) {
    event := &MemoryEvent{IndexSnapshot: MemoryEvent_IndexSnapshot{}}
    m.pushEntry(&MemoryEntry{
        Kind:       kind,
        Content:    content,
        Timestamp:  GetNowSecs(),
        Event:      event,
    })
}

func (m *MemoryStore) AddEvent(kind string, content string, event MemoryEvent) {
    m.pushEntry(&MemoryEntry{
        Kind:       kind,
        Content:    content,
        Timestamp:  GetNowSecs(),
        Event:      &event,
    })
}

func (m *MemoryStore) Recent(limit int) []*MemoryEntry {
    m.mu.Lock()
    defer m.mu.Unlock()
    if limit > len(m.entries) {
        return m.entries
    } else {
        return m.entries[len(m.entries)-limit:]
    }
}

func (m *MemoryStore) getEventsByType(eventType string) []*MemoryEntry {
    m.mu.Lock()
    defer m.mu.Unlock()
    events := make([]*MemoryEntry, 0)
    for _, item := range m.entries {
        if item.Kind == eventType {
            events = append(events, item)
        }
    }
    sort.Slice(events, func(i, j int) bool {
        return events[i].Timestamp > events[j].Timestamp
    })
    return events
}

func (m *MemoryStore) GetLatestEntry(eventType string) *MemoryEntry {
    m.mu.Lock()
    defer m.mu.Unlock()
    events := m.getEventsByType(eventType)
    if len(events) == 0 {
        return nil
    } else {
        return events[0]
    }
}

func (m *MemoryStore) getEventsSince(timestamp uint64) []*MemoryEntry {
    m.mu.Lock()
    defer m.mu.Unlock()
    events := make([]*MemoryEntry, 0)
    for _, item := range m.entries {
        if item.Timestamp >= timestamp {
            events = append(events, item)
        } else if len(events) > 0 && string(item.Kind) < string(events[len(events)-1].Kind) {
            return events
        }
    }
    return events
}

func (m *MemoryStore) Search(query string, limit int) []*MemoryEntry {
    m.mu.Lock()
    defer m.mu.Unlock()
    if query == "" {
        return make([]*MemoryEntry, 0)
    }
    trimmed := strings.ToLower(query)
    events := make([]*MemoryEntry, 0)
    for _, item := range m.entries {
        kind := strings.ToLower(item.Kind)
        content := strings.ToLower(item.Content)
        if kind == trimmed || content == trimmed {
            events = append(events, item)
        }
        if len(events) >= limit {
            break
        }
    }
    sort.Slice(events, func(i, j int) bool {
        return events[i].Timestamp > events[j].Timestamp
    })
    return events
}

func (m *MemoryStore) pushEntry(entry *MemoryEntry) {
    m.mu.Lock()
    defer m.mu.Unlock()
    if m.maxEntries == 0 {
        return
    }
    if len(m.entries) == 0 || m.entryDiff(m.entries[len(m.entries)-1], entry) == "" {
        m.entries = append(m.entries, entry)
        m.trimToMax()
        err := m.save()
        if err != nil {
            log.Println(err)
        }
    } else {
        log.Println("entry already exist")
    }
}

func (m *MemoryStore) trimToMax() {
    if m.maxEntries == 0 {
        return
    }
    if len(m.entries) > m.maxEntries {
        m.mu.Lock()
        defer m.mu.Unlock()
        m.entries = m.entries[len(m.entries)-m.maxEntries:]
    }
}

func (m *MemoryStore) save() error {
    m.mu.Lock()
    defer m.mu.Unlock()
    writer, err := os.Create(m.path)
    if err != nil {
        return err
    }
    defer writer.Close()

    encoder := json.NewEncoder(writer)
    if err != nil {
        return err
    }

    encoder.Encode(m.entries)

    return nil
}

func (m *MemoryStore) entryDiff(prev, curr *MemoryEntry) string {
    if prev.Kind == curr.Kind && prev.Content == curr.Content {
        if prev.Event == nil && curr.Event == nil {
            return ""
        }
        if prev.Event == nil && curr.Event != nil {
            return "event changed"
        } else if prev.Event != nil && curr.Event == nil {
            return "event removed"
        } else {
            switch {
            case prev.Event != nil && curr.Event != nil && prev.Event.IndexSnapshot != nil && curr.Event.IndexSnapshot != nil:
                if strings.Compare(string(prev.Event.IndexSnapshot.FileCount), string(curr.Event.IndexSnapshot.FileCount)) != 0 {
                    return "event file count changed"
                }
                if strings.Compare(string(prev.Event.IndexSnapshot.TotalLines), string(curr.Event.IndexSnapshot.TotalLines)) != 0 {
                    return "event total lines changed"
                }
                if len(prev.Event.IndexSnapshot.Languages) != len(curr.Event.IndexSnapshot.Languages) {
                    return "event languages size changed"
                }
                for lang := range prev.Event.IndexSnapshot.Languages {
                    if strings.Compare(string(prev.Event.IndexSnapshot.Languages[lang]), string(curr.Event.IndexSnapshot.Languages[lang])) != 0 {
                        return "event languages key changed"
                    }
                }
            case prev.Event != nil && curr.Event != nil && prev.Event.MigrationRun != nil && curr.Event.MigrationRun != nil:
                if strings.Compare(string(prev.Event.MigrationRun.From), string(curr.Event.MigrationRun.From)) != 0 {
                    return "event from changed"
                }
                if strings.Compare(string(prev.Event.MigrationRun.To), string(curr.Event.MigrationRun.To)) != 0 {
                    return "event to changed"
                }
                if prev.Event.MigrationRun.FileCount != curr.Event.MigrationRun.FileCount {
                    return "event file count changed"
                }
            case prev.Event != nil && curr.Event != nil && prev.Event.TeamSession != nil && curr.Event.TeamSession != nil:
                if strings.Compare(string(prev.Event.TeamSession.Developer), string(curr.Event.TeamSession.Developer)) != 0 {
                    return "event developer changed"
                }
                if strings.Compare(string(prev.Event.TeamSession.TaskID), string(curr.Event.TeamSession.TaskID)) != 0 {
                    return "event task id changed"
                }
                if prev.Event.TeamSession.DurationSecs != curr.Event.TeamSession.DurationSecs {
                    return "event duration secs changed"
                }
                if prev.Event.TeamSession.LinesAdded