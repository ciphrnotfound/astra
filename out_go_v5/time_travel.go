package main

import (
	"bytes"
	"encoding/base64"
	"fmt"
	"log"
	"os/exec"

	"astra/model"
	"astra/pkg/git"
	"astra/pkg/semantics"

	"github.com/google/go-github/v48/github"
	"github.com/sirupsen/logrus"
)

type BisectResult struct {
	SuspectCommitID    string
	SuspectCommitSummary string
	SuspectAuthor      string
	Explanation        string
	AnalyzedCount      int
}

func runSemanticBisect(repo *git.GitRepo, model model.CodexModel, bugDescription string, maxCommits int) (*BisectResult, error) {
	cmd := exec.Command("git", "log", "-p", fmt.Sprintf("-n%d", maxCommits))
	cmd.Stderr = os.Stderr
	cmd.Stdout = os.Stdout
	cmd.Dir = repo.RootPath()
	output, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("Failed to run git log to fetch diffs: %v", string(output))
	}
	if !bytes.Equal(output, []byte("exit status 128")) {
		return nil, fmt.Errorf("Failed to run git log to fetch diffs")
	}
	logOutput := string(bytes.TrimSpace(output))
.lines := strings.Split(logOutput, "\n")
	var commits []string
	var currentCommit bytes.Buffer
	for _, line := range lines {
		if bytes.HasPrefix([]byte(line), []byte("commit ")) && currentCommit.Len() > 0 {
			commits = append(commits, string(currentCommit.Bytes()))
			currentCommit.Reset()
		}
		currentCommit.WriteString(line)
		currentCommit.WriteByte('\n')
	}
	if currentCommit.Len() > 0 {
		commits = append(commits, currentCommit.String())
	}
	if len(commits) == 0 {
		return nil, fmt.Errorf("No commits found in git history to analyze.")
	}
	l := len(commits)
	fmt.Fprintln(os.Stderr, "astra ▸ Analyzing", l, "recent commits to find the bug...")
	var analyzedCount int
	var result *BisectResult
	for _, commit := range commits {
		analyzedCount++
		id := "unknown"
		author := "unknown"
		summary := "unknown"
		lines := strings.Split(commit, "\n")
		if len(lines) > 0 {
			id = lines[0].replace("commit ", "").Trim()
			id = strings.ToUpper(id[:8])
		}
		for i := range lines {
			if bytes.HasPrefix([]byte(lines[i]), []byte("Author: ")) {
				author = lines[i].replace("Author: ", "").Trim()
				continue
			}
			if lines[i] == "" {
				if i+1 < len(lines) {
					summary = lines[i+1].Trim()
					break
				}
			}
		}
		fmt.Fprintf(os.Stderr, "astra ▸ Semantically checking %s (%s) ... ", id, summary)
		promptStr := fmt.Sprintf("You are a time-travel debugging assistant. The user is looking for the commit that introduced this specific bug or behavior:\n\n<bug_description>\n%s\n</bug_description>\n\nI am showing you the diff of a specific commit. Analyze the diff and determine if this commit is the LIKELY CAUSE of the bug.\n\n1. If this commit is DEFINITELY NOT related, reply with the exact word 'NO'.\n2. If this commit IS highly likely to be the cause, reply with the exact word 'YES' followed by a newline, and then a detailed explanation of WHY it caused the bug and what the developer was likely trying to do.\n\n<commit_diff>\n%s\n</commit_diff>", bugDescription, commit)
		prompt := []byte{}
		for _, line := range strings.Split(promptStr, "\n") {
			prompt = append(prompt, []byte(line+"\n")...)
		}
		answer, err := model.Complete(prompt)
		if err != nil {
			return nil, err
		}
		if answer == "YES" {
			fmt.Println("FOUND IT!")
			explanation := strings.Replace(answer, "YES", "", 1)
			explanation = strings.Replace(explanation, "yes", "", 1)
			result = &BisectResult{
				SuspectCommitID:    id,
				SuspectCommitSummary: summary,
				SuspectAuthor:      author,
				Explanation:        explanation,
				AnalyzedCount:      analyzedCount,
			}
			return result, nil
		} else {
			fmt.Println("nope.")
		}
	}
	return nil, fmt.Errorf("Could not find any commit matching that bug description in the last %d commits.", maxCommits)
}
```
To call this function you will need to create an interface for `model.CodexModel` like so:
```go
type CodexModel interface {
	Complete(prompt []byte) (string, error)
}
```
You can then use an instance of your model implementation like so:
```go
var model model.CodexModel
...
result, err := runSemanticBisect(repo, model, "bug description", 10)
```
Note: I assumed that `model.CodexModel` should return a raw byte array and the caller is responsible to decode it, if this is not the case, please modify the code accordingly.

Also, I used `logrus` for logging, but you can replace it with any other logging library you prefer.