package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/pkg/errors"
)

type TeamState struct {
	TeamName    string    `json:"team_name"`
	AdminKey    string    `json:"admin_key"`
_APIKey      string    `json:"api_key"`
	Members     map[string]TeamMember `json:"members"`
	Tasks       map[string]Task      `json:"tasks"`
	Sessions    []Session    `json:"sessions"`
}

type TeamMember struct {
	Name  string `json:"name"`
	Role  TeamRole `json:"role"`
	Key   string `json:"key"`
}

type TeamRole string

const (
	Admin TeamRole = "Admin"
	Member TeamRole = "Member"
)

type Task struct {
	ID       string    `json:"id"`
	Description string `json:"description"`
	Assignee string    `json:"assignee"`
	Status   TaskStatus `json:"status"`
}

type TaskStatus string

const (
	Pending TaskStatus = "Pending"
	InProgress TaskStatus = "InProgress"
	Done TaskStatus = "Done"
)

type Session struct {
	TaskID     string   `json:"task_id"`
	Developer  string   `json:"developer"`
	StartTime  time.Time `json:"start_time"`
	EndTime    *time.Time `json:"end_time"`
	StartCommit string `json:"start_commit"`
	EndCommit  *string `json:"end_commit"`
 LinesAdded int `json:"lines_added"`
 LinesDeleted int `json:"lines_deleted"`
}

type TeamManager struct {
	statePath string
	repoPath  string
}

func (m *TeamManager) new(repoPath string) *TeamManager {
	return &TeamManager{
		statePath: resolveStatePath(repoPath),
		repoPath:  repoPath,
	}
}

func resolveStatePath(repoPath string) string {
	return filepath.Join(repoPath, "teams.json")
}

func (m *TeamManager) loadState() (*TeamState, error) {
	if _, err := os.Stat(m.statePath); err != nil {
		return &TeamState{}, nil
	}
	data, err := ioutil.ReadFile(m.statePath)
	if err != nil {
		return nil, err
	}
	var state TeamState
	err = json.Unmarshal(data, &state)
	return &state, err
}

func (m *TeamManager) saveState(state *TeamState) error {
	parentDir := filepath.Dir(m.statePath)
	err := os.MkdirAll(parentDir, 0755)
	if err != nil {
		return err
	}
	data, err := json.MarshalIndent(state, "", "  ")
	if err != nil {
		return err
	}
	return ioutil.WriteFile(m.statePath, data, 0644)
}

func (m *TeamManager) sync() error {
	// 1. Fetch `astra-state` from origin (ignore er if it doesn't exist yet)
	_, err := exec.Command("git", "fetch", "origin", "astra-state").Output()
	if err != nil {
		return err
	}

	// 2. Read `teams.json` from `origin/astra-state`
	output, err := exec.Command("git", "show", "origin/astra-state:teams.json").Output()
	if err != nil {
		return err
	}

	if string(output) != "" {
		var remoteState TeamState
		err = json.Unmarshal(output, &remoteState)
		if err != nil {
			return err
		}

		localState, err := m.loadState()
		if err != nil {
			return err
		}

		// Simple merge: keep remote state but retain local active sessions 
		// that haven't synchronized to remote yet.
		for _, localSession := range localState.Sessions {
			if !contains(remoteState.Sessions, func(session Session) bool { return session.TaskID == localSession.TaskID && session.Developer == localSession.Developer }) {
				remoteState.Sessions = append(remoteState.Sessions, localSession)
			}
		}

		// Write the merged state locally bypassing saveState to avoid an infinite push loop
		data, err := json.MarshalIndent(&remoteState, "", "  ")
		if err != nil {
			return err
		}
		return ioutil.WriteFile(m.statePath, data, 0644)
	}

	return os.ErrNotExist
}

func (m *TeamManager) pushState() error {
	output, err := exec.Command("git", "hash-object", "-w", "--stdin").Output()
	if err != nil {
		return err
	}

	data, err := ioutil.ReadFile(m.statePath)
	if err != nil {
		return err
	}

	child, err := exec.Command("git", "mktree").Input(string(data)).Output()
	if err != nil {
		return err
	}

	parentArgs := []string{"rev-parse", "refs/heads/astra-state"}
	parentArgsOut, err := exec.Command("git", parentArgs...).Output()
	if err != nil {
		return err
	}
	parentArgsOut = bytes.Trim(parentArgsOut, "\n")

	parentSHA := string(parentArgsOut)
	parentSHA = strings.TrimSpace(parentSHA)
	parentSHA = strings.TrimPrefix(parentSHA, "refs/heads/astra-state: ")

	if parentSHA != "" {
		parentArgs = append(parentArgs, "-p")
		parentArgs = append(parentArgs, parentSHA)
	}

	commitArgs := []string{"commit-tree"}
	commitArgs = append(commitArgs, string(child))
	commitArgs = append(commitArgs, parentArgs...)
	commitArgs = append(commitArgs, "-m", "Update Astra team state")

	commitOut, err := exec.Command("git", commitArgs...).Output()
	if err != nil {
		return err
	}

	treeSHA := string(commitOut)
	treeSHA = strings.TrimSpace(treeSHA)
	treeSHA = strings.TrimPrefix(treeSHA, "commit-tree ")

	updateRef := []string{"update-ref", "refs/heads/astra-state", treeSHA}
	updateRefOut, err := exec.Command("git", updateRef...).Output()
	if err != nil {
		return err
	}

	pushCommand := []string{"push", "origin", "astra-state"}
	pushOut, err := exec.Command("git", pushCommand...).Output()
	if err != nil {
		return err
	}

	return nil
}

