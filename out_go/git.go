package main

import (
	"bytes"
	"fmt"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	git "github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/pkg/errors"
)

const (
	HEAD = "HEAD"
)

type GitRepo struct {
	root string
	repo *git.Repository
}

type CommitSummary struct {
	id      string
	summary string
	author  string
	time    time.Time
}

type CommitInfo struct {
	id      string
	summary string
	author  string
	date    string
}

func NewGitRepo(repoPath string) (*GitRepo, error) {
	repo, err := git.PlainOpen(repoPath)
	if err != nil {
		return nil, err
	}
	return &GitRepo{root: filepath.Clean(repoPath), repo: repo}, nil
}

func (gr *GitRepo) RecentCommitCount(limit int) int {
	out, err := exec.Command(
		"git", "-C", gr.root,
		"rev-list", "--count", "-n",
		strconv.Itoa(limit), "HEAD",
	).Output()
	if err != nil {
		return 0
	}
	count, _ := strconv.Atoi(strings.TrimSpace(string(out)))
	return count
}

func (gr *GitRepo) TotalCommitCount() int {
	out, err := exec.Command(
		"git", "-C", gr.root,
		"rev-list", "--count", "HEAD",
	).Output()
	if err != nil {
		return 0
	}
	count, _ := strconv.Atoi(strings.TrimSpace(string(out)))
	return count
}

func (gr *GitRepo) UncommittedFileCount() int {
	args := []string{"git", "-C", gr.root, "status", "--porcelain"}
	out, err := exec.Command(args[0], args[1:]...).Output()
	if err != nil {
		return 0
	}
	lines := strings.Split(strings.TrimSpace(string(out)), "\n")
	count := 0
	for _, l := range lines {
		if strings.TrimSpace(l) != "" {
			count++
		}
	}
	return count
}

func (gr *GitRepo) ChangedFiles() ([]string, error) {
	args := []string{"git", "-C", gr.root, "status", "--porcelain"}
	out, err := exec.Command(args[0], args[1:]...).Output()
	if err != nil {
		return nil, err
	}
	var files []string
	for _, line := range strings.Split(string(out), "\n") {
		if len(line) > 3 {
			files = append(files, strings.TrimSpace(line[3:]))
		}
	}
	return removeDuplicates(files), nil
}

func (gr *GitRepo) RootPath() string {
	return gr.root
}

func (gr *GitRepo) GetHeadCommit() (string, error) {
	head, err := gr.repo.Head()
	if err != nil {
		return "", errors.Wrap(err, "repository has no HEAD")
	}
	return head.Hash().String(), nil
}

func (gr *GitRepo) GetDiffStats(fromCommit string) (int, int, error) {
	out, err := exec.Command("git", "-C", gr.root, "diff", "--numstat", fromCommit).Output()
	if err != nil {
		return 0, 0, errors.Wrap(err, "failed to run git diff")
	}
	var added int
	var deleted int
	lines := strings.Split(string(out), "\n")
	for _, l := range lines {
		p := strings.Fields(l)
		if len(p) >= 2 {
			if a, err := strconv.Atoi(p[0]); err == nil {
				added += a
			}
			if d, err := strconv.Atoi(p[1]); err == nil {
				deleted += d
			}
		}
	}
	return added, deleted, nil
}

func (gr *GitRepo) GetCommitsByAuthor(author string, limit int) (string, error) {
	arg := fmt.Sprintf("--author=%s", author)
	out, err := exec.Command("git", "-C", gr.root, "log", arg, "-p", "-n"+strconv.Itoa(limit)).Output()
	if err != nil {
		return "", errors.Wrapf(err, "failed to get commits for author %s", author)
	}
	return string(out), nil
}

func (gr *GitRepo) LastCommitInfo() (CommitInfo, error) {
	out, err := exec.Command("git", "-C", gr.root, "log", "-1", "--format=%H|%an|%ad|%s", "--date=iso-strict").Output()
	if err != nil {
		return CommitInfo{}, errors.Wrap(err, "failed to get last commit")
	}
	parts := strings.SplitN(string(out), "|", 4)
	if len(parts) < 4 {
		return CommitInfo{}, fmt.Errorf("unexpected git log output format")
	}
	cInfo := CommitInfo{
		id:      strings.TrimSpace(parts[0]),
		author:  strings.TrimSpace(parts[1]),
		date:    strings.TrimSpace(parts[2]),
		summary: strings.TrimSpace(parts[3]),
	}
	return cInfo, nil
}

func (gr *GitRepo) RecentCommits(limit int) ([]CommitSummary, error) {
	iter, err := gr.repo.Log(&git.LogOptions{From: plumbing.ZeroHash}) // Simplified for example
	if err != nil {
		// Try from HEAD
		head, err := gr.repo.Head()
		if err != nil {
			return nil, err
		}
		iter, err = gr.repo.Log(&git.LogOptions{From: head.Hash()})
		if err != nil {
			return nil, err
		}
	}
	var commits []CommitSummary
	err = iter.ForEach(func(c *object.Commit) error {
		commits = append(commits, gr.summarizeCommit(c))
		if len(commits) >= limit {
			return fmt.Errorf("limit reached")
		}
		return nil
	})
	if err != nil && err.Error() != "limit reached" {
		return nil, err
	}
	return commits, nil
}

func (gr *GitRepo) RecentCommitsForPath(relPath string, limit int) ([]CommitSummary, error) {
	head, err := gr.repo.Head()
	if err != nil {
		return nil, err
	}
	iter, err := gr.repo.Log(&git.LogOptions{From: head.Hash(), FileName: &relPath})
	if err != nil {
		return nil, err
	}
	var commits []CommitSummary
	err = iter.ForEach(func(c *object.Commit) error {
		commits = append(commits, gr.summarizeCommit(c))
		if len(commits) >= limit {
			return fmt.Errorf("limit reached")
		}
		return nil
	})
	if err != nil && err.Error() != "limit reached" {
		return nil, err
	}
	return commits, nil
}

func (gr *GitRepo) summarizeCommit(commit *object.Commit) CommitSummary {
	return CommitSummary{
		id:      shortOID(commit.Hash),
		summary: strings.TrimSpace(commit.Message),
		author:  commit.Author.Name,
		time:    commit.Author.When,
	}
}

func shortOID(oid plumbing.Hash) string {
	return oid.String()[:8]
}

func removeDuplicates(a []string) []string {
	m := make(map[string]bool)
	result := []string{}
	for _, s := range a {
		if _, ok := m[s]; !ok {
			result = append(result, s)
			m[s] = true
		}
	}
	return result
}