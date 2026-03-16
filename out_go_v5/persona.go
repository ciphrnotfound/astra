package main

import (
	"encoding/xml"
	"encoding/yaml"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"strings"
)

type Persona struct {
	Name                 string                 `yaml:"name" xml:"name"`
	Language             string                 `yaml:"language" xml:"language" default:"professional english"`
	Roast_level           string                 `yaml:"roast_level" xml:"roast_level" default:"none"`
	Catchphrase          string                 `yaml:"catchphrase" xml:"catchphrase"`
	Model                *string                `yaml:"model"`
	API_key              *string                `yaml:"api_key"`
}

type personaResponse struct {
	Persona `xml:",inline"`
}

func defaultLanguage() string {
	return "professional english"
}

func defaultRoastLevel() string {
	return "none"
}

func defaultPersonaName() string {
	userName := os.Getenv("USER")
	if userName == "" {
		userName = os.Getenv("USERNAME")
	}
	if userName != "" {
		return fmt.Sprintf("Astra (%s)", userName)
	}
	return "Astra"
}

func (p *Persona) Load(root string) error {
	preferredPersonaFile := filepath.Join(root, ".astra", "persona.yaml")
	previousPersonaFile := filepath.Join(root, ".forge", "persona.yaml")
	legacyPersonaFile := filepath.Join(root, ".codex", "persona.yaml")

	if _, err := os.Stat(preferredPersonaFile); !os.IsNotExist(err) {
		contents, err := ioutil.ReadFile(preferredPersonaFile)
		if err == nil {
			persona := Persona{}
			err = yaml.Unmarshal(contents, &persona)
			if err == nil {
				return nil
			}
		}
	}

	if _, err := os.Stat(previousPersonaFile); !os.IsNotExist(err) {
		contents, err := ioutil.ReadFile(previousPersonaFile)
		if err == nil {
			persona := Persona{}
			err = yaml.Unmarshal(contents, &persona)
			if err == nil {
				return nil
			}
		}
	}

	if _, err := os.Stat(legacyPersonaFile); !os.IsNotExist(err) {
		contents, err := ioutil.ReadFile(legacyPersonaFile)
		if err == nil {
			persona := Persona{}
			err = yaml.Unmarshal(contents, &persona)
			if err == nil {
				return nil
			}
		}
	}

	p.Name = defaultPersonaName()
	p.Language = defaultLanguage()
	p.Roast_level = defaultRoastLevel()
	p.Catchphrase = ""

	return nil
}

func FromVibe(vibe string) Persona {
	cleanVibe := strings.ToLower(strings.Trim(vibe, "-"))
	matchVibe := cleanVibe

	p := Persona{
		Name:                 "",
		Language:             "",
		Roast_level:           "",
		Catchphrase:          "",
		Model:                nil,
		API_key:              nil,
	}

	switch matchVibe {
	case "nigerian-pidgin":
		p = Persona{
			Name:                 "Oga Astra",
			Language:             "Nigerian Pidgin mixed with Yoruba. Speak like a professional Senior Dev from a Lagos tech hub. Use Pidgin and Yoruba words naturally but sparingly (max once or twice per response). DO NOT repeat words like 'Omo' or 'OmO' over and over. Be helpful and direct.",
			Roast_level:           "none",
			Catchphrase:          "No shaking, we go run am.",
			Model:                nil,
			API_key:              nil,
		}

	case "brutal":
		p = Persona{
			Name:                 "Linus",
			Language:             "English",
			Roast_level:           "extreme, no sugarcoating, savage, direct",
			Catchphrase:          "This is garbage.",
			Model:                nil,
			API_key:              nil,
		}

	case "hype-man":
		p = Persona{
			Name:                 "Hype",
			Language:             "Gen Z slang, lots of emojis",
			Roast_level:           "none, incredibly supportive and enthusiastic",
			Catchphrase:          "W W W W LET'S GOOOOO!",
			Model:                nil,
			API_key:              nil,
		}

	case "shakespeare":
		p = Persona{
			Name:                 "The Bard",
			Language:             "Early Modern English (Shakespearean)",
			Roast_level:           "witty, dramatic",
			Catchphrase:          "Alas, poor codebase!",
			Model:                nil,
			API_key:              nil,
		}

	case "senior-engineer":
		p = Persona{
			Name:                 "Senior Dev",
			Language:             "Jaded corporate English",
			Roast_level:           "disappointed but helpful, heavy sighing",
			Catchphrase:          "*sigh* I remember when we didn't need a framework for this...",
			Model:                nil,
			API_key:              nil,
		}

	default:
		if err := p.Load("."); err == nil {
			return p
		}
	}

	return p
}

func (p *Persona) SystemPrompt() string {
	prompt := fmt.Sprintf("Your name is %s. You must reply in the following language/tone: %s. ", p.Name, p.Language)

	if p.Roast_level != "none" {
		prompt += fmt.Sprintf("Your 'roast level' is %s. You should heavily critique, roast, and stylize your feedback according to this level. ", p.Roast_level)
	} else {
		prompt += "Be helpful, professional, and do not be excessively rude. "
	}

	if len(p.Catchphrase) > 0 {
		prompt += fmt.Sprintf("Try to organically include your catchphrase: %s. ", p.Catchphrase)
	}

	prompt += "Always stay in character. Never break character or acknowledge that you are an AI playing a character."

	return prompt
}

func main() {
	// Example usage
	vibe := "brutal"
	persona := FromVibe(vibe)
	log.Println(persona.Name)
	log.Println(persona.SystemPrompt())
}