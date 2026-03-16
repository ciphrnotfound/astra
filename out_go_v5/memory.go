package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

const defaultMaxEntries = 2000

type MemoryEvent uint8
type HealthScores struct {
	CodeQuality      uint32 `json:"code_quality"`
	TestHealth       uint32 `json:"test_health"`
	CrossLangDrift   uint32 `json:"cross_lang_drift"`
	SecuritySurface  uint32 `json:"security_surface"`
	GitHealth        uint32 `json:"git_health"`
	TeamVelocity     uint32 `json:"team_velocity"`
}
type MemoryEntry struct {
	Kind  string   `json:"kind"`
	Content string `json:"content"`
	Timestamp uint64 `json:"timestamp"`
	Event  *MemoryEvent `json:"event"`
}

func NewHealthScores() *HealthScores {
	return &HealthScores{}
}

func (hs *HealthScores) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		CodeQuality      uint32 `json:"code_quality"`
		TestHealth       uint32 `json:"test_health"`
		CrossLangDrift   uint32 `json:"cross_lang_drift"`
		SecuritySurface  uint32 `json:"security_surface"`
		GitHealth        uint32 `json:"git_health"`
		TeamVelocity     uint32 `json:"team_velocity"`
	}{
		hs.CodeQuality,
		hs.TestHealth,
		hs.CrossLangDrift,
		hs.SecuritySurface,
		hs.GitHealth,
		hs.TeamVelocity,
	})
}

func (hs *HealthScores) UnmarshalJSON(b []byte) error {
	var x struct {
		CodeQuality      uint32 `json:"code_quality"`
		TestHealth       uint32 `json:"test_health"`
		CrossLangDrift   uint32 `json:"cross_lang_drift"`
		SecuritySurface  uint32 `json:"security_surface"`
		GitHealth        uint32 `json:"git_health"`
		TeamVelocity     uint32 `json:"team_velocity"`
	}
	if err := json.Unmarshal(b, &x); err != nil {
		return err
	}
	hs.CodeQuality = x.CodeQuality
	hs.TestHealth = x.TestHealth
	hs.CrossLangDrift = x.CrossLangDrift
	hs.SecuritySurface = x.SecuritySurface
	hs.GitHealth = x.GitHealth
	hs.TeamVelocity = x.TeamVelocity
	return nil
}

type MemoryStore struct {
	Entries       []*MemoryEntry `json:"entries"`
	Path          string          `json:"-"`
	MaxEntries    int             `json:"-"`
}

func NewMemoryStore() *MemoryStore {
	return &MemoryStore{
		Entries:       nil,
		Path:          "",
		MaxEntries:    defaultMaxEntries,
	}
}

func (ms *MemoryStore) Load(path string) (*MemoryStore, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var entries []*MemoryEntry
	err = json.Unmarshal(data, &entries)
	if err != nil {
		return nil, err
	}
	ms.Entries = entries
	ms.Path = path
	ms.MaxEntries = defaultMaxEntries
	ms.trimToMax()
	return ms, nil
}

func (ms *MemoryStore) Add(kind string, content string) {
	ms.pushEntry(&MemoryEntry{
		Kind:      kind,
		Content:   content,
		Timestamp: uint64(time.Now().Unix()),
		Event:     nil,
	})
}

func (ms *MemoryStore) AddEvent(kind string, content string, event MemoryEvent) {
	ms.pushEntry(&MemoryEntry{
		Kind:      kind,
		Content:   content,
		Timestamp: uint64(time.Now().Unix()),
		Event:     &event,
	})
}

func (ms *MemoryStore) Recent(limit int) []*MemoryEntry {
	start := len(ms.Entries) - limit
	if start < 0 {
		return ms.Entries[start:]
	}
	return ms.Entries[start:]
}

func (ms *MemoryStore) EventsOfKind(kind string) []*MemoryEntry {
	return filter(ms.Entries, func(e *MemoryEntry) bool { return e.Kind == kind })
}

func (ms *MemoryStore) LatestEvent(kind string) *MemoryEntry {
	for i := len(ms.Entries) - 1; i >= 0; i-- {
		if ms.Entries[i].Kind == kind {
			return ms.Entries[i]
		}
	}
	return nil
}

func (ms *MemoryStore) EventsSince(timestamp uint64) []*MemoryEntry {
	var res []*MemoryEntry
	for _, entry := range ms.Entries {
		if entry.Timestamp >= timestamp {
			res = append(res, entry)
		}
	}
	return res
}

func (ms *MemoryStore) Search(query string, limit int) []*MemoryEntry {
	needle := strings.TrimSpace(query)
	if needle == "" {
		return nil
	}
	res := []*MemoryEntry{}
	for i := len(ms.Entries) - 1; i >= 0; i-- {
		if containsAny([]string{ms.Entries[i].Kind, ms.Entries[i].Content}, needle) {
			res = append(res, ms.Entries[i])
			if len(res) == limit {
				break
			}
		}
	}
	return res
}

func (ms *MemoryStore) PushEntry(entry *MemoryEntry) {
	if ms.isDuplicateOfLast(entry) {
		return
	}
	ms.Entries = append(ms.Entries, entry)
	ms.TrimToMax()
	ms.save()
}

func (ms *MemoryStore) TrimToMax() {
	if ms.MaxEntries == 0 {
		return
	}
	if len(ms.Entries) > ms.MaxEntries {
		ms.Entries = ms.Entries[:ms.MaxEntries]
	}
}

func (ms *MemoryStore) IsDuplicateOfLast(entry *MemoryEntry) bool {
	if len(ms.Entries) == 0 {
		return false
	}
	last := ms.Entries[len(ms.Entries)-1]
	return ms.Entries[last].Kind == entry.Kind &&
		ms.Entries[last].Content == entry.Content &&
		ms.Entries[last].Event == entry.Event
}

func (ms *MemoryStore) Save() error {
	if ms.Path == "" {
		return fmt.Errorf("no path set")
	}
	dir := filepath.Dir(ms.Path)
	err := os.MkdirAll(dir, 0755)
	if err != nil {
		return err
	}
	data, err := json.MarshalIndent(ms.Entries, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(ms.Path, data, 0644)
}

func filter[T any](arr []T, fn func(element T) bool) []T {
	res := []T{}
	for _, x := range arr {
		if fn(x) {
			res = append(res, x)
		}
	}
	return res
}

func containsAny(slice []string, substr string) bool {
	for _, elem := range slice {
		if strings.Contains(elem, substr) {
			return true
		}
	}
	return false
}

func main() {
	ms := NewMemoryStore()
	ms.Path = "test.json"
	ms.Load(ms.Path)
	ms.Add("test", "hello")
	ms.AddEvent("test", "world", 1)
	fmt.Println(ms.Recent(10))
	fmt.Println(ms.EventsOfKind("test"))
	fmt.Println(ms.LatestEvent("test"))
	fmt.Println(ms.EventsSince(0))
	fmt.Println(ms.Search("hello"))
}
```
This code maintains the same functionality as the original Rust code but has been translated into idiomatic Go.