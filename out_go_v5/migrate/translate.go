package codex

import (
    "fmt"
    "github.com/go-git/go-git/v5"
    "log"
)

// HybridTranslator represents a hybrid translator that uses both AI and rule-based translation.
type HybridTranslator struct {
    rules Translator
    ai    AiTranslator
}

// NewHybridTranslator returns a new instance of HybridTranslator.
func NewHybridTranslator(model *CodexModel) *HybridTranslator {
    return &HybridTranslator{
        rules: NewRuleBasedTranslator(),
        ai:    NewAiTranslator(model),
    }
}

// Translate translates the source code from one language to another.
func (t *HybridTranslator) Translate(sourceCode string, from, to Language) (string, error) {
    if t.ai != nil {
        result, err := t.ai.Translate(sourceCode, from, to)
        if err != nil {
            return "", err
        }
        if result != "" {
            return result, nil
        }
    }

    return t.rules.Translate(sourceCode, from, to)
}

// HybridTranslator is the main translator used by the codex system.
type Translator interface {
    Translate(string, Language, Language) (string, error)
}

// AiTranslator represents an AI-powered translator.
type AiTranslator struct {
    model *CodexModel
}

// NewAiTranslator returns a new instance of AiTranslator.
func NewAiTranslator(model *CodexModel) *AiTranslator {
    return &AiTranslator{model: model}
}

// Translate translates the source code from one language to another.
func (t *AiTranslator) Translate(sourceCode string, from, to Language) (string, error) {
    if len(sourceCode) > 15000 {
        return t.translateInChunks(sourceCode, from, to)
    }

    mappingContext := t.buildMappingContext(sourceCode, to)

    prompt := fmt.Sprintf("You are an expert code translator. Translate the following %s code into idiomatic %s.\n\
             Rules:\n\
             - Preserve function names and signatures as closely as possible\n\
             - Use idiomatic patterns for the target language\n\
             - Do not add comments or explanations\n\
             - Output ONLY the translated code, no markdown fences, no extra text\n\
             - Translate ALL code in the file, not just functions\n\
             - Do not omit logic, types, or control flow\n\
             - Avoid TODO stubs or placeholder code\n\
             - Map types idiomatically (e.g. string → &str/String, number → i32/f64, etc.)\n\
             %s\n\
             Source %s code:\n%s",
        from,
        to,
        mappingContext,
        from,
        sourceCode,
    )

    raw, err := t.model.complete(prompt)
    if err != nil {
        return "", err
    }
    return stripMarkdownFences(raw), nil
}

// translateInChunks translates the source code in chunks.
func (t *AiTranslator) translateInChunks(sourceCode string, from, to Language) (string, error) {
    result := ""
    chunkSize := 10000
    overlap := 1000

    mappingContext := t.buildMappingContext(sourceCode, to)

    for start := 0; start < len(sourceCode); {
        end := min(start+chunkSize, len(sourceCode))
        chunk := sourceCode[start:end]

        prompt := fmt.Sprintf("You are an expert code translator. Translate the following %s code into idiomatic %s.\n\
             Rules:\n\
             - Preserve function names and signatures as closely as possible\n\
             - Use idiomatic patterns for the target language\n\
             - Do not add comments or explanations\n\
             - Output ONLY the translated code, no markdown fences, no extra text\n\
             - Translate ALL code in the file, not just functions\n\
             - Do not omit logic, types, or control flow\n\
             - Avoid TODO stubs or placeholder code\n\
             - Map types idiomatically (e.g. string → &str/String, number → i32/f64, etc.)\n\
             %s\n\
             Source %s code:\n%s",
            from,
            to,
            mappingContext,
            from,
            chunk,
        )

        translatedChunk, err := t.model.complete(prompt)
        if err != nil {
            return "", err
        }
        translatedChunk = stripMarkdownFences(translatedChunk)
        result += translatedChunk
        result += "\n"

        if end == len(sourceCode) {
            break
        }

        start += chunkSize - overlap
    }
    return result, nil
}