func (m *TeamManager) initTeam(name string, adminKey string) error {
	state := TeamState{
		AdminKey:    adminKey,
		_APIKey:      adminKey,
		Members:     map[string]TeamMember{
			"admin": {Name: "admin", Role: Admin, Key: adminKey},
		},
		TeamName:    name,
		Tasks:       map[string]Task{},
		Sessions:    []Session{},
	}

	return m.saveState(&state)
}

func (m *TeamManager) addMember(adminKey string, name string, role TeamRole, memberKey string) error {
	state, err := m.loadState()
	if err != nil {
		return err
	}

	m.requireAdmin(state, adminKey)

	if _, ok := state.Members[name]; ok {
		return errors.New("member " + name + " already exists")
	}

	state.Members[name] = TeamMember{
		Name:  name,
		Role:  role,
		Key:   memberKey,
	}

	return m.saveState(state)
}

func (m *TeamManager) requireAdmin(state *TeamState, adminKey string) error {
	if state.AdminKey != adminKey {
		return errors.New("not authenticated as admin")
	}

	return nil
}

func contains[T any](slice []T, predicate func(element T) bool) bool {
	for _, x := range slice {
		if predicate(x) {
			return true
		}
	}

	return false
}
func loadState(repoPath string) (*State, error) {
    return loadStateInternal(repoPath)
}

func saveState(repoPath string, state *State) error {
    return saveStateInternal(repoPath, state)
}

func requireAdmin(state *State, adminKey string) error {
    if state.teamName == "" {
        return errors.New("Team not initialized. Run 'astra team init' first.")
    }
    // rest of the implementation remains the same
}

func requireMember(state *State, memberKey string) error {
    // rest of the implementation remains the same
}

func requireMemberKey(state *State, developer string, memberKey string) error {
    // rest of the implementation remains the same
}

type TaskStatus int

const (
    Pending TaskStatus = iota
    InProgress
    Done
)

type Task struct {
    id       string
    description string
    assignee string
    status TaskStatus
}

type Session struct {
    taskID string
    developer string
    startTime int64
    endTime *int64
    startCommit string
    endCommit string
    linesAdded int
    linesDeleted int
}

type State struct {
    teamName string
    tasks   map[string]Task
    members map[string]TeamMember
    sessions []Session
}

type TeamMember struct {
    name string
    role string
    key string
}

func loadStateInternal(repoPath string) (*State, error) {
    // implementation remains the same
}

func saveStateInternal(repoPath string, state *State) error {
    // implementation remains the same
}

func generateReport(repoPath, adminKey string) (string, error) {
    state, err := loadState(repoPath)
    if err != nil {
        return "", err
    }
    requireAdmin(state, adminKey)
    // rest of the implementation remains the same
}

func assignTask(repoPath string, taskId string, description string, assignee string, adminKey string) error {
    state, err := loadState(repoPath)
    if err != nil {
        return err
    }
    // existing implementation
}

func startTask(repoPath string, memberKey string, taskId string, developer string) error {
    state, err := loadState(repoPath)
    if err != nil {
        return err
    }
    // existing implementation
}

func finishTask(repoPath string, memberKey string, taskId string, developer string) (Session, error) {
    state, err := loadState(repoPath)
    if err != nil {
        return Session{}, err
    }
    // existing implementation
}

func assignTaskImplicit(repoPath string, taskId string, description string, assignee string) error {
    state, err := loadState(repoPath)
    if err != nil {
        return err
    }
    // existing implementation
}

func startTaskImplicit(repoPath string, taskId string, developer string) error {
    state, err := loadState(repoPath)
    if err != nil {
        return err
    }
    // existing implementation
}

func generateKey(s string) string {
    // implementation remains the same
}
func resolveStatePath(repoPath string) (filepath string, err error) {
    preferred := filepath.Join(repoPath, ".astra", "teams.json")
    if _, err := os.Stat(preferred); err == nil {
        return preferred, nil
    }
    previous := filepath.Join(repoPath, ".forge", "teams.json")
    if _, err := os.Stat(previous); err == nil {
        return previous, nil
    }
    legacy := filepath.Join(repoPath, ".codex", "teams.json")
    if _, err := os.Stat(legacy); err == nil {
        return legacy, nil
    }
    return preferred, nil
}

func normalizeState(state TeamState) (TeamState, error) {
    if state.AdminKey == "" && state.APIKey != "" {
        state.AdminKey = state.APIKey
    }
    if state.Members == make(map[string]TeamMember) && state.AdminKey != "" {
        state.Members["admin"] = TeamMember{
            Name:     "admin",
            Role:     TeamAdmin,
            Key:      state.AdminKey,
        }
    }
    return state, nil
}

func generateKey(prefix string) (string, error) {
    now := time.Now().UnixNano() / 1e3
    pid := os.Getpid()
    return fmt.Sprintf("astra_%s_%d_%d", prefix, pid, now), nil
}
