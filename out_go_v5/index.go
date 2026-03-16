package main

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/your-username/go-library-for-golang-extraction"
	"github.com/your-username/astra-parser/rust-parser"
	"github.com/your-username/astra-parser/typescript-parser"
	"github.com/your-username/astra-parser/javascript-parser"
	"github.com/your-username/astra-parser/python-parser"
	"github.com/your-username/astra-parser/go-parser"
	"github.com/your-username/astra-parser/java-parser"
)

type CodeIndex struct {
	files map[string]FileSummary
	graph *SemanticGraph
}

type FileSummary struct {
	lineCount    int
	language     string
	approxFncount int
	symbols      []SymbolSummary
}

type SymbolKind int

const (
	Function        SymbolKind = iota
_struct_
_class_
_Interface_
(Enum)         SymbolKind = iota
_Type_
_Constant_
)

type SymbolSummary struct {
	name  string
	kind  SymbolKind
}

func (c CodeIndex) Stats() IndexStats {
	var fileCount, totalLines int
	for _, summary := range c_files {
		fileCount++
		totalLines += summary.lineCount
	}
	return IndexStats{fileCount, totalLines, c.graph.stats()}
}

func (c CodeIndex) FilesByLanguage() map[string]int {
	counts := map[string]int{}
	for _, summary := range c.files {
		counts[summary.language]++
	}
	return counts
}

func (c CodeIndex) SymbolsByLanguage() map[string]int {
	counts := map[string]int{}
	for _, summary := range c.files {
		counts[summary.language] += len(summary.symbols)
	}
	return counts
}

func (c CodeIndex) Files() map[string]FileSummary {
	return c.files
}

func (c CodeIndex) LinesByLanguage() map[string]int {
	counts := map[string]int{}
	for _, summary := range c.files {
		counts[summary.language] += summary.lineCount
	}
	return counts
}

func (c CodeIndex) TotalFncount() int {
	sum := 0
	for _, file := range c.files {
		sum += file.approxFncount
	}
	return sum
}

func (c CodeIndex) TotalSymbolCount() int {
	sum := 0
	for _, file := range c.files {
		sum += len(file.symbols)
	}
	return sum
}

func (c CodeIndex) GraphStats() GraphStats {
	return c.graph.stats()
}

func (c CodeIndex) GraphDot() string {
	return c.graph.toString()
}

type IndexStats struct {
	fileCount    int
	totalLines   int
	graphStats   GraphStats
}

type GraphStats struct {
	nodeCount             int
	edgeCount             int
	fileNodes             int
	symbolNodes           int
}

type SemanticGraph struct {
	graph  map[int]GraphNode
	nodes  map[string]int
	edges  map[int]Edge
	fileNodes map[string]int
}

type GraphNode string

type Edge string

func (graph SemanticGraph) Stats() GraphStats {
	var (
		nodeCount, edgeCount, fileNodes, symbolNodes int
	)
	for _, node := range graph.graph {
		switch node {
		case "file":
			fileNodes++
		case "symbol":
			symbolNodes++
		}
	}
	nodeCount = len(graph.graph)
	edgeCount = len(graph.edges)
	return GraphStats{
		nodeCount,
		edgeCount,
		fileNodes,
		symbolNodes,
	}
}

func (graph SemanticGraph) toString() string {
	var (
		nodes       []string
		edges       []string
		fileNodes   []string
		symbolNodes []string
	)

	for node := range graph.graph {
		switch graph.graph[node] {
		case "file":
			fileNodes = append(fileNodes, fmt.Sprintf("node_%d[label=\"file\\n%s\"]", node, graph.nodes[string(node)]))
		case "symbol":
			symbolNodes = append(symbolNodes, fmt.Sprintf("node_%d[label=\"symbol\\n%s\"]", node, graph.nodes[string(node)]))
		}
		nodes = append(nodes, fmt.Sprintf("node_%d", node))
	}
	for id, edge := range graph.edges {
		edges = append(edges, fmt.Sprintf("node_%d --> node_%d [label=\"%s\"];\n", edge.src, edge.tgt, edge.name))
	}

	graphviz := bytes.NewBufferString("")
	graphviz.WriteString("digraph {\n")
	graphviz.WriteString(strings.Join(nodes, "\n"))
	graphviz.WriteString("\n")
	graphviz.WriteString(strings.Join(fileNodes, "\n"))
	graphviz.WriteString("\n")
	graphviz.WriteString(strings.Join(symbolNodes, "\n"))
	graphviz.WriteString("\n")
	graphviz.WriteString(strings.Join(edges, "\n"))
	graphviz.WriteString("}\n")

	return graphviz.String()
}

