package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"

	"github.com/google/go-cmp/cmp"
)

type HealthReport struct {
	Scores    HealthScores
	PrevScores *HealthScores
	Details  HealthDetails
}

type HealthScores struct {
	CodeQuality   uint32
	TestHealth    uint32
	CrossLangDrift uint32
	SecuritySurface uint32
	GitHealth     uint32
	TeamVelocity  uint32
}

type HealthDetails struct {
	TodoCount      uint32
	TotalLines    uint32
	TestFiles      uint32
	TotalFiles    uint32
	LanguageCount uint32
	MigrationCount uint32
	SecurityFiles uint32
	UncommittedChanges uint32
	RecentCommits uint32
	TasksDone uint32
	TasksTotal uint32
}

type CodeIndex struct {
	Stats  Stats
 Files map[string][]string
	FilesByLang map[string][]string
}

type Stats struct {
	TotalLines uint32
	FileCount  uint32
}

func computeHealth(root string, index CodeIndex, memory map[string][]byte, teamMgr *TeamManager) HealthReport {
	stats := index.Stats
	files := index.Files
	byLang := index.FilesByLang

	todoCount := uint32(0)
	for _, contents := range files {
		for _, line := range contents {
			match := regexp.MustCompile("TODO|FIXME|HACK").FindStringSubmatch(line)
			if len(match) > 0 {
				todoCount++
			}
		}
	}
	var totalLines uint32
	for _, contents := range files {
		totalLines += uint32(len(contents))
	}
	todoRatio := float64(todoCount) / float64(totalLines) * 500.0
	codeQuality := (100.0 - todoRatio).clamp(0.0, 100.0) as uint32

	testFiles := uint32(0)
	for _, name := range files {
		if strings.Contains(strings.ToLower(name), "test") || strings.Contains(strings.ToLower(name), "spec") || strings.Contains(strings.ToLower(name), "tests") {
			testFiles++
		}
	}
	testRatio := float64(testFiles) / float64(stats.FileCount) * 500.0
	testHealth := (testRatio).clamp(0.0, 100.0) as uint32

	languageCount := uint32(0)
	for _, lang := range byLang {
		if contains(CODE_LANGUAGES, lang) {
			languageCount++
		}
	}
	migrationEvents := memory["migration"]
	migrationCount := uint32(len(migrationEvents))
	driftPenalty := (languageCount - 1) * 15
	migrationBonus := migrationCount * 5
	crossLangDrift := (100 - driftPenalty + migrationBonus).clamp(0, 100) as uint32

	securityFiles := uint32(0)
	for _, contents := range files {
		match := regexp.MustCompile("password|secret|token|auth|credential|api_key|apikey|private_key|jwt").FindStringSubmatch(contents)
		if len(match) > 0 {
			securityFiles++
		}
	}
	secRatio := float64(securityFiles) / float64(stats.FileCount) * 200.0
	securitySurface := (100.0 - secRatio).clamp(0.0, 100.0) as uint32

	uncommittedChanges := uint32(0)
	recentCommits := uint32(0)
	if teamMgr != nil {
		state := teamMgr.State()
		uncommittedChanges = state.UncommittedChanges
		recentCommits = state.RecentCommits
	}
	commitScore := (recentCommits as float64 * 3.3).clamp(0.0, 100.0)
	uncommittedPenalty := (uncommittedChanges as float64 * 2.0).clamp(0.0, 40.0)
	gitHealth := (commitScore - uncommittedPenalty).clamp(0.0, 100.0) as uint32

	tasksDone := uint32(0)
	tasksTotal := uint32(0)
	if teamMgr != nil {
		tasksTotal = teamMgr.TasksTotal()
		tasksDone = teamMgr.TasksDone()
	}
	teamVelocity := (int32(tasksDone) / int32(tasksTotal) * 100.0) as uint32

	scores := HealthScores{
		CodeQuality:   codeQuality,
		TestHealth:    testHealth,
		CrossLangDrift: crossLangDrift,
		SecuritySurface: securitySurface,
		GitHealth:     gitHealth,
		TeamVelocity:  teamVelocity,
	}

	prevScores := memoryLatestEvent("health")
	if prevScores != nil {
		prev := HealthScores{
			CodeQuality:   prevScores.CodeQuality,
			TestHealth:    prevScores.TestHealth,
			CrossLangDrift: prevScores.CrossLangDrift,
			SecuritySurface: prevScores.SecuritySurface,
			GitHealth:     prevScores.GitHealth,
			TeamVelocity:  prevScores.TeamVelocity,
		}
		scores = prev
	}

	return HealthReport{
		Scores: scores,
		PrevScores: &scores,
		Details: HealthDetails{
			TodoCount:      todoCount,
			TotalLines:    totalLines,
			TestFiles:      testFiles,
			TotalFiles:    stats.FileCount,
			LanguageCount: languageCount,
			MigrationCount: migrationCount,
			SecurityFiles: securityFiles,
			UncommittedChanges: uncommittedChanges,
			RecentCommits: recentCommits,
			TasksDone: tasksDone,
			TasksTotal: tasksTotal,
		},
	}
}

