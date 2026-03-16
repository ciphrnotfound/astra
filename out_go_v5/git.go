package main

import (
	"fmt"
	"io/ioutil"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/storer"
	"github.com/go-git/go-git/v5/storage/memory"
)

type GitRepo struct {
	rootPath string
	repo     *git.Repository
}

type CommitSummary struct {
	id        string
	summary   string
	author    string
	timestamp int64
}

type CommitInfo struct {
	id        string
	summary   string
	author    string
	date      string
}

func NewGitRepo(rootPath string) (*GitRepo, error) {
	repo, err := git.Clone(memory.NewReturnEmptyStorage(), filepath.Dir(rootPath), &git.CloneOptions{
		URL:                   "",
		Depth:                 1,
		SingleBranch:          true,
		RecurseSubmodules:     git.DefaultRecursionDegree,
		RemoteName:            "origin",
		Bare:                  false,
		Progress:              true,
		DepthSpec:             "",
		NoHardwireCredentials: true,
	})
	if err != nil {
		return nil, err
	}
	workdir := filepath.Dir(rootPath)
	err = git.Clean(repo, workdir)
	if err != nil {
		return nil, err
	}
	return &GitRepo{rootPath: filepath.Dir(rootPath), repo: repo}, nil
}

func (g *GitRepo) RecentCommitCount(limit int) int {
	walk := g.repo.RevWalk(storer.NewTreeWalk(g.repo.Storage()))
	if walk.Step(&plumbing.NewCommit(walk)) {
		if !walk.Step(&plumbing.NewCommit(walk)) {
			return 0
		}
	}
	return walk.Take(limit).Count()
}

func (g *GitRepo) TotalCommitCount() int {
	walk := g.repo.RevWalk(storer.NewTreeWalk(g.repo.Storage()))
	if walk.Step(&plumbing.NewCommit(walk)) {
		if !walk.Step(&plumbing.NewCommit(walk)) {
			return 0
		}
	}
	return walk.Count()
}

func (g *GitRepo) UncommittedFileCount() int {
	statuses, err := g.repo.Status("", &git.StatusOptions{})
	if err != nil {
		return 0
	}
	return len(statuses)
}

func (g *GitRepo) ChangedFiles() []string {
	statuses, err := g.repo.Status("")
	if err != nil {
		return []string{}
	}
	files := []string{}
	for _, entry := range statuses {
		if entry.IsWorktreeModified() {
			path := filepath.Clean(entry.Filespace())
			files = append(files, path)
		}
	}
	return files
}

func (g *GitRepo) RootPath() string {
	return g.rootPath
}

func (g *GitRepo) GetHeadCommit() (string, error) {
	head, err := g.repo.Head()
	if err != nil {
		return "", err
	}
	headOid, err := head.Name()
	if err != nil {
		return "", err
	}
	cmd := exec.Command("git", "rev-parse", headOid)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", err
	}
	return string(out), nil
}

func (g *GitRepo) GetDiffStats(fromCommit string) (int, int, error) {
	cmd := exec.Command("git", "diff", "--numstat", fromCommit)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return 0, 0, err
	}
	output := string(out)
	addedLine := 0
	deletedLine := 0
	for _, line := range strings.Split(output, "\n") {
		parts := strings.Fields(line)
		if len(parts) >= 2 {
			if i, err := strconv.Atoi(parts[0]); err == nil {
				addedLine += i
			}
			if i, err := strconv.Atoi(parts[1]); err == nil {
				deletedLine += i
			}
		}
	}
	return addedLine, deletedLine, nil
}

func (g *GitRepo) GetCommitsByAuthor(authorName string, limit int) (string, error) {
	output, err := g.run("git", "log", "--author="+authorName, "-p", fmt.Sprintf("-n%d", limit))
	if err != nil {
		return "", err
	}
	return output, nil
}

func (g *GitRepo) LastCommitInfo() (CommitInfo, error) {
	output, err := g.run("git", "log", "-1", "--format=%H|%an|%ad|%s", "--date=iso-strict")
	if err != nil {
		return CommitInfo{}, err
	}
	commitInfo := strings.Split(string(output), "|")
	if len(commitInfo) != 4 {
		return CommitInfo{}, fmt.Errorf("Invalid commit information")
	}
	return CommitInfo{
		id:        commitInfo[0],
		summary:   commitInfo[3],
		author:    commitInfo[1],
		date:      commitInfo[2],
	}, nil
}

func (g *GitRepo) RecentCommits(limit int) ([]CommitSummary, error) {
	commitSummaries := []CommitSummary{}
	tree := storer.NewTreeWalk(g.repo.Storage())
	commitOid, err := g.repo.Head().Name()
	if err != nil {
		return nil, err
	}
	if err := tree.Step(&plumbing.NewCommit(g.repo.Storage(), commitOid)); err != nil {
		return nil, err
	}
	for limit > 0 {
		commitOid, err := tree.Parent(0)
		if err != nil {
			break
		}
		commit, err := g.repo.FindCommit(commitOid)
		if err != nil {
			break
		}
		commitSummaries = append(commitSummaries, g.SummarizeCommit(commit))
		limit--
	}
	return commitSummaries, nil
}

func (g *GitRepo) RecentCommitsForPath(relPath string, limit int) ([]CommitSummary, error) {
	tree := storer.NewTreeWalk(g.repo.Storage())
	commitOid, err := g.repo.Head().Name()
	if err != nil {
		return nil, err
	}
	if err := tree.Step(&plumbing.NewCommit(g.repo.Storage(), commitOid)); err != nil {
		return nil, err
	}
	commitSummaries := []CommitSummary{}
	for limit > 0 {
		commitOid, err := tree.Parent(0)
		if err != nil {
			break
		}
		commit, err := g.repo.FindCommit(commitOid)
		if err != nil {
			break
		}
		if g.CommitTouchesPath(commit, relPath) {
			commitSummaries = append(commitSummaries, g.SummarizeCommit(commit))
			limit--
		}
	}
	return commitSummaries, nil
}

func (g *GitRepo) CommitTouchesPath(commit *git.Commit, relPath string) bool {
	tree := commit.Tree()
	parent := commit.Parent(0)
	parentTree := parent.Tree()
	diff := g.repo.DiffTreeToTree(parentTree, tree, &git.DiffOptions{
		IncludeUntracked: true,
	})
	diff.ForEach(
		g.commitDiffHandler(g.rootPath, relPath, diff),
		&storer.TreeWalkConfig{Tree: parentTree},
		&storer.TreeWalkConfig{Tree: tree},
		&storer.TreeWalkConfig{Tree: tree},
	)
	return diff.HasTouched
}

func (g *GitRepo) SummarizeCommit(commit *git.Commit) CommitSummary {
	id := shortHash(commit.ID())
	summary := commit.Summary()
	author := commit.Author().Name()
	timestamp := commit.Time().Unix()

	return CommitSummary{
		id:        id,
		summary:   summary,
		author:    author,
		timestamp: timestamp,
	}
}

func (g *GitRepo) run(cmd string, args ...string) ([]byte, error) {
	parts := append([]string{cmd}, args...)
	output, err := exec.Command(parts[0], parts[1:]...).CombinedOutput()
	if err != nil {
		return nil, err
	}
	return output, nil
}

func shortHash(hash *plumbing.Hash) string {
	str := hash.String()
	return str[:8]
}

func (g *GitRepo) commitDiffHandler(rootPath string, relPath string, diff *git.Diff) func(delta *git.DiffDelta, out bool) bool {
	return func(delta *git.DiffDelta, out bool) bool {
		if newFile := delta.NewFile(); newFile != nil {
			if newFile.Path().String() != filepath.Join(rootPath, relPath) {
				return true
			}
		}
		if oldFile := delta.OldFile(); oldFile != nil {
			if oldFile.Path().String() != filepath.Join(rootPath, relPath) {
				return true
			}
		}
		return false
	}
}

func main() {
	g := &GitRepo{}
}