func NewCodeIndex() CodeIndex {
	return CodeIndex{
		files: map[string]FileSummary{},
		graph: &SemanticGraph{
			graph:     map[int]GraphNode{},
			nodes:     map[string]int{},
			edges:     map[int]Edge{},
			fileNodes: map[string]int{},
		},
	}
}

func (code CodeIndex) AddFile(path string, contents string) {
	fileSummary := FileSummary{
		lineCount:    0,
		language:     code.fileLanguage(path),
		approxFncount: 0,
		symbols:      []SymbolSummary{},
	}
	for _, line := range strings.Split(contents, "\n") {
		fileSummary.lineCount++
	}
	symbols := code.extractSymbols(fileSummary.language, contents, path)
	fileSummary.symbols = symbols

	for _, fileSummary := range code.files {
		fileSummary.approxFncount += len(fileSummary.symbols)
		_, ok := code.graph.fileNodes[filepath.FromSlash(fileSummary.language)]
		if !ok {
			code.graph.fileNodes[filepath.FromSlash(fileSummary.language)] = len(code.graph.graph)
			code.graph.graph[len(code.graph.graph)] = "file"
			code.graph.nodes[filepath.FromSlash(fileSummary.language)] = len(code.graph.graph)
		}
		for _, symbolSummary := range fileSummary.symbols {
			node := code.graph.graph[len(code.graph.graph)]
			code.graph.graph[len(code.graph.graph)] = "symbol"
			code.graph.nodes[symbolSummary.name] = len(code.graph.graph)
			if node := code.graph.fileNodes[filepath.FromSlash(fileSummary.language)]; node != 0 {
				code.graph.edges[len(code.graph.edges)] = Edge{"node_" + fmt.Sprint(node) + " --> node_" + fmt.Sprint(len(code.graph.graph)) + " [label=\"Contains\"];"}
			}
		}
	}
}

func (code CodeIndex) fileLanguage(file string) string {
	ext := strings.ToLower(filepath.Ext(file))
	switch ext {
	case ".go":
		return "go"
	case ".ts":
		return "typescript"
	case ".js":
		return "javascript"
	case ".py":
		return "python"
	case ".java":
		return "java"
	default:
		return ext
	}
}

func (code CodeIndex) extractSymbols(lang string, contents string, path string) []SymbolSummary {
 Extractors := map[string]func(string, string) []SymbolSummary{}
 Extractors["typescript"] = code.extractTsSymbols
 Extractors["javascript"] = code.extractJsSymbols
 Extractors["python"] = code.extractPythonSymbols
 Extractors["go"] = code.extractGoSymbols
 Extractors["java"] = code.extractJavaSymbols
 Extractors["rust"] = code.extractRustSymbols
	return Extractors[lang](contents)
}

func (code CodeIndex) extractTsSymbols(contents string) []SymbolSummary {
	// ...
}

func (code CodeIndex) extractJsSymbols(contents string) []SymbolSummary {
	// ...
}

func (code CodeIndex) extractPythonSymbols(contents string) []SymbolSummary {
	// ...
}

func (code CodeIndex) extractGoSymbols(contents string) []SymbolSummary {
	// ...
}

func (code CodeIndex) extractJavaSymbols(contents string) []SymbolSummary {
	// ...
}

func (code CodeIndex) extractRustSymbols(contents string, path string) []SymbolSummary {
	// ...
}
package main

import (
    "regexp"
    "strings"
)

type SymbolSummary struct {
    Name     string
    Kind     string
}

type SymbolKind string

const (
    Function  SymbolKind = "Function"
    Struct    SymbolKind = "Struct"
    Class     SymbolKind = "Class"
    Interface SymbolKind = "Interface"
    Type      SymbolKind = "Type"
    Constant  SymbolKind = "Constant"
    Enum      SymbolKind = "Enum"
)

func map_parsed_symbols(items []ParsedSymbol) []SymbolSummary {
    symbolSummaries := []SymbolSummary{}
    for _, item := range items {
        symbolSummary := SymbolSummary{
            Name: item.Name,
            Kind: KindString(item.Kind),
        }
        symbolSummaries = append(symbolSummaries, symbolSummary)
    }
    return symbolSummaries
}

func KindString(kind ParsedSymbolKind) SymbolKind {
    switch kind {
    case Function:
        return Function
    case Struct:
        return Struct
    case Class:
        return Class
    case Interface:
        return Interface
    case Type:
        return Type
    case Constant:
        return Constant
    case Enum:
        return Enum
    }

    return ""
}

