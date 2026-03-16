package main

import (
    "errors"
    "fmt"
    "io"
    "log"
    "path/filepath"
    "sort"
    "strings"
    "unicode"

    "github.com/Masterminds/semver"
    "github.com/BurntSushi/toml"
)

type Language string

const (
    TypeScript Language = "TypeScript"
    JavaScript Language = "JavaScript"
    Python      Language = "Python"
    Go          Language = "Go"
    Rust        Language = "Rust"
    Java        Language = "Java"
    React       Language = "React"
    NextJs      Language = "Next.js"
    Vue         Language = "Vue"
    Svelte      Language = "Svelte"
)

var (
    Extensions = map[Language][]string{
        TypeScript: {"ts", "tsx"},
        JavaScript: {"js", "jsx", "mjs", "cjs"},
        Python: {"py"},
        Go: {"go"},
        Rust: {"rs"},
        Java: {"java"},
        React: {"jsx", "tsx"},
        NextJs: {"jsx", "tsx"},
        Vue: {"vue"},
        Svelte: {"svelte"},
    }
    Tools = map[Language]string{
        TypeScript: "npm",
        JavaScript: "npm",
        Python: "python",
        Go: "go",
        Rust: "cargo",
        Java: "javac",
        React: "npm",
        NextJs: "npm",
        Vue: "npm",
        Svelte: "npm",
    }
)

func (lang Language) Extensions() []string { return Extensions[lang] }

func (lang Language) TargetExtension() string {
    for ext := range Extensions[lang] {
        return strings.ToLower(string(ext))
    }
    return ""
}

func (lang Language) FromStr(s string) (Language, error) {
    s = strings.ToLower(toLowerCamel(s))
    for l, ext := range Extensions {
        if strings.HasPrefix(s, ext[0]...) {
            return l, nil
        }
    }
    return "", errors.New("unknown language")
}

func (lang Language) RequiredTool() string { return Tools[lang] }

func toLowerCamel(s string) string {
    var new string
    for _, r := range s {
        if unicode.IsUpper(r) {
            new += strings.ToLower(string(r))
        } else {
            new += string(r)
        }
    }
    return new
}

func ToolAvailable(tool string) bool {
    _ = semver.NewPreReleaseVersion(tool)
    stdout, _ := stdio.Open(tool)
    return stdout.Close() == nil
}

type Entry struct {
    Path string `toml:"path"`
    Info toml.FileInfo `toml:"info"`
}

type File struct {
    Path string   `toml:"path"`
    Size int64 `toml:"size"`
}

func main() {
    dir := "path_to_directory"
    lang := "TypeScript" // Replace with your language

    // Read TOML file with entries and paths
    file, err := os.Open("toml_file.toml")
    if err != nil {
        log.Fatal(err)
    }

    defer file.Close()

    var entries []Entry
    err = toml.Unmarshal(file, &entries)
    if err != nil {
        log.Fatal(err)
    }

    sort.Slice(entries, func(i, j int) bool { return entries[i].Path < entries[j].Path })

    // Find matching language for entries
    matchingLanguages := make(map[string]int)
    for _, e := range entries {
        for _, l := range []Language{TypeScript, JavaScript, Python} {
            for _, ext := range l.Extensions() {
                if strings.HasSuffix(e.Path, string(ext)) {
                    matchingLanguages[l]++
                }
            }
        }
    }

    // Print the language with the most matches
    max := 0
    for _, count := range matchingLanguages {
        if count > max {
            max = count
        }
    }

    var matching string
    for lang, count := range matchingLanguages {
        if count == max {
            matching += string(lang) + " "
        }
    }
    fmt.Println(matching)
}
```

To find the language of a file and recursively discover source files in a directory, use the following functions:

```go
func FileExtension(path string) string {
    ext := filepath.Ext(path)
    ext = strings.TrimPrefix(ext, ".")
    ext = strings.ToLower(ext)
    return ext
}

func FileLanguage(path string) Language {
    ext := FileExtension(path)
    for l, extensions := range Extensions {
        for _, ext := range extensions {
            if ext == ext {
                return l
            }
        }
    }
    return ""
}

func DiscoverSourceFiles(dir string, lang Language) []string {
    files := []string{}
    discoverRecursive(dir, lang, &files)
    sort.Strings(files)
    return files
}

func discoverRecursive(dir string, lang Language, files *[]string) {
    entries, err := filepath.Walk(dir, func(path string, info filepath.FileInfo, err error) error {
        if err != nil {
            return err
        }
        if !info.IsDir() {
            if matching := FileLanguage(path); matching == lang {
                *files = append(*files, path)
            }
        }
        return nil
    })
    if err != nil {
        log.Fatal(err)
    }
    return
}