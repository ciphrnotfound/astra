package main

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

type HealthReport struct {
	Scores     HealthScores
	PrevScores *HealthScores
	Details    HealthDetails
}

type HealthScores struct {
	CodeQuality      uint32
	TestHealth       uint32
	CrossLangDrift   uint32
	SecuritySurface  uint32
	GitHealth       uint32
	TeamVelocity     uint32
}

type HealthDetails struct {
	TodoCount       uint64
	TotalLines      uint64
	TestFiles       uint64
	TotalFiles      uint64
	LanguageCount   uint64
	MigrationCount  uint64
	SecurityFiles   uint64
	UncommittedChanges uint64
	RecentCommits   uint64
	TasksDone       uint64
	TasksTotal      uint64
}

const (
	securityKeywords = [...]string{
		"password", "secret", "token", "auth", "credential",
		"api_key", "apikey", "private_key", "jwt",
	}
	crudLanguages = [...]string{
		"rust", "typescript", "javascript", "python", "go", "java",
		"c", "cpp", "csharp", "ruby", "swift", "kotlin", "scala",
	}
)

func computeHealth(root string, index *Index, memory *MemoryStore, teamMgr *TeamManager) *HealthReport {
	stats, files, byLang := index.stats(), index.files(), index.filesByLanguage()

	todoCount := uint64(0)
	for path, _ := range files {
		absPath := filepath.Join(root, path)
		if content, err := os.ReadFile(absPath); err == nil {
			for _, line := range strings.Split(string(content), "\n") {
				line = strings.ToUpper(line)
				if strings.Contains(line, "TODO") || strings.Contains(line, "FIXME") || strings.Contains(line, "HACK") {
					todoCount++
				}
			}
		}
	}

	todoRatio := float64(0)
	if stats.TotalLines > 0 {
		todoRatio = float64(todoCount) / float64(stats.TotalLines)
	}
	codeQuality := uint32(100 - (todoRatio * 500))

	testFiles := uint64(0)
	for _, path := range files {
		name := strings.ToLower(path)
		if strings.Contains(name, "test") || strings.Contains(name, "spec") || strings.Contains(name, "tests") {
			testFiles++
		}
	}

	testRatio := float64(0)
	if stats.FileCount > 0 {
		testRatio = float64(testFiles) / float64(stats.FileCount)
	}
	testHealth := uint32(testRatio * 500)

	languageCount := uint64(0)
	for lang := range byLang {
		if idx := contains(crudLanguages[:], lang); idx != -1 {
		.languageCount++
		}
	}
	migrationEvents := memory.EventsOfKind("migration")
	migrationCount := uint64(len(migrationEvents))
	driftPenalty := (languageCount - 1) * 15
	migrationBonus := migrationCount * 5
	crossLangDrift := uint32(100 - (driftPenalty + migrationBonus))

	secFiles := uint64(0)
	for _, path := range files {
		absPath := filepath.Join(root, path)
		if content, err := os.ReadFile(absPath); err == nil {
			lower := strings.ToLower(string(content))
			if contains(securityKeywords[:], lower) {
				secFiles++
			}
		}
	}

	secRatio := float64(0)
	if stats.FileCount > 0 {
		secRatio = float64(secFiles) / float64(stats.FileCount)
	}
	securitySurface := uint32(100 - (secRatio * 200))

	uncommittedChanges, recentCommits := uint64(0), uint64(0)
	if repo, err := DiscoverGitRepo(root); err == nil {
		uncommittedChanges = repo.UncommittedFileCount()
		recentCommits = repo.RecentCommitCount(30)
	}

	commitScore := (recentCommits * 3.3)
	uncommittedPenalty := (uncommittedChanges * 2)
	gitHealth := uint32(commitScore - uncommittedPenalty)

	tasksDone, tasksTotal := uint64(0), uint64(0)
	if teamMgr != nil {
		if state, err := teamMgr.LoadState(); err == nil {
			tasksTotal = uint64(len(state.Tasks))
			tasksDone = count(
				state.Tasks,
				func(t Task) bool { return t.Status == TaskStatusDone },
			)
		}
	}

	teamVelocity := uint32(0)
	if tasksTotal > 0 {
		teamVelocity = uint32((float64(tasksDone) / float64(tasksTotal)) * 100)
	} else {
		teamVelocity = 50
	}

	prevScores := memory.LatestEvent("health")
	if prevScores != nil {
		prevScores.Map(func(event MemoryEvent) interface{} {
			if h, ok := event.Value.(*HealthSnapshot); ok {
				return h.Scores
			}
			return nil
		})
		prevScores.Map(func(event MemoryEvent) interface{} {
			if h, ok := event.Value.(*HealthSnapshot); ok {
				return h.Scores
			}
			return nil
		})
		prevScores.Map(func(event MemoryEvent) interface{} {
			if h, ok := event.Value.(*HealthSnapshot); ok {
				return h.Scores
			}
			return nil
		})
		prevScores.Map(func(event MemoryEvent) interface{} {
			if h, ok := event.Value.(*JSON); ok {
				prevScores = prevScores.Map(func(event MemoryEvent) interface{} {
					return h.Data
				})
			}
			prevScores = prevScores.Map(func(event MemoryEvent) interface{} {
				return h.Data
			})

			prevScores.Map(func(event MemoryEvent) interface{} {
				return h.Scores
			})
			prevScores.Map(func(event MemoryEvent) interface{} {
				return h.Scores
			})

			prevScores.Map(func(event MemoryEvent) interface{} {
				if h, ok := event.Value.(*HealthScores); ok {
					return h
				}
				return nil
			})
			prevScores.Map(func(event MemoryEvent) interface{} {
				if h, ok := event.Value.(*HealthScores); ok {
					return h
				}
				return nil
			})
			prevScores.Map(func(event MemoryEvent) interface{} {
				if h, ok := event.Value.(*HealthScores); ok {
					return h
				}
				return nil
			})
			prevScores.Map(func(event MemoryEvent) interface{} {
				if h, ok := event.Value.(*HealthScores); ok {
					return h
				}
				return nil
			})
			prevScores.Map(func(event MemoryEvent) interface{} {
				if h, ok := event.Value.(*HealthSnapshot); ok {
					if h.Scores != nil {
						return h.Scores
					}
					return nil
				}
				return nil
			})

			prevScores.Map(func(event MemoryEvent) interface{} {
				if h, ok := event.Value.(*HealthSnapshot); ok {
					if h.Scores != nil {
						return h.Scores
					}
					return nil
				}
				return nil
			})

		})
		prevScores.Map(func(event MemoryEvent) interface{} {
			if h, ok := event.Value.(*HealthSnapshot); ok {
				return h.Scores
			}
			return nil
		})
	}

	return &HealthReport{
		Scores: HealthScores{
			CodeQuality:      codeQuality,
			TestHealth:       testHealth,
			CrossLangDrift:   crossLangDrift,
			SecuritySurface:  securitySurface,
			GitHealth:        gitHealth,
			TeamVelocity:     teamVelocity,
		},
		PrevScores: prevScores,
		Details: HealthDetails{
			TodoCount:       todoCount,
			TotalLines:      stats.TotalLines,
			TestFiles:       testFiles,
			TotalFiles:      stats.FileCount,
			LanguageCount:   languageCount,
			MigrationCount:  migrationCount,
			SecurityFiles:   secFiles,
			UncommittedChanges: uncommittedChanges,
			RecentCommits:   recentCommits,
			TasksDone:       tasksDone,
			TasksTotal:      tasksTotal,
		},
	}
}

func trendArrow(current, prev uint32) string {
	if current > prev+3 {
		return "▲"
	} else if current+3 < prev {
		return "\u25BC"
	} else {
		return "\u25C9"
	}
}

func trendDelta(current, prev uint32) string {
	diff := int32(current) - int32(prev)
	if diff > 0 {
		return fmt.Sprintf("(%d)", diff)
	} else if diff < 0 {
		return fmt.Sprintf("(%)", diff)
	} else {
		return ""
	}
}

func scoreBar(score uint32) string {
	switch {
	case score >= 90 && score <= 100:
		return strings.Repeat("\u2588", 10)
	case score >= 80 && score < 90:
		return strings.Repeat("\u2588", 9) + "\u2591"
	case score >= 70 && score < 80:
		return strings.Repeat("\u2588",