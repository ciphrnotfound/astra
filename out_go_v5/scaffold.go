package main

import (
	"fmt"
	"strings"
)

type ScaffoldPlan struct {
	Stack    string
	Commands []string
	Notes    []string
}

func planScaffold(stack string) ScaffoldPlan {
	switch stack {
	case "node-express":
		return ScaffoldPlan{
			Stack: stack,
			Commands: []string{
				"mkdir backend && cd backend",
				"npm init -y",
				"npm install express",
				"npm install --save-dev typescript @types/node @types/express",
				"npx tsc --init",
			},
			Notes: []string{
				"Create src/index.ts with a basic Express server.",
				"Add npm scripts for dev (nodemon/ts-node) and build.",
			},
		}
	case "rust-axum":
		return ScaffoldPlan{
			Stack: stack,
			Commands: []string{
				"cargo new backend",
				"cd backend",
				`cargo add axum tokio --features tokio/full`,
			},
			Notes: []string{
				"Update src/main.rs to start an Axum HTTP server.",
				"Add routes and extractors for your core API endpoints.",
			},
		}
	default:
		return ScaffoldPlan{
			Stack: stack,
			Commands: []string{},
			Notes: []string{
				"No predefined scaffold for this stack.",
				"Describe the stack in detail and let the LLM propose a plan.",
			},
		}
	}
}

func main() {
	fmt.Println(planScaffold("node-express"))
	fmt.Println(planScaffold("rust-axum"))
	fmt.Println(planScaffold("unknown"))
}