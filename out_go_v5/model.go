package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"

	"github.com/PuerkitoBio/goquery"
	"github.com/google/uuid"
	"github.com/valyala/bytebufferpool"
)

type CodexModel interface {
	Complete(prompt string) (string, error)
}

type SearchProvider interface {
	Search(query string) (string, error)
}

type GroqModel struct {
	Client  *http.Client
	ApiKey  string
	Model   string
	Endpoint string
}

type OllamaModel struct {
	Client  *http.Client
	Model   string
	Endpoint string
}

type TavilySearchRequest struct {
	ApiKey      string   `json:"api_key"`
	Query       string   `json:"query"`
	SearchDepth string   `json:"search_depth"`
	IncludeRaw  bool     `json:"include_raw_content"`
	MaxResults  int      `json:"max_results"`
}

type TavilySearchResponse struct {
	Results []TavilyResult `json:"results"`
}

type TavilyResult struct {
	Title  string `json:"title"`
	Url    string `json:"url"`
	Content string `json:"content"`
}

type TavilySearch struct {
	Client  *http.Client
	ApiKey  string
}

func (t *TavilySearch) fromEnv() (*TavilySearch, error) {
	apiKey := os.Getenv("TAVILY_API_KEY")
	if apiKey == "" {
		return nil, fmt.Errorf("TAVILY_API_KEY environment variable is not set")
	}

	t.Client = &http.Client{}
	t.ApiKey = apiKey

	return t, nil
}

func (t *TavilySearch) search(query string) (string, error) {
	body := TavilySearchRequest{
		ApiKey:      t.ApiKey,
		Query:       query,
		SearchDepth: "basic",
		IncludeRaw:  false,
		MaxResults:  5,
	}

	req, err := http.NewRequest("POST", "https://api.tavily.com/search", json.Marshal(body))
	if err != nil {
		return "", err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := t.Client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if !http.StatusOK == resp.StatusCode {
		return "", fmt.Errorf("Tavily API error: %d", resp.StatusCode)
	}

	var parsed TavilySearchResponse
	err = json.NewDecoder(resp.Body).Decode(&parsed)
	if err != nil {
		return "", err
	}

	var summary string
	for _, res := range parsed.Results {
		summary += fmt.Sprintf("### %s\nURL: %s\n%s\n\n", res.Title, res.Url, res.Content)
	}

	return summary, nil
}

type GroqModelRequest struct {
	Model string `json:"model"`
	Messages []GroqMessage `json:"messages"`
}

type GroqMessage struct {
	Role string `json:"role"`
	Content string `json:"content"`
}

type GroqChoice struct {
	Message GroqMessage `json:"message"`
}

type GroqChatResponse struct {
	Choices []GroqChoice `json:"choices"`
}

type GroqModelImpl struct {
	Client  *http.Client
	ApiKey  string
	Model   string
	Endpoint string
}

func NewGroqModel() (*GroqModelImpl, error) {
	ApiKey := os.Getenv("GROQ_API_KEY")
	if ApiKey == "" {
		return nil, fmt.Errorf("GROQ_API_KEY environment variable is not set")
	}

	Client := &http.Client{}
	ApiKey = os.Getenv("GROQ_API_KEY")
	if ApiKey == "" {
		ApiKey = "your-api-key-here"
	}

	Model := os.Getenv("GROQ_MODEL")
	if Model == "" {
		Model = "llama3.1:8b"
	}

	Endpoint := os.Getenv("GROQ_ENDPOINT")
	if Endpoint == "" {
		Endpoint = "https://api.groq.com/openai/v1/chat/completions"
	}

	return &GroqModelImpl {
		Client: Client,
		ApiKey: ApiKey,
		Model: Model,
		Endpoint: Endpoint,
	}, nil
}

func (g *GroqModelImpl) complete(prompt string) (string, error) {
	body := GroqModelRequest {
		Model: g.Model,
		Messages: []GroqMessage {
			{ Role: "user", Content: prompt },
		},
	}

	req, err := http.NewRequest("POST", g.Endpoint, json.Marshal(body))
	if err != nil {
		return "", err
	}
	req.Header.Set("Authorization", "Bearer "+g.ApiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := g.Client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		var buffer bytebufferpool.Buffer
		n, err := resp.Body.Read(buffer.Bytes())
		if err != nil {
			return "", err
		}

		var parsed map[string]interface{}
		err = json.Unmarshal(buffer.Bytes()[:n], &parsed)
		if err != nil {
			return "", err
		}

		return "", fmt.Errorf("Groq API error: %v - %v", resp.Status, parsed)
	}

	var parsed GroqChatResponse
	err = json.NewDecoder(resp.Body).Decode(&parsed)
	if err != nil {
		return "", err
	}

	return parsed.Choices[0].Message.Content, nil
}

type OllamaModelImpl struct {
	Client  *http.Client
	Model   string
	Endpoint string
}

func NewOllamaModel() (*OllamaModelImpl, error) {
	Model := os.Getenv("OLLAMA_MODEL")
	if Model == "" {
		Model = "llama3.1:8b"
	}

	Endpoint := os.Getenv("OLLAMA_ENDPOINT")
	if Endpoint == "" {
		Endpoint = "http://localhost:11434/api/generate"
	}

	return &OllamaModelImpl{
		Model: Model,
		Endpoint: Endpoint,
	}, nil
}

func (o *OllamaModelImpl) complete(prompt string) (string, error) {
(body, err := json.Marshal(GroqModelRequest {
		Model: o.Model,
		Messages: []GroqMessage {
			{ Role: "user", Content: prompt },
		},
}))

	req, err := http.NewRequest("POST", o.Endpoint, body)
	if err != nil {
		return "", err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := o.Client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("Ollama API error: %d - %s", resp.StatusCode, resp.Status)
	}

	var parsed GroqChatResponse
	err = json.NewDecoder(resp.Body).Decode(&parsed)
	if err != nil {
		return "", err
	}

	return parsed.Choices[0].Message.Content, nil
}
```
This Go program contains four implementations for the `CodexModel` and `SearchProvider` interfaces. The implementations are:
- `TavilySearch`: a search provider for the Tavily API.
- `GroqModelImpl`: a codex model for the Groq API.
- `OllamaModelImpl`: a codex model for the Ollama API.
Each implementation provides a way to interact with its respective provider or model using a consistent interface.
```go
func main() {
	// Create new instances of the models
	models := []interface{}{
		TavilySearch{Client: &http.Client{ }},
		GroqModelImpl{Client: &http.Client{}},
		GroqModelImpl{Client: &http.Client{}},
		OllamaModelImpl{Client: &http.Client{}},
	}

	// Test each model
	for _, model := range models {
		t, err := model.(CodexModel).complete("test")
		if err != nil {
			log.Println(err)
		}
		log.Println(t)
	}
}
```
This `main` function demonstrates how to use each model's `Complete` method, passing in a prompt and handling any errors that may be returned.
```go
func TestCodexModel(t *testing.T) {
	// Create new instances of the models
	models := []interface{}{
		TavilySearch{Client: &http.Client{ }},
		GroqModelImpl{Client: &http.Client{}},
		GroqModelImpl{Client: &http.Client{}},
		OllamaModelImpl{Client: &http.Client{}},
	}

	// Test each model
	for _, model := range models {
		t, err := model.(CodexModel).complete("test")
		if err != nil {
			t.Errorf("Test failed: %v", err)
		}
		if t != "test" {
			t.Errorf("Test failed: expected 'test' but got '%s'", t)
		}
	}
}