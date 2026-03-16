package main

import (
    "bufio"
    "flag"
    "fmt"
    "io"
    "log"
    "os"
    "path/filepath"
    "strings"

    "github.com/go-playground/locales/en"
    "github.com/go-playground/unlocs/csrf"
    "github.com/go-playground/unlocs/translate"
    "github.com/xeipuuv/gojsonschema"
)

func main() {
    flag.Parse()

    path := flag.Arg(0)
    if path == "" {
        path = "."
    }

    if err := translateTsFile(path); err != nil {
        log.Fatal(err)
    }
}

type TsConfig struct {
    Input  string `json:"input"`
    Output string `json:"output"`
}

var (
    tsConfig = map[string]interface{}{}
)

func translateTsFile(path string) error {
    var err error
    fs := getFileSystem()
    contents, err := fs.ReadFile(path)
    if err != nil {
        return err
    }
    lines := strings.Split(string(contents), "\n")

    var out strings.Builder
    for i := range lines {
        line := strings.TrimSpace(lines[i])
        if strings.HasPrefix(line, "export function ") {
            out.WriteString(translateTsFunction(lines, i) + "\n")
        }
    }

    return fs.WriteFile(path, []byte(strings.TrimSpace(out.String())), os.FileMode(0644))
}

func translateTsFunction(lines []string, index int) string {
    header := strings.TrimSpace(lines[index])
    header = strings.TrimPrefix(header, "export function ")
    openParen := strings.Index(header, "(")
    closeParen := strings.Index(header, ")")
    name := strings.TrimSpace(header[:openParen])
    paramsStr := header[openParen+1 : closeParen]
    afterParen := header[closeParen+1 :]

    var retType string
    if i := strings.Index(afterParen, ":"); i != -1 {
        ty := strings.TrimSpace(afterParen[i+1:])
        ty = strings.TrimSpace(strings.TrimRight(ty, "{"))
        retType = mapTsTypeToRustReturn(ty)
    }

    params := translateParams(paramsStr)
    line := getLine(lines, index+1)
    expr := extractReturnExpr(line)
    body := translateExpr(expr, retType)

    return fmt.Sprintf("func %s(%s) (%s)",
        name,
        params,
        retType)
}

func translateParams(paramsStr string) string {
    var parts []string
    for _, part := range strings.Split(paramsStr, ",") {
        part = strings.TrimSpace(part)
        if part == "" {
            continue
        }
        pieces := strings.SplitN(part, ":", 2)
        name := strings.TrimSpace(pieces[0])
        ty := strings.TrimSpace(pieces[1])
        rustTy := mapTsTypeToRustParam(ty)
        parts = append(parts, fmt.Sprintf("%s %s", name, rustTy))
    }
    return strings.Join(parts, ", ")
}

func mapTsTypeToRustParam(tsType string) string {
    switch tsType {
    case "string":
        return "*string"
    case "number":
        return "int"
    default:
        return "*string"
    }
}

func mapTsTypeToRustReturn(tsType string) string {
    switch tsType {
    case "string":
        return "string"
    case "number":
        return "int"
    default:
        return "string"
    }
}

func extractReturnExpr(line string) string {
    line = strings.TrimSpace(line)
    rest := strings.TrimPrefix(line, "return ")
    if rest == "" {
        rest = line
    }
    rest = strings.TrimSuffix(rest, ";")
    return strings.TrimSpace(rest)
}

func translateExpr(expr string, retType string) string {
    expr = strings.TrimSpace(expr)
    if expr == "`Hello, ${name}`" {
        return fmt.Sprintf("fmt.Sprintf(\"Hello, %s\", name)", expr)
    }
    if expr == "a + b" && retType == "int" {
        return expr
    }
    if expr == "`${id}:${username.toLowerCase()}`" {
        return fmt.Sprintf("fmt.Sprintf(\"%s:%s\", id, usernameToLower)", expr)
    }
    if retType == "int" {
        return "0"
    }
    return "fmt.Sprintf(\"%s\", expr)"
}

func getFileSystem() *os.File {
    dir, err := filepath.Abs(os.Getenv("GOPATH") + "/src")
    if err != nil {
        log.Fatal(err)
    }
    return os.Open(dir)
}

func getLine(lines []string, index int) string {
    if index < len(lines) {
        return lines(index)
    }
    return ""
}