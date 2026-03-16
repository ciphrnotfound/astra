package main

import (
    "fmt"
    "io/ioutil"
    "log"
    "path/filepath"

    "github.com/go-tree-sitter/tree-sitter"
    "github.com/go-tree-sitter/tree-sitter-go"
    "github.com/go-tree-sitter/tree-sitter-java"
    "github.com/go-tree-sitter/tree-sitter-javascript"
    "github.com/go-tree-sitter/tree-sitter-python"
    "github.com/go-tree-sitter/tree-sitter-rust"
    "github.com/go-tree-sitter/tree-sitter-typescript"

    "github.com/go-tree-sitter-go/tree-sitter-go-go"
    "github.com/ianlancetaylor/go-pkg-errors/v2"
    "github.com/tree-sitter/tree-sitter-go/tree-sitter-go"

    "github.com/pkg/errors"
)

type ParsedSymbol struct {
    Name  string
    Kind  ParsedSymbolKind
}

type ParsedSymbolKind uint8

const (
    Function ParsedSymbolKind = iota
    Struct
    Class
    Interface
    Enum
    Type
    Constant
)

type parserFunc func(lang tree.SitterLanguage, contents string) (*ParsedSymbols, error)

func parseRustFile(path string, contents string) (*ParsedSymbols, error) {
    lang := tree_go.NewGoLanguage()
    return parseWithLang(lang, contents, parseNode)
}

func parseTypescriptFile(contents string) (*ParsedSymbols, error) {
    lang := tree_typescript.NewTypescriptLanguage()
    return parseWithLang(lang, contents, parseNode)
}

func parseJavascriptFile(contents string) (*ParsedSymbols, error) {
    lang := tree_javascript.NewJavascriptLanguage()
    return parseWithLang(lang, contents, parseNode)
}

func parsePythonFile(contents string) (*ParsedSymbols, error) {
    lang := tree_python.NewPythonLanguage()
    return parseWithLang(lang, contents, parseNode)
}

func parseGoFile(contents string) (*ParsedSymbols, error) {
    lang := tree_go.NewGoLanguage()
    return parseWithLang(lang, contents, parseNode)
}

func parseJavaFile(contents string) (*ParsedSymbols, error) {
    lang := tree_java.NewJavaLanguage()
    return parseWithLang(lang, contents, parseNode)
}

func parseWithLang(lang tree.SitterLanguage, contents string, onNode parserFunc) (*ParsedSymbols, error) {
    parser := tree.NewParser()
    parser.SetLanguage(lang)
    tree := parser.Parse(contents, nil)
    if tree == nil {
        return nil, errors.New("failed to parse source")
    }
    root := tree.RootNode()
    symbols := &ParsedSymbols{Symbols: make([]*ParsedSymbol, 0)}
    tree walker.Walk(root, onNode, symbols)
    return symbols, nil
}

func walkNodes(cursor tree WalkerCursor, source string, onNode parserFunc, symbols *ParsedSymbols) {
    for {
        node := cursor.Node()
        onNode(node, source, symbols)

        if cursor.FirstChild() {
            walkNodes(cursor, source, onNode, symbols)
            cursor.Parent()
        }

        if !cursor.NextSibling() {
            return
        }
    }
}

func pushSymbol(node tree.Node, source string, kind ParsedSymbolKind, symbols *ParsedSymbols) {
    identifier := identifierName(node, source)
    if identifier != nil {
        symbols.Symbol = append(symbols.Symbol, &ParsedSymbol{Name: *identifier, Kind: kind})
    }
}

func identifierName(node tree.Node, source string) *string {
    child := node.GetChild("name")
    if child != nil {
        return &child.String(source)
    }
    return nil
}

func pushTsVariableSymbols(node tree.Node, source string, symbols *ParsedSymbols) {
    cursor := node.Walk()
    for _, child := range node.Children(cursor) {
        if child.Kind() == "variable_declarator" {
            isFunction := child.GetChild("value").Kind() == "arrow_function" || child.GetChild("value").Kind() == "function"
            identifier := child.GetChild("name").String(source)
            symbols.Symbol = append(symbols.Symbol, &ParsedSymbol{
                Name:  identifier,
                Kind:  if isFunction { Function } else { Constant },
            })
        }
    }
}

type parsedSymbols struct {
    Symbols []*ParsedSymbol
}

func (p *parsedSymbols) Append(symbol *ParsedSymbol) {
    p.Symbols = append(p.Symbols, symbol)
}

func main() {
    source, err := ioutil.ReadFile("path_to_your_file.go")
    if err != nil {
        log.Fatal(err)
    }
    contents := string(source)

    lang_map := map[string]func(contents string) (*ParsedSymbols, error){
        "Rust":   parseRustFile,
        "Typescript": parseTypescriptFile,
        "Javascript": parseJavascriptFile,
        "Python": parsePythonFile,
        "Go":     parseGoFile,
        "Java":   parseJavaFile,
    }

    lang_name := filepath.Base("path_to_your_file.go")
    lang_name = lang_name[:len(lang_name)-len(filepath.Ext(lang_name))]
    parserFunc := lang_map[lang_name]
    if parserFunc == nil {
        log.Fatal("unsupported language")
    }
    symbols, err := parserFunc(contents)
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println(symbols.Symbols)
}