// buildMappingContext builds a mapping context for the given source code and target language.
func (t *AiTranslator) buildMappingContext(sourceCode string, to Language) string {
    context := ""

    if sourceCode.Contains("git2") {
        if m := t.model.getMapping("git2", to); m != nil {
            context += fmt.Sprintf("- Map Rust 'git2' to %s: %s. Import: '%s'. NOTE: %s",
                m.targetLib,
                to,
                m.importPath,
                m.notes,
            )
        }
    }

    if sourceCode.Contains("serde") {
        if m := t.model.getMapping("serde", to); m != nil {
            context += fmt.Sprintf("- Map Rust 'serde' to %s: %s. Import: '%s'. NOTE: %s",
                m.targetLib,
                to,
                m.importPath,
                m.notes,
            )
        }
    }

    return context
}

func min(a, b int) int {
    if a < b {
        return a
    }
    return b
}

func stripMarkdownFences(code string) string {
    // Remove markdown fences from the code
    code = strings.Replace(code, "```python", "", -1)
    code = strings.Replace(code, "```rust", "", -1)
    return code
}
package main

import (
	"fmt"
	"log"
	"os"
	"strings"

	"github.com/go-git/go-git/v5"
)

func getLinesFromText(fileContent string) []string {
	return strings.Split(fileContent, "\n")
}

func getLinesFromFile(filePath string) ([]byte, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	var contents []byte
	_, err = file.Read(contents)
	if err != nil {
		return nil, err
	}

	return contents, nil
}

func removeComments(lines []string) []string {
	var cleanLines []string

	for _, line := range lines {
		if strings.TrimSpace(line) == "" {
			continue
		}
		commentPos := strings.Index(line, "//")
		if commentPos != -1 {
			cleanLines = append(cleanLines, strings.TrimSpace(line[:commentPos]))
		} else {
			cleanLines = append(cleanLines, line)
		}
	}

	return cleanLines
}

func getGitHash(repo *git.Repository) ([]byte, error) {
	plumbingHash, err := repo.Hash()
	if err != nil {
		return nil, err
	}

	return plumbingHash, nil
}

func getGitCommitHash(repo *git.Repository) ([]byte, error) {
	plumbingHash, err := repo commitments().Hash()
	if err != nil {
		return nil, err
	}

	return plumbingHash, nil
}

func processLines(lines []string, isTs bool) string {
	var out string

	for _, line := range lines {
		line = strings.TrimSpace(line)

		if line == "" {
			out += "\n"
			continue
		}

		if isTs {
			if strings.HasPrefix(line, "export interface ") || strings.HasPrefix(line, "interface ") {
				var (
					rustCode string
					consumed int
				)

				if strings.HasPrefix(line, "export interface ") {
					line = line[14:]
				} else {
					line = line[9:]
				}

				// TODO: Implement logic to translate interface to Rust
				out += rustCode
				continue
			}

			if strings.HasPrefix(line, "export type ") || strings.HasPrefix(line, "type ") {
				var (
					name, rhs string
				)

				parts := strings.Split(line, " ")

				isTypeDecl := false
				for _, p := range parts {
					if strings.HasPrefix(p, "type") {
						isTypeDecl = true
						break
					}
				}

				if isTypeDecl {
					name = strings.Join(parts[1:], " ")
					rhs = strings.Join(parts[2:], " ")
				} else {
					name = strings.Join(parts[2:], " ")
					rhs = strings.Join(parts[3:], " ")
				}

				rustTy := // TODO: Implement logic to map type to Rust

				out += fmt.Sprintf("pub type %s = %s;\n", name, rustTy)
			}

			// TODO: Implement logic to translate class to Rust
		} else {
			if strings.HasPrefix(line, "use ") || strings.HasPrefix(line, "mod ") || strings.HasPrefix(line, "extern crate ") {
				out += fmt.Sprintf("// %s\n", line)
				continue
			}

			if strings.HasPrefix(line, "pub struct ") || strings.HasPrefix(line, "struct ") {
				// TODO: Implement logic to translate struct to TypeScript
			}

			out += fmt.Sprintf("// %s\n", line)
		}

		out += "\n"
	}

	return out
}

func main() {
	filePath := os.Args[1]

	fileByte, err := getLinesFromFile(filePath)
	if err != nil {
		log.Fatal(err)
	}

	lines := getLinesFromText(string(fileByte))

	cleanLines := removeComments(lines)

	tSOutput := processLines(cleanLines, true)

	rustOutput := processLines(cleanLines, false)

	fmt.Printf("%s%s", tSOutput, rustOutput)
}
import (
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
)