func name_after_keyword(line string, keyword string) string {
    regex := regexp.MustCompile("\\b" + keyword + "\\s+(\\w+)")
    match := regex.FindStringSubmatch(line)
    if len(match) == 0 {
        return ""
    }
    name := match[1]
    return strings.TrimSpace(name)
}

func name_after_keyword_anywhere(line string, keyword string) string {
    regex := regexp.MustCompile("\\b" + keyword + "\\s+(\\w+)")
    match := regex.FindStringSubmatch(line)
    if len(match) == 0 {
        return ""
    }
    name := match[1]
    return strings.TrimSpace(name)
}

func sanitize_identifier(name string) string {
    regex := regexp.MustCompile("^[a-zA-Z_]+[a-zA-Z0-9_]*$")
    match := regex.FindStringSubmatch(name)
    if len(match) == 0 {
        return ""
    }
    trimmed := strings.TrimSpace(name)
    return trimmed
}

func parse_go_func_name(line string) string {
    if !strings.HasPrefix(line, "func") {
        return ""
    }
    trimmed := strings.TrimPrefix(line, "func ")
    var (
        name  string
        paren bool
    )
    for _, token := range strings.Fields(trimmed) {
        if paren {
            return tokenize_after_paren(paren, trimmed)
        }
        if token == "(" {
            paren = true
            continue
        }
        if token == ")" {
            paren = false
            continue
        }
        name = token
    }
    if !strings.Contains(trimmed, "(") {
        name = trimmed
    }
    return strings.TrimSpace(name)
}

func tokenize_after_paren(paren string, line string) string {
    line = strings.TrimPrefix(line, paren)
    if !strings.Contains(line, ")") {
        return ""
    }
    var (
        name string
        open bool
    )
    for _, token := range strings.Fields(line) {
        if open {
            name = token
            break
        }
        if token == "(" {
            open = true
            continue
        }
        if token == ")" {
            open = false
            continue
        }
    }
    if open {
        return name
    }
    return strings.TrimSpace(name)
}

func extract_go_symbols(contents string) []SymbolSummary {
    var symbolSummaries []SymbolSummary
    var (
        kind   SymbolKind
        name    = ""
    )
    parts := strings.Split(contents, "\n")
    for _, line := range parts {
        trimmed := strings.Trim(line, " \t")
        if trimmed == "" || strings.HasPrefix(trimmed, "//") {
            continue
        }
        if trimmed.HasPrefix("func ") {
            name = parse_go_func_name(trimmed)
            kind = Function
            if name != "" {
                symbolSummaries = append(symbolSummaries, SymbolSummary{
                    Name: name,
                    Kind: kind,
                })
            }
            continue
        }
        if trimmed.Contains("type") {
            keyword := name_after_keyword(trimmed, "type")
            name = keyword
            kind = Type
            if name != "" {
                symbolSummaries = append(symbolSummaries, SymbolSummary{
                    Name: name,
                    Kind: kind,
                })
            }
            continue
        }
    }
    return symbolSummaries
}

func extract_java_symbols(contents string) []SymbolSummary {
    var symbolSummaries []SymbolSummary
    var (
        kind   SymbolKind
        name    = ""
    )
    parts := strings.Split(contents, "\n")
    for _, line := range parts {
        trimmed := strings.Trim(line, " \t")
        if trimmed == "" || strings.HasPrefix(trimmed, "//") || strings.HasPrefix(trimmed, "*") {
            continue
        }
        if trimmed.Contains("class") {
            name = name_after_keyword_anywhere(trimmed, "class")
            kind = Class
            if name != "" {
                symbolSummaries = append(symbolSummaries, SymbolSummary{
                    Name: name,
                    Kind: kind,
                })
            }
            continue
        }
        if trimmed.Contains("interface") {
            name = name_after_keyword_anywhere(trimmed, "interface")
            kind = Interface
            if name != "" {
                symbolSummaries = append(symbolSummaries, SymbolSummary{
                    Name: name,
                    Kind: kind,
                })
            }
            continue
        }
        if trimmed.Contains("enum") {
            name = name_after_keyword_anywhere(trimmed, "enum")
            kind = Enum
            if name != "" {
                symbolSummaries = append(symbolSummaries, SymbolSummary{
                    Name: name,
                    Kind: kind,
                })
            }
            continue
        }
        if looksLikeJavaMethod(trimmed) {
            methodParts := strings.Split(trimmed, "(")
            if len(methodParts) > 1 {
                name = name_before_paren(methodParts[0])
                kind = Function
                if name != "" {
                    symbolSummaries = append(symbolSummaries, SymbolSummary{
                        Name: name,
                        Kind: kind,
                    })
                }
            }
        }
    }
    return symbolSummaries
}
