package refactorbot

import (
	"bytes"
	"fmt"
	"strings"
)

type CodeSmell struct {
	Name        string
	Description string
	LineHint    *int
}

type CleanupEngine struct {
	Model CodexModel
}

type CodexModel interface {
	Complete(prompt string) (string, error)
}

func NewCleanupEngine(model CodexModel) *CleanupEngine {
	return &CleanupEngine{Model: model}
}

func (c *CleanupEngine) DetectSmells(code string, lang Language) []*CodeSmell {
	var smells []*CodeSmell

	switch lang {
	case Python:
		// Smell: JSON dump of objects
		if strings.Contains(code, "json.dump(") && !strings.Contains(code, "asdict(") && !strings.Contains(code, "JSONEncoder") {
			smells = append(smells, &CodeSmell{
				Name: "Custom Object Serialization",
				Description: "Found `json.dump` without `asdict` or custom encoder. This will crash for custom classes.",
				LineHint:     nil,
			})
		}
		// Smell: Missing dataclasses
		if strings.Contains(code, "def __init__(self") && !strings.Contains(code, "@dataclass") {
			smells = append(smells, &CodeSmell{
				Name: "Missing Dataclasses",
				Description: "Boilerplate __init__ mirrors source struct; consider using @dataclass.",
				LineHint:     nil,
			})
		}
		// Smell: Static methods for defaults
		if strings.Contains(code, "@staticmethod") && (strings.Contains(code, "default") || strings.Contains(code, "max_entries")) {
			smells = append(smells, &CodeSmell{
				Name: "Rust-style Static Helpers",
				Description: "Static methods like `default_max_entries` should be converted to default parameters.",
				LineHint:     nil,
			})
		}
	}

	return smells
}

func (c *CleanupEngine) Clean(code string, lang Language) (string, []*CodeSmell, error) {
	smells := c.DetectSmells(code, lang)
	if len(smells) == 0 {
		return code, nil, nil
	}

	smellsDescription := make([]string, len(smells))
	for i, s := range smells {
		smellsDescription[i] = fmt.Sprintf("- %s: %s", s.Name, s.Description)
	}
	smellsDescriptionStr := strings.Join(smellsDescription, "\n")

	prompt := fmt.Sprintf(`You are an expert Refactor Bot. Clean up the following %s code to be fully idiomatic.

WE DETECTED THESE MIGRATION SMELLS:
%s

CODE TO FIX:
%s

RULES:
- FIX ALL detected smells.
- Use standard idiomatic patterns (e.g., @dataclass, typing, proper serialization).
- Output ONLY the fixed code, no explanations or markdown fences.

`, lang, smellsDescriptionStr, code)

	cleaned, err := c.Model.Complete(prompt)
	if err != nil {
		return "", nil, err
	}
	cleaned = strings.TrimSpace(cleaned)
	if strings.HasPrefix(cleaned, "```") {
		cleaned = cleaned[3:]
	}

	return cleaned, smells, nil
}

func (c *CleanupEngine) stripMarkdown(s string) string {
	var out bytes.Buffer
	if strings.HasPrefix(s, "```") {
		out.WriteString(strings.TrimSpace(s[3:]))
		out.WriteString("\n")
	}
	return out.String()
}

const (
	Python = iota
)