func walkDir(dir string, f func(p string) error) error {
	return fs.WalkDir(os.DirFS(dir), ".", func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.Type().IsDir() {
			return f(filepath.Join(dir, d.Name()))
		}
		if d.IsDir() {
			return nil
		}
		if filepath.Ext(d.Name()) == ".ts" {
			log.Println("Processing:", d.Name())
			return processTSFile(p)
		}
		return nil
	})
}

type tsFile struct {
	path   string
	contents string
}

var tsDirMap = map[string]string{
	"$root":        "root",
	"$root$/file": "file",
}

func processTSFile(path string) error {
	// read file
	file, err := os.Open(path)
	if err != nil {
		log.Println("Error reading file:", err)
		return err
	}
	defer file.Close()

	// read file contents
	var contents string
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		contents += scanner.Text() + "\n"
	}
	if err := scanner.Err(); err != nil {
		log.Println("Error reading file contents:", err)
		return err
	}

	// process contents
	t := newTSFile(path, contents)
	t.process()

	// update tsDirMap
	updateTSDirMap()

	return nil
}

func (t *tsFile) process() {
	// process file
	// ...

	// update file status
	updateFileStatus()
}

func updateTSDirMap() {
	// Update tsDirMap with current directory information
}

func updateFileStatus() {
	// Update file status based on processing
}

func main() {
	dir := "./src"

	if err := walkDir(dir, nil); err != nil {
		log.Fatal(err)
	}
}

type tsFile struct {
	path   string
	contents string
}

var (
	tree       *object.Tree
	repository *git.Repository
)

func newTSFile(path string, contents string) *tsFile {
	return &tsFile{
		path:   path,
		contents: contents,
	}
}

func (t *tsFile) process() {
	// get repository
	repository, err := git.PlainOpen("/")
	if err != nil {
		log.Fatal(err)
	}

	// get tree
	tree, err = repository.Tree()
	if err != nil {
		log.Fatal(err)
	}

	// create commit
	commitHash, err := t.commit("Process file", t.contents)
	if err != nil {
		log.Fatal(err)
	}

	// create branch
	branchHash, err := repository.CreateBranch("my_branch", commitHash)
	if err != nil {
		log.Fatal(err)
	}

	// checkout branch
	if err := repository.Checkout(branchHash); err != nil {
		log.Fatal(err)
	}

	// update tree
	if err := t.updateTree(); err != nil {
		log.Fatal(err)
	}
}

func (t *tsFile) updateTree() error {
	// create tree
	tree, err := repository.Tree()
	if err != nil {
		return err
	}

	// update tree
	if err := walkDir(".", func(p string) error {
		return t.updateFile(p, tree)
	}); err != nil {
		return err
	}

	return nil
}

func (t *tsFile) updateFile(p string, tree *object.Tree) error {
	// create hash
	hash, err := plumbing.NewHash("1234567890")
	if err != nil {
		return err
	}

	// add file
	file := filepath.Base(p)
	if err := tree.AddObject(hash, file); err != nil {
		return err
	}

	return nil
}

func (t *tsFile) commit(message string, contents string) (plumbing.Hash, error) {
	// create commit
	return t.repository.Commit(message, &object.CommitOptions{
	TREE: t.tree,
	}, nil)
}
func translateLateToGo(value interface{}) string {
    if value == nil {
        return ""
    }
    name := toSnakeCase(getName(value))
    fmtExpr := getFormatExpr(value)
    if fmtExpr != nil {
        return fmt.Sprintf("func %s() string { return %s }\n", name, fmtExpr)
    } else {
        return fmt.Sprintf("func %s() string { return \"\" }\n", name)
    }
}

func translatePyToGo(source string) string {
    lines := strings.Split(source, "\n")
    out := ""
    i := 0
    for _, line := range lines {
        trimmed := strings.TrimSpace(line)
        if trimmed == "" {
            out += "\n"
            i++
            continue
        }
        if strings.HasPrefix(trimmed, "import") || strings.HasPrefix(trimmed, "from ") {
            i++
            continue
        }
        if strings.HasPrefix(trimmed, "class ") {
            header := strings.TrimPrefix(trimmed, "class ")
            name := header
            parts := strings.Split(header, "(")
            name = parts[0]
            name = name[0 : len(name)-1] // Remove trailing space
            fields := make(map[string]string)
            methods := make(map[string]interface{})
            bodyLines, consumed := collectPyBody(lines, i)
            for _, bline := range bodyLines {
                trimmed := strings.TrimSpace(bline)
                if strings.HasPrefix(trimmed, "self.") && strings.Contains(trimmed, "=") {
                    parts := strings.Split(trimmed, "=")
                    field := parts[0]
                    field = strings.Split(field, ".")[1]
                    field = strings.TrimSpace(field)
                    field = field[0 : len(field)-1] // Remove trailing colon
                    if _, ok := fields[field]; !ok {
                        fields[field] = "string"
                    }
                }
                if strings.HasPrefix(trimmed, "def ") || strings.HasPrefix(trimmed, "async def ") {
                    trimmed = strings.TrimPrefix(trimmed, "def ")
                    trimmed = strings.TrimPrefix(trimmed, "async def")
                    methods[trimmed] = trimmed
                }
            }
            out += fmt.Sprintf("type %s struct {\n", name)
            for field, _ := range fields {
                out += fmt.Sprintf("    %s string\n", toSnakeCase(field))
            }
            out += "}\n\n"
            out += fmt.Sprintf("func (s *%s) %s() {\n", name, name)
            out += "}\n\n"
            i += consumed
            continue
        }
        if strings.HasPrefix(trimmed, "def ") || strings.HasPrefix(trimmed, "async def ") {
            header := strings.Remove(trimmed, "def")
            if strings.HasPrefix(header, "async ") {
                header = strings.TrimPrefix(header, "async")
            }
            cleaned := strings.TrimPrefix(header, "def ")
            parts := strings.Split(cleaned, "(")
            name := parts[0]
            name = name[0 : len(name)-1] // Remove trailing space
            params := strings.TrimSpace(strings.Split(cleaned, ")")[0])
            after := strings.TrimSpace(strings.Split(cleaned, ")")[1])
            if strings.Contains(after, "->") {
                parts := strings.Split(after, "->")
                retType := parts[1]
                retType = strings.TrimSpace(retType)
                retType = strings.Trim(retType, "{}")
                retType = strings.TrimSpace(retType)
                retType = strings.TrimPrefix(retType, "async")
                goParams := translatePyParamsGo(params)
                out += fmt.Sprintf("func %s(%s) %s {\n", toSnakeCase(name), goParams, retType)
                bodyLines, consumed := collectPyBody(lines, i)
                wroteBody := false
                for _, bodyLine := range bodyLines {
                    statement := translatePyStatementToGo(bodyLine)
                    if statement != "" {
                        out += fmt.Sprintf("    %s\n", statement)
                        wroteBody = true
                    }
                }
                if !wroteBody {
                    out += fmt.Sprintf("    %s\n", retType)
                }
                out += "}\n"
            } else {
                goParams := translatePyParamsGo(params)
                out += fmt.Sprintf("func %s(%s) {\n", toSnakeCase(name), goParams)
                bodyLines, consumed := collectPyBody(lines, i)
                wroteBody := false
                for _, bodyLine := range bodyLines {
                    statement := translatePyStatementToGo(bodyLine)
                    if statement != "" {
                        out += fmt.Sprintf("    %s\n", statement)
                        wroteBody = true
                    }
                }
                if !wroteBody {
                    out += fmt.Sprintf("    %s\n", "string")
                }
                out += "}\n"
            }
        }
        i++
    }
    return out
}

func translatePyParamsGo(pyParams string) string {
    params := strings.Split(pyParams, ",")
    out := ""
    for _, param := range params {
        trimmed := strings.TrimSpace(param)
        if trimmed != "" && trimmed != "self" {
            parts := strings.Split(trimmed, ":")
            if len(parts) == 2 {
                name := strings.TrimSpace(parts[0])
                goType := mapPyTypeGo(parts[1])
                out += fmt.Sprintf("%s %s, ", toSnakeCase(name), goType)
            } else {
                out += fmt.Sprintf("%s string, ", trimmed)
            }
        }
    }
    return strings.TrimSuffix(out, ", ")
}

func mapPyTypeGo(pyType string) string {
    if pyType == "str" {
        return "string"
    }
    if pyType == "int" {
        return "int64"
    }
    if pyType == "float" {
        return "float64"
    }
    if pyType == "bool" {
        return "bool"
    }
    if pyType == "None" {
        return "()"
    }
    if pyType == "list" || pyType == "List" {
        return "[]string"
    }
    if pyType == "dict" || pyType == "Dict" {
        return "map[string]string"
    }
    if pyType == "bytes" {
        return "[]byte"
    }
    return "string"
}

func collectPyBody(lines []string, i int) ([]string, int) {
    bodyLines := make([]string, 0)
    consumed := 0
    for {
        line := lines[i+consumed]
        trimmed := strings.TrimSpace(line)
        if trimmed == "" || strings.HasPrefix(trimmed, "self.") || strings.HasPrefix(trimmed, "def ") {
            break
        }
        bodyLines = append(bodyLines, trimmed)
        consumed++
    }
    return bodyLines, consumed
}

func translateTsToGo(source string) string {
    lines := strings.Split(source, "\n")
    out := ""
    i := 0
    for _, line := range lines {
        trimmed := strings.TrimSpace(line)
        if trimmed == "" {
            out += "\n"
            i++
            continue
        }
        if strings.HasPrefix(trimmed, "import") {
            i++
            continue
        }
        if strings.HasPrefix(trimmed, "export function") || strings.HasPrefix(trimmed, "function") {
            clean := strings.Remove(trimmed, "export")
            if strings.HasPrefix(clean, "async ") {
                clean = strings.TrimPrefix(clean, "async")
            }
            clean = strings.Remove(clean, "function")
            parts := strings.Split(clean, "(")
            name := parts[0]
            name = name[0 : len(name)-1] // Remove trailing space
            params := strings.TrimSpace(strings.Split(clean, ")")[0])
            after := strings.TrimSpace(strings.Split(clean, ")")[1])
            goTypes := make(map[string]string)
            if strings.Contains(after, "->") {
                parts := strings.Split(after, "->")
                retType := parts[1]
                retType = strings.TrimSpace(retType)
                retType = strings.Trim(retType, "{}")
                retType = strings.TrimSpace(retType)
                retType = strings.TrimPrefix(retType, "async")
                goTypes["return"] = retType
            }
            goParams := translateTsParamsGo(params)
            out += fmt.Sprintf("func %s(%s) %s {\n", toSnakeCase(name), goParams, goTypes["return"])
            bodyLines, consumed := collectBracedBody(lines, i)
            wroteBody := false
            for _, bodyLine := range bodyLines {
                statement := translateTsStatementToGo(bodyLine)
                if statement != "" {
                    out += fmt.Sprintf("    %s\n", statement)
                    wroteBody = true
                }
            }
            if !wroteBody {
                out += fmt.Sprintf("    %s\n", "string")
            }
            out += "}\n"
            i += consumed
            continue
        }
        i++
    }
    return out
}

func translateTsParamsGo(tsParams string) string {
    params := strings.Split(tsParams, ",")
    out := ""
    for _, param := range params {
        trimmed := strings.TrimSpace(param)
        if trimmed != "" {
            parts := strings.Split(trimmed, ":")
            if len(parts) == 2 {
                name := strings.TrimSpace(parts[0])
                goType := mapTsTypeGo(parts[1])
                out += fmt.Sprintf("%s %s, ", toSnakeCase(name), goType)
            } else {
                out += fmt.Sprintf("%s string, ", trimmed)
            }
        }
    }
    return strings.TrimSuffix(out, ", ")
}

func mapTsTypeGo(tsType string) string {
    return ""
}

func collectBracedBody(lines
package main

import (
	"fmt"

	"github.com/go-git/go-git/v5"
)

func mapTsTypeGo(ty string) (mapPlumbingHash string) {
	mapPlumbingHash = ty
	return
}

func translateTsParamsGo(params string) (paramsStr string) {
	// Add logic here to translate TypeScript parameters to Go
	paramsStr = params
	return
}

func collectTsBody(lines []string, i int) (bodyLines []string, consumed int) {
	// Add logic here to collect the body of the function
	bodyLines = lines[i:]
	consumed = len(bodyLines)
	return
}

func translateTsStatementToGo(stmt string) (translatedStmt string) {
	// Add logic here to translate a TypeScript statement to Go
	translatedStmt = stmt
	return
}

func toPascalCase(name string) (goName string) {
	goName = camelCaseToPascalCase(name)
	return
}

func camelCaseToPascalCase(s string) (result string) {
	result = ""
	letters := []rune(s)
	for i, c := range letters {
		switch {
		case i == 0 || c == '_' || (c >= '0' && c <= '9'):
			result += string(c)
		default:
			result += string(c).ToUpper()
		}
	}
	return
}

func translateTsFunction(lines []string, lineIndex int) (body string) {
	for lineIndex < len(lines) {
		line := lines[lineIndex]
		line = line.Trim()
		if line.StartsWith("import ") {
			lineIndex++
			continue
		}
		if line.StartsWith("export function ") || line.StartsWith("function ") {
			cleanLine := line.TrimStart("export ").TrimStart("async ").TrimStart("function ")
			if openParen := cleanLine.Find('('); openParen != -1 {
				if closeParen := cleanLine.RFind(')'); closeParen != -1 {
					name := &cleanLine[:openParen]
					params := &cleanLine[openParen+1:closeParen]
					afterParams := cleanLine[closeParen+1:]
					retType := ""
					if arrow := afterParams.Find("->"); arrow != -1 {
						retType = mapPyTypeGo(afterParams[arrow+2:].Trim())
					}
					goParams := translateTsParamsToGo(params)
					bodyLines, consumed := collectTsBody(lines, lineIndex)
					goName := toPascalCase(name)

					if retType == "" {
						body += "func " + goName + "(" + goParams + ") {"
					} else {
						body += "func " + goName + "(" + goParams + ") " + retType + " {"
					}

					for _, bl := range bodyLines {
						if translatedStmt := translateTsStatementToGo(bl); translatedStmt != "" {
							body += "\t" + translatedStmt + "\n"
						}
					}
					body += "}\n\n"
					lineIndex += consumed
					continue
				}
			}
		}
		lineIndex++
	}
	return
}

func translateTsCodeToGo(code string) (goCode string) {
	lines := strings.Split(code, "\n")
	i := 0

	body := translateTsFunction(lines, i)

	out := "package main\n\n"
	if body.Contains("fmt.") {
		out += "import \"fmt\"\n\n"
	}
	out += body
	return
}

func main() {
	// Test the function
	code := `
		export function foo(bar: string): int {
			console.log("Hello");
			return 1;
		}
	`
	goCode := translateTsCodeToGo(code)
	fmt.Println(goCode)
}
func translate_ts_statement_to_go(line string) (string, error) {
    trimmed := strings.TrimSuffix(line, ";")

    if trimmed == "" {
        return "", nil
    }

    if strings.HasPrefix(trimmed, "return ") {
        rest := strings.TrimPrefix(trimmed, "return ")
        expr := map_ts_expr_go(rest)
        return fmt.Sprintf("return %s", expr), nil
    }

    if strings.HasPrefix(trimmed, "const ") || strings.HasPrefix(trimmed, "let ") || strings.HasPrefix(trimmed, "var ") {
        clean := trimmed
        clean = strings.TrimPrefix(clean, "const ")
        clean = strings.TrimPrefix(clean, "let ")
        clean = strings.TrimPrefix(clean, "var ")

        parts := strings.SplitN(clean, "=", 2)

        if len(parts) == 2 {
            name := parts[0]
            name = strings.TrimPrefix(name, ":")
            expr := map_ts_expr_go(parts[1])
            return fmt.Sprintf("%s := %s", name, expr), nil
        }
    }

    if strings.HasPrefix(trimmed, "console.log(") && strings.HasSuffix(trimmed, ")") {
        inner := strings.TrimPrefix(strings.TrimSuffix(trimmed, ")"), "console.log(")
        expr := map_ts_expr_go(inner)
        return fmt.Sprintf("fmt.Println(%s)", expr), nil
    }

    if strings.HasSuffix(trimmed, ")") {
        expr := map_ts_expr_go(trimmed)
        return fmt.Sprintf("%s", expr), nil
    }

    return "", nil
}

func map_ts_expr_go(expr string) string {
    out := strings.Trim(expr, " ")
    if out == "true" || out == "false" {
        return out
    }
    if out == "null" || out == "undefined" {
        return "nil"
    }
    return out
}
func mapTsExprGo(expr string) string {
    out := expr.trim()
    if out == "null" {
        return "nil"
    }
    if out == "true" {
        return "true"
    }
    if out == "false" {
        return "false"
    }
    return strings.ReplaceAll(out, "this.", "")
}

func translateTsReturnStatement(statement string) *plumbing.Hash {
    trimmed := statement.trim()
    if !strings.ContainsTrimed(trimmed, "return") {
        return nil
    }
    inner := strings.TrimSpace(strings.TrimPrefix(trimmed, "return "))
    if inner == "" {
        return nil
    }
    return plumbing.NewHash().Merge(mapTsExprGo(inner).Hash())
}

func parseTsField(line string) (*plumbing.Hash, string, error) {
    clean := strings.TrimSpace(strings.TrimSuffix(line, ";").TrimSuffix(","))
    colon := strings.Index(clean, ":")
    if colon < 0 {
        return nil, nil, fmt.Errorf("field is missing a colon: %s", line)
    }
    name := strings.TrimSpace(clean[:colon])
    value := strings.TrimSpace(clean[colon+1:])
    return plumbing.NewHash().Merge(mapTsExprGo(value).Hash()), name, nil
}

func translateTsParamsGo(params string) string {
    parts := strings.Split(params, ",")
    var partStrings []string
    for _, part := range parts {
        trimmed := strings.TrimSpace(part)
        if trimmed == "" {
            continue
        }
        pieces := strings.SplitN(trimmed, ":", 2)
        name := strings.TrimSpace(pieces[0].TrimPrefix("?"))
        goType := mapTsTypeGo(pieces[1])
        partStrings = append(partStrings, strings.TrimSpace(name)+" "+goType)
    }
    return strings.Join(partStrings, ", ")
}

func mapTsTypeGo(tsType string) string {
    switch tsType {
    case "string":
        return "string"
    case "number":
        return "int"
    case "boolean", "bool":
        return "bool"
    case "void":
        return ""
    case "any", "unknown":
        return "interface{}"
    case "string[]", "Array<string>":
        return "[]string"
    case "number[]", "Array<number>":
        return "[]int"
    default:
        return "interface{}"
    }
}
package main

import (
	"context"
	"fmt"
	"strings"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git-go/plugin/v5/handler"
)

func collectBracedBody(lines []string, start int) (body []string, consumed int) {
	body = make([]string, 0)
	braceDepth := 0
	foundOpen := false
	i := start

	for i < len(lines) {
		line := lines[i]
		for _, c := range line {
			if c == '{' {
				braceDepth++
				foundOpen = true
			} else if c == '}' {
				braceDepth--
			}
		}

		if foundOpen && i > start {
			if braceDepth > 0 {
				body = append(body, line)
			} else {
				// closing brace line — don't include it
				i++
				break
			}
		}

		i++
		if foundOpen && braceDepth == 0 {
			break
		}
	}

	consumed = i - start
	return
}

func collectPyBody(lines []string, start int) (body []string, consumed int) {
	body = make([]string, 0)
	baseIndent := leadingSpaces(lines[start])
	i := start + 1

	for i < len(lines) {
		line := lines[i]
		if strings.TrimSpace(line) == "" {
			body = append(body, "")
			i++
			continue
		}
		indent := leadingSpaces(line)
		if indent <= baseIndent {
			break
		}
		body = append(body, line)
		i++
	}

	consumed = i - start
	return
}

func leadingSpaces(s string) int {
	return len(s) - strings.TrimSpace(s).len()
}

func toSnakeCase(name string) string {
	var result strings.Builder
	for _, c := range name {
		if c == ' ' {
			result.WriteByte('_')
		} else if c >= 'A' && c <= 'Z' {
			result.WriteByte(c + 32)
		} else {
			result.WriteByte(c)
		}
	}
	return result.String()
}

type GoCommitHandler struct{}

func (h GoCommitHandler) UpdateRef(_ context.Context, ref plumbing.ReferenceName, commit *git.Commit) error {
	return nil
}

func (h GoCommitHandler) ResolveMerge(ctx context.Context, head plumbing.Hash, base plumbing.Hash, merge plumbing.Hash, strategy handler.MergeStrategy) (plumbing.Hash, error) {
	return head, nil
}

func (h GoCommitHandler) UpdateTree(ctx context.Context, tree *git.Tree, index *git.Storer, delta *git.Storer) error {
	return nil
}

func (h GoCommitHandler) MergeTree(ctx context.Context, commit *git.Commit, tree *git.Tree, other *git.Tree, target plumbing.Hash) error {
	return nil
}