func contains(s []string, e string) bool {
	for _, a := range s {
		if a == e {
			return true
		}
	}
	return false
}

func clamp(n float64, lower, upper float64) float64 {
	if n < lower {
		return lower
	} else if n > upper {
		return upper
	} else {
		return n
	}
}

func memoryLatestEvent(prefix string) *HealthScores {
	// simulate memory access
	events := map[string][]byte{
		"health": []byte(`{"CodeQuality": 90, "TestHealth": 80, "CrossLangDrift": 70, "SecuritySurface": 60, "GitHealth": 90, "TeamVelocity": 80}`),
	}
	for _, event := range events {
		match := regexp.MustCompile(`{"CodeQuality":\s*([0-9]+),\s*"TestHealth":\s*([0-9]+),\s*"CrossLangDrift":\s*([0-9]+),\s*"SecuritySurface":\s*([0-9]+),\s*"GitHealth":\s*([0-9]+),\s*"TeamVelocity":\s*([0-9]+)")`).FindStringSubmatch(string(event))
		if match != nil && prefix == match[1] {
			return &HealthScores{
				CodeQuality:   uint32(match[2]),
				TestHealth:    uint32(match[3]),
				CrossLangDrift: uint32(match[4]),
				SecuritySurface: uint32(match[5]),
				GitHealth:     uint32(match[6]),
				TeamVelocity:  uint32(match[7]),
			}
		}
	}
	return nil
}

func (h *HealthReport) Render() string {
	out := strings.Builder{}
	fmt.Println("╔══════════════════════════════════════════════════════╗")
	fmt.Println("║            CODEBASE HEALTH REPORT                   ║")
	fmt.Println("╠══════════════════════════════════════════════════════╣")

	metrics := []struct {
		Name string
		Score uint32
	}{
		{"Code Quality", h.Scores.CodeQuality},
		{"Test Health", h.Scores.TestHealth},
		{"Cross-Lang Drift", h.Scores.CrossLangDrift},
		{"Security Surface", h.Scores.SecuritySurface},
		{"Git Health", h.Scores.GitHealth},
		{"Team Velocity", h.Scores.TeamVelocity},
	}

	prevValues := h.PrevScores
	prev := map[string]uint32{}
	if prevValues != nil {
		prev = map[string]uint32{
			"Code Quality":   prevValues.CodeQuality,
			"Test Health":    prevValues.TestHealth,
			"Cross-Lang Drift": prevValues.CrossLangDrift,
			"Security Surface": prevValues.SecuritySurface,
			"Git Health":     prevValues.GitHealth,
			"Team Velocity":  prevValues.TeamVelocity,
		}
	}

	sort.Slice(metrics, func(i, j int) bool {
		return metrics[i].Score > metrics[j].Score
	})

	for i, metric := range metrics {
		score := uint32(100)
		s := "█"
		for j := 0; j < metric.Score; j++ {
			s += "█"
		}
		for ; j < 100; j++ {
			s += "░"
		}
		trend := ""
		if prev[metric.Name] != 0 {
			if metric.Score > prev[metric