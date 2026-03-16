package main

import (
	"encoding/json"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/shirou/gopsutil/path"
)

type SecurityIssue struct {
	File        string
	LineNumber uint32
	Severity    string
	Description string
	Snippet     string
}

type SecurityReport struct {
	Issues                  []SecurityIssue
	FilesScanned            uint32
	AiAnalysis             string
}

const (
	highRiskPatterns = []struct {
		Pattern      string
		Severity     string
		Description  string
	}{
		{"api_key\\s*=", "High", "Hardcoded API key detected"},
		{"password\\s*=", "High", "Hardcoded password detected"},
		{"secret\\s*=", "High", "Hardcoded secret detected"},
		{"http://", "Medium", "Unencrypted HTTP connection used instead of HTTPS"},
		{"\\.execute\\(.*?\\+.*?", "High", "Potential SQL injection via string concatenation"},
	}

	simplePatterns = []struct {
		Pattern      string
		Severity     string
		Description  string
	}{
		{"api_key =", "High", "Hardcoded API Key"},
		{"api_key=", "High", "Hardcoded API Key"},
		{"password =", "High", "Hardcoded Password"},
		{"password=", "High", "Hardcoded Password"},
		{"secret =", "High", "Hardcoded Secret"},
		{"secret=", "High", "Hardcoded Secret"},
		{"http://", "Medium", "Unencrypted HTTP (should be HTTPS)"},
		{"SELECT * FROM", "Medium", "Raw SQL query detected (check for injection risk)"},
	}
)

type CodexModel interface {
	Complete(text string) (string, error)
}

type index struct {
	files []struct {
		RelPath  string
		Summary  string
		AbsPath  string
		Contents string
	}
}

type model struct{}

func runSecurityScan(root, indexPath, modelPath string) (*SecurityReport, error) {
	var issues []SecurityIssue
	var filesScanned uint32

	// Load index
	var idx index
	data, err := ioutil.ReadFile(indexPath)
	if err != nil {
		return nil, err
	}
	json.Unmarshal(data, &idx)

	// Convert primitive patterns into something we can check easily
	for _, issue := range idx.files {
		filesScanned++
		if err := path.Stat(issue.AbsPath); err != nil {
			log.Printf("Skipping non-existent file: %s\n", issue.AbsPath)
			continue
		}
		if contents, err := ioutil.ReadFile(issue.AbsPath); err != nil {
			log.Printf("Error reading file: %s\n", issue.AbsPath)
			continue
		} else {
			issue.Contents = string(contents)
		}

		// Simple substring/contains checks
		for _, pattern := range simplePatterns {
			if strings.Contains(strings.ToLower(issue.Contents), pattern.Pattern) {
				if !strings.Contains(issue.AbsPath, "security.go") &&
					!strings.Contains(issue.AbsPath, ".env.example") {
					issues = append(issues, SecurityIssue{
						File:        issue.AbsPath,
						LineNumber: 0,
						Severity:   pattern.Severity,
						Description: pattern.Description,
						Snippet:     strings.Split(strings.Trim(issue.Contents, "\n\r"), "\n")[0],
					})
				}
			}
		}
	}

	var aiAnalysis string
	if len(issues) > 0 {
		// Complete model interface
		var m model
		prompt := fmt.Sprintf("You are an expert security researcher. Review these potential vulnerabilities found by a static scanner and summarize the overall risk level and next steps for the developer:\n\n")
		for i, issue := range issues[:15] {
			prompt += fmt.Sprintf("%d. [{}] %s:%d - %s\n Code: %s\n", i+1, issue.Severity, issue.File, issue.LineNumber, issue.Description, issue.Snippet)
		}
		if aiAnalysis, err = m.complete(prompt); err != nil {
			log.Printf("Error completing analysis: %s\n", err)
		}
	} else {
		return &SecurityReport{issues: issues}, nil
	}

	return &SecurityReport{issues: issues, FilesScanned: filesScanned, AiAnalysis: aiAnalysis}, nil
}

func main() {
	root := "."
	indexPath := "./index.json"
	modelPath := "./model"
	repo := &SecurityReport{}
	err := runSecurityScan(root, indexPath, modelPath)
	if err != nil {
		log.Fatal(err)
	}
	repo.render()
}

type stringList []string

func (s stringList) Len() int           { return len(s) }
func (s stringList) Swap(i, j int)      { s[i], s[j] = s[j], s[i] }
func (s stringList) Less(i, j int) bool { return s[i] < s[j] }

func main() {
	// Render report
	sort.Strings([]string{repo.Issues[0].File})
	report := fmt.Sprintf("🛡️  **SECURITY HUNTER REPORT** 🛡️\n")
	report += fmt.Sprintf("Scanned %d files.\n", repo.FilesScanned)
	if len(repo.Issues) == 0 {
		report += "✅ No obvious security vulnerabilities found."
		return
	}
	highCount := 0
	medCount := 0
	for _, issue := range repo.Issues {
		if issue.Severity == "High" {
			highCount++
		} else if issue.Severity == "Medium" {
			medCount++
		}
	}
	report += fmt.Sprintf("Found %d vulnerabilities (%d High, %d Medium).\n", len(repo.Issues), highCount, medCount)
	for _, issue := range repo.Issues {
		severityMap := map[string]string{"High": "🔴", "Medium": "🟡"}
		repr := severityMap[issue.Severity]
		if !strings.Contains(issue.File, "security.go") && !strings.Contains(issue.File, ".env.example") {
			var description, snippet string
			if index, err := filepath.Abs(issue.File); err != nil {
				index = issue.File
			}
			if _, err := os.Stat(index); err != nil {
				log.Fatal(err)
			}
			data, err := ioutil.ReadFile(index)
			if err != nil {
				log.Fatal(err)
			}
			description = string(data)
			snippet = strings.Split(strings.Trim(string(data), "\n\r"), "\n")[0]
			report += fmt.Sprintf("%s **%s** in `%s %d`\n", repr, issue.Severity, issue.File, issue.LineNumber)
			report += fmt.Sprintf("   Issue: %s\n", issue.Description)
			report += fmt.Sprintf("   Code:\n%s\n", snippet)
		}
	}
	if repo.AiAnalysis != "" {
		report += fmt.Sprintf("\n🤖 **AI Security Analysis:**\n%s\n", repo.AiAnalysis)
	} else {
		report += fmt.Sprintf("\n*(Run with an LLM enabled for deeper AI analysis of these findings)*")
	}
	fmt.Println(report)
}