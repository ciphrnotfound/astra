package main

import (
	"encoding/json"
	"fmt"
	"sort"

	"github.com/go-git/go-git/v5"
)

type LibraryMapping struct {
	SourceLib    string `json:"source_lib"`
	TargetLang   string `json:"target_lang"`
	TargetLib    string `json:"target_lib"`
	ImportPath   string `json:"import_path"`
	Notes        string `json:"notes"`
	GoGitMapping struct {
		PreferredHash       git.TreeHash `json:"preferred_hash"`
		RevWalkMethodName  string        `json:"rev_walk_method_name"`
	} `json:"go_git_mapping"`
}

type LibraryRegistry struct {
	mappings map[string]map[string]LibraryMapping
}

func (r *LibraryRegistry) new() *LibraryRegistry {
	registry := &LibraryRegistry{mappings: make(map[string]map[string]LibraryMapping)}
	registry.initDefaults()
	return registry
}

func (r *LibraryRegistry) initDefaults() {
	r.add("git2", "go", LibraryMapping{
		SourceLib:    "git2",
		TargetLang:   "go",
		TargetLib:    "go-git",
		ImportPath:   "github.com/go-git/go-git/v5",
		Notes:        "Use go-git v5. Prefer plumbing.Hash over OID. Avoid inventing RevWalk() methods.",
		GoGitMapping: struct {
			PreferredHash       git.TreeHash `json:"preferred_hash"`
			RevWalkMethodName  string        `json:"rev_walk_method_name"`
		}{PreferredHash: git.TreeHash{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
			RevWalkMethodName:  "NewRevWalk"},
	})

	r.add("git2", "python", LibraryMapping{
		SourceLib:  "git2",
		TargetLang: "python",
		TargetLib:  "subprocess",
		ImportPath: "import subprocess",
		Notes:      "Use the Git CLI via subprocess. Prefer porcelain output formatting.",
	})
	r.add("serde", "python", LibraryMapping{
		SourceLib:  "serde",
		TargetLang: "python",
		TargetLib:  "dataclasses",
		ImportPath: "from dataclasses import dataclass",
		Notes:      "Use @dataclass for structs.",
	})
}

func (r *LibraryRegistry) add(lang, sourceLib string, mapping LibraryMapping) {
	targetLangs := r.mappings[lang]
	if targetLangs == nil {
		targetLangs = make(map[string]LibraryMapping)
		r.mappings[lang] = targetLangs
	}
	targetLangs[sourceLib] = mapping
	sort.Strings(targetLangs.keys())
}

func (r *LibraryRegistry) get(sourceLib string, targetLang string) *LibraryMapping {
	targetLangs := r.mappings[targetLang]
	if targetLangs == nil {
		return nil
	}
	mappings, ok := targetLangs[sourceLib]
	if !ok {
		return nil
	}
	return &mappings
}
```
```go
// main.go
package main

import "fmt"

func main() {
	registry := new(LibraryRegistry)
	fmt.Println(registry.new().get("git2", "go").ImportPath)
}