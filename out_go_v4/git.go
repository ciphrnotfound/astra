package gitrepo

import (
	"bytes"
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
)

type GitRepo struct {
	root  string
	repo  *git.Repository
}

type CommitSummary struct {
	id     string
.summary string
	author  string
	time    int64
}

type CommitInfo struct {
	id     string
.summary string
	author  string
	date    string
}

func (r *GitRepo) Discover(root string) (*GitRepo, error) {
	repo, err := git.PlainDiscover(root)
	if err != nil {
		return nil, err
	}
	workdir, err := repo.Worktree()
	if err != nil {
		return nil, err
	}
	return &GitRepo{
		root:  workdir.Root().Path(),
		repo:  repo,
	}, nil
}

func (r *GitRepo) RecentCommitCount(limit int) int {
	revwalk, err := r.repo.RevWalk(0)
	if err != nil {
		return 0
	}
	revwalk.PushHead()
	for revwalk.Next() {
		if revwalk.Count() >= limit {
			break
		}
	}
	return revwalk.Count()
}

func (r *GitRepo) TotalCommitCount() int {
	revwalk, err := r.repo.RevWalk(0)
	if err != nil {
		return 0
	}
	revwalk.PushHead()
	return revwalk.Count()
}

func (r *GitRepo) UncommittedFileCount() int {
	statuses, err := r.repo statuses()
	if err != nil {
		return 0
	}
	return len(statuses)
}

func (r *GitRepo) ChangedFiles() []string {
	statuses, err := r.repo.Statuses()
	if err != nil {
		return nil
	}
	var files []string
	for _, entry := range statuses {
		if entry.Path() != nil {
			files = append(files, *entry.Path())
		}
	}
	sort.Strings(files)
	sort.Strings(files)
	return files
}

func (r *GitRepo) RootPath() string {
	return r.root
}

func (r *GitRepo) GetHeadCommit() (string, error) {
	head, err := r.repo.Head()
	if err != nil {
		return "", err
	}
	output, err := exec.Command("git", "rev-parse", "HEAD").Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(output)), nil
}

func (r *GitRepo) GetDiffStats(fromCommit string) (int, int, error) {
	output, err := exec.Command("git", "diff", "--numstat", fromCommit).Output()
	if err != nil {
		return 0, 0, err
	}
	var added, deleted int
	for _, line := range strings.Split(strings.TrimSpace(string(output)), "\n") {
		parts := strings.Fields(line)
		if len(parts) >= 2 {
			if n, err := strconv.Atoi(parts[0]); err == nil {
				added += n
			}
			if n, err := strconv.Atoi(parts[1]); err == nil {
				deleted += n
			}
		}
	}
	return added, deleted, nil
}

func (r *GitRepo) GetCommitsByAuthor(author string, limit int) (string, error) {
	output, err := exec.Command("git", "log", "--author", author, "-p", "-n", fmt.Sprintf("%d", limit)).Output()
	if err != nil {
		return "", err
	}
	return string(output), nil
}

func (r *GitRepo) LastCommitInfo() (*CommitInfo, error) {
	output, err := exec.Command("git", "log", "-1", "--format=%H|%an|%ad|%s", "--date=iso-strict").Output()
	if err != nil {
		return nil, err
	}
	line := strings.TrimSpace(string(output))
	parts := strings.Fields(line)
	if len(parts) < 4 {
		return nil, err
	}
	return &CommitInfo{
		id:     parts[0],
	.summary: parts[3],
		author:  parts[1],
		date:    parts[2],
	}, nil
}

func (r *GitRepo) RecentCommits(limit int) ([]*CommitSummary, error) {
	revwalk, err := r.repo.RevWalk(0)
	if err != nil {
		return nil, err
	}
	revwalk.PushHead()
	var commits []*CommitSummary
	for oidResult := revwalk; oidResult.Next(); {
		oid := oidResult.Id()
		commit, err := r.repo.CommitObject(oid)
		if err != nil {
			continue
		}
		commits = append(commits, r.summarizeCommit(commit))
		if len(commits) >= limit {
			break
		}
	}
	return commits, nil
}

func (r *GitRepo) RecentCommitsForPath(path string, limit int) ([]*CommitSummary, error) {
	revwalk, err := r.repo.RevWalk(0)
	if err != nil {
		return nil, err
	}
	revwalk.PushHead()
	var commits []*CommitSummary
	for oidResult := revwalk; oidResult.Next(); {
		oid := oidResult.Id()
		commit, err := r.repo.CommitObject(oid)
		if err != nil {
			continue
		}
		if r.commitTouchesPath(commit, &git.PathObject{Path: path}, r.repo) {
			commits = append(commits, r.summarizeCommit(commit))
			if len(commits) >= limit {
				break
			}
		}
	}
	return commits, nil
}

func (r *GitRepo) commitTouchesPath(commit *object.Commit, path *git.PathObject, repo *git.Repository) bool {
	tree, err := commit.Tree()
	if err != nil {
		return false
	}
	parent, err := commit.Parents()
	if err != nil {
		return false
	}
	if len(parent) == 0 {
		return false
	}
	parentTree, err := parent[0].Tree()
	if err != nil {
		return false
	}
	diff := repo.DiffTreeToTree(parentTree, tree)
	diff_FOREACH := func(delta *git.DiffEntry, _ int) bool {
		if newFile := delta.NewFile(); newFile.Path != nil {
			if strings.Contains(newFile.Path.String(), path.Path) {
				return false
			}
		}
		if oldFile := delta.OldFile(); oldFile.Path != nil {
			if strings.Contains(oldFile.Path.String(), path.Path) {
				return false
			}
		}
		return true
	}
	// diff.ForEach(diff_FOREACH)
	return true
}

func (r *GitRepo) summarizeCommit(commit *object.Commit) *CommitSummary {
	id := shortOID(commit.Id())
	summary := commit.Message()
	if summary == "" {
		summary = "<no summary>"
	}
	author := commit.Author().Name()
	if author == "" {
		author = "<unknown>"
	}
	time := commit.Author().When().Unix()
	return &CommitSummary{
		id:     id,
	.summary: summary,
		author:  author,
		time:    time,
	}
}

func shortOID(oid *plumbing.Hash) string {
	s := oid.String()
	return s[:8]
}
```

This code maps the given Rust code to Go, following the provided rules.