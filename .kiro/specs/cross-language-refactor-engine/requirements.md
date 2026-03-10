# Requirements Document

## Introduction

The Cross-Language Refactor Engine extends the cli_codex project to provide semantic understanding and refactoring capabilities across multiple programming languages (TypeScript, Go, Python, Rust, Java). The engine maintains a local semantic memory of the codebase structure, enables cross-language construct mapping, and supports safe multi-step refactor planning without modifying code directly. All processing occurs locally without external service dependencies.

## Glossary

- **Refactor_Engine**: The core system component that orchestrates semantic analysis, cross-language mapping, and refactor planning
- **Semantic_Graph**: A graph data structure representing code entities (functions, types, modules) as nodes and their relationships (calls, imports, inheritance) as edges
- **Language_IR**: An intermediate representation that normalizes language-specific AST constructs into language-agnostic semantic nodes
- **AST_Parser**: A component that parses source files into abstract syntax trees for a specific programming language
- **Cross_Language_Mapper**: A component that identifies equivalent constructs across different programming languages
- **Refactor_Plan**: A sequence of transformation steps with preconditions, effects, and uncertainty markers
- **Memory_Store**: A persistent storage layer for semantic graphs, historical decisions, and cross-language patterns
- **CodeIndex**: The existing index structure that maps file paths to file summaries
- **CodexEngine**: The existing engine that handles user input and manages the CodeIndex
- **Uncertainty_Marker**: A flag indicating ambiguous or risky aspects of a refactor operation
- **Construct_Mapping**: A bidirectional association between semantically equivalent constructs in different languages
- **Semantic_Node**: A language-agnostic representation of a code entity with type, name, scope, and attributes
- **LSP_Adapter**: An interface layer that exposes Refactor_Engine capabilities through the Language Server Protocol
- **Hook_Adapter**: An interface layer that integrates Refactor_Engine with git and CI workflows
- **TUI_Adapter**: An interface layer that provides terminal-based visualization of semantic graphs and refactor plans

## Requirements

### Requirement 1: Semantic Graph Construction

**User Story:** As a developer, I want the system to build a semantic graph of my codebase, so that I can understand cross-file and cross-language relationships.

#### Acceptance Criteria

1. WHEN the Refactor_Engine indexes a source file, THE AST_Parser SHALL parse the file into an abstract syntax tree
2. WHEN an abstract syntax tree is available, THE Refactor_Engine SHALL extract semantic nodes for functions, types, variables, and modules
3. WHEN semantic nodes are extracted, THE Refactor_Engine SHALL create edges representing calls, imports, inheritance, and type references
4. THE Semantic_Graph SHALL store node attributes including name, type, scope, visibility, location, and language
5. THE Semantic_Graph SHALL store edge attributes including relationship type, direction, and source location
6. WHEN indexing completes, THE Refactor_Engine SHALL persist the Semantic_Graph to the Memory_Store
7. FOR ALL indexed files, THE Refactor_Engine SHALL maintain bidirectional navigation between nodes and their source locations

### Requirement 2: Multi-Language AST Parsing

**User Story:** As a developer, I want the system to parse TypeScript, Go, Python, Rust, and Java files, so that all languages in my monorepo are understood.

#### Acceptance Criteria

1. WHEN a TypeScript file is encountered, THE AST_Parser SHALL parse interfaces, classes, functions, types, imports, and exports
2. WHEN a Go file is encountered, THE AST_Parser SHALL parse structs, interfaces, functions, methods, packages, and imports
3. WHEN a Python file is encountered, THE AST_Parser SHALL parse classes, functions, methods, dataclasses, type hints, and imports
4. WHEN a Rust file is encountered, THE AST_Parser SHALL parse structs, enums, traits, functions, methods, modules, and use statements
5. WHEN a Java file is encountered, THE AST_Parser SHALL parse classes, interfaces, methods, fields, packages, and imports
6. IF a parse error occurs, THEN THE AST_Parser SHALL log the error location and continue processing other files
7. THE AST_Parser SHALL extract documentation comments for all parsed entities

### Requirement 3: Language-Agnostic IR Normalization

**User Story:** As a developer, I want language-specific constructs normalized into a common representation, so that cross-language analysis is possible.

#### Acceptance Criteria

1. WHEN a TypeScript interface is parsed, THE Refactor_Engine SHALL create a Semantic_Node with type "interface" and field definitions
2. WHEN a Go struct is parsed, THE Refactor_Engine SHALL create a Semantic_Node with type "struct" and field definitions
3. WHEN a Python dataclass is parsed, THE Refactor_Engine SHALL create a Semantic_Node with type "dataclass" and field definitions
4. WHEN a Rust struct is parsed, THE Refactor_Engine SHALL create a Semantic_Node with type "struct" and field definitions
5. WHEN a Java class is parsed, THE Refactor_Engine SHALL create a Semantic_Node with type "class" and field definitions
6. THE Language_IR SHALL normalize type annotations across languages into canonical type representations
7. THE Language_IR SHALL normalize visibility modifiers (public, private, protected) into a unified visibility model
8. THE Language_IR SHALL preserve language-specific attributes that cannot be normalized

### Requirement 4: Cross-Language Construct Mapping

**User Story:** As a developer, I want the system to identify equivalent constructs across languages, so that I can maintain consistency in multi-language codebases.

#### Acceptance Criteria

1. WHEN analyzing the Semantic_Graph, THE Cross_Language_Mapper SHALL identify structurally similar nodes across languages
2. THE Cross_Language_Mapper SHALL create Construct_Mappings between TypeScript interfaces and Go structs with matching field names and types
3. THE Cross_Language_Mapper SHALL create Construct_Mappings between Python dataclasses and Rust structs with matching field names and types
4. THE Cross_Language_Mapper SHALL create Construct_Mappings between Java interfaces and TypeScript interfaces with matching method signatures
5. WHEN field types differ, THE Cross_Language_Mapper SHALL mark the mapping with an Uncertainty_Marker
6. WHEN field names differ but types match, THE Cross_Language_Mapper SHALL compute a similarity score and include it in the mapping
7. THE Cross_Language_Mapper SHALL store all Construct_Mappings in the Memory_Store with confidence scores

### Requirement 5: Cross-Language Inconsistency Detection

**User Story:** As a developer, I want the system to detect inconsistencies between equivalent constructs, so that I can maintain synchronization across languages.

#### Acceptance Criteria

1. WHEN a Construct_Mapping exists, THE Refactor_Engine SHALL compare field counts between mapped constructs
2. WHEN field counts differ, THE Refactor_Engine SHALL report the inconsistency with affected file locations
3. WHEN field types differ, THE Refactor_Engine SHALL report the type mismatch with expected and actual types
4. WHEN field names differ, THE Refactor_Engine SHALL report the naming inconsistency
5. WHEN a construct exists in one language but not in mapped languages, THE Refactor_Engine SHALL report the missing construct
6. THE Refactor_Engine SHALL aggregate all inconsistencies into a report with severity levels
7. THE Refactor_Engine SHALL persist detected inconsistencies to the Memory_Store with timestamps

### Requirement 6: Refactor Plan Generation

**User Story:** As a developer, I want the system to generate multi-step refactor plans, so that I can understand the impact of proposed changes.

#### Acceptance Criteria

1. WHEN a refactor operation is requested, THE Refactor_Engine SHALL analyze affected nodes in the Semantic_Graph
2. THE Refactor_Engine SHALL identify all direct and transitive dependencies of affected nodes
3. THE Refactor_Engine SHALL generate a Refactor_Plan with ordered transformation steps
4. THE Refactor_Plan SHALL include preconditions for each step that must be satisfied before execution
5. THE Refactor_Plan SHALL include expected effects for each step describing resulting changes
6. WHEN ambiguity exists in a transformation, THE Refactor_Engine SHALL add an Uncertainty_Marker to the affected step
7. THE Refactor_Plan SHALL include rollback steps for each transformation to enable safe reversal

### Requirement 7: Diff Generation

**User Story:** As a developer, I want the system to generate diffs for refactor plans, so that I can review exact changes before applying them.

#### Acceptance Criteria

1. WHEN a Refactor_Plan is finalized, THE Refactor_Engine SHALL generate unified diffs for each affected file
2. THE Refactor_Engine SHALL include file path, line numbers, and context lines in each diff
3. THE Refactor_Engine SHALL group diffs by transformation step in the Refactor_Plan
4. THE Refactor_Engine SHALL mark diffs containing Uncertainty_Markers with warning annotations
5. THE Refactor_Engine SHALL compute diff statistics including lines added, removed, and modified
6. THE Refactor_Engine SHALL validate that generated diffs produce syntactically valid code
7. THE Refactor_Engine SHALL serialize diffs in a format compatible with standard patch tools

### Requirement 8: Memory Store Persistence

**User Story:** As a developer, I want the system to persist semantic graphs and decisions, so that analysis results are preserved across sessions.

#### Acceptance Criteria

1. WHEN the Semantic_Graph is updated, THE Memory_Store SHALL persist nodes and edges to local storage
2. WHEN Construct_Mappings are created, THE Memory_Store SHALL persist mappings with confidence scores and timestamps
3. WHEN inconsistencies are detected, THE Memory_Store SHALL persist inconsistency reports with detection timestamps
4. WHEN a Refactor_Plan is generated, THE Memory_Store SHALL persist the plan with generation context
5. THE Memory_Store SHALL support incremental updates without requiring full graph recomputation
6. WHEN the Refactor_Engine starts, THE Memory_Store SHALL load the most recent Semantic_Graph
7. THE Memory_Store SHALL store all data in local files without network communication

### Requirement 9: Memory Store Querying

**User Story:** As a developer, I want to query the semantic memory, so that I can explore relationships and historical patterns.

#### Acceptance Criteria

1. THE Memory_Store SHALL support queries for nodes by name, type, or language
2. THE Memory_Store SHALL support queries for edges by relationship type or connected nodes
3. THE Memory_Store SHALL support queries for Construct_Mappings by language pair or confidence threshold
4. THE Memory_Store SHALL support queries for historical Refactor_Plans by date range or affected files
5. THE Memory_Store SHALL support graph traversal queries with depth limits
6. THE Memory_Store SHALL return query results within 100 milliseconds for graphs with up to 100,000 nodes
7. THE Memory_Store SHALL support filtering query results by file path patterns

### Requirement 10: LSP Integration Interface

**User Story:** As a developer, I want to access refactor engine capabilities through my IDE, so that I can use familiar development tools.

#### Acceptance Criteria

1. THE LSP_Adapter SHALL expose Semantic_Graph navigation through LSP "go to definition" requests
2. THE LSP_Adapter SHALL expose cross-language references through LSP "find references" requests
3. THE LSP_Adapter SHALL expose inconsistency detection through LSP diagnostic messages
4. THE LSP_Adapter SHALL expose Refactor_Plan generation through LSP code action requests
5. THE LSP_Adapter SHALL expose Construct_Mappings through LSP hover information
6. WHEN the Semantic_Graph is updated, THE LSP_Adapter SHALL publish change notifications to connected clients
7. THE LSP_Adapter SHALL communicate with the Refactor_Engine through a message-passing interface

### Requirement 11: Git Hook Integration Interface

**User Story:** As a developer, I want refactor validation in my git workflow, so that inconsistencies are caught before commit.

#### Acceptance Criteria

1. THE Hook_Adapter SHALL provide a pre-commit hook that validates changed files against the Semantic_Graph
2. WHEN a pre-commit hook runs, THE Hook_Adapter SHALL detect new cross-language inconsistencies in staged files
3. IF inconsistencies are detected, THEN THE Hook_Adapter SHALL report them and return a non-zero exit code
4. THE Hook_Adapter SHALL provide a post-commit hook that updates the Semantic_Graph with committed changes
5. THE Hook_Adapter SHALL provide a pre-push hook that validates Construct_Mappings across all changed files
6. THE Hook_Adapter SHALL complete pre-commit validation within 5 seconds for changesets under 100 files
7. THE Hook_Adapter SHALL log all hook executions to the Memory_Store for audit purposes

### Requirement 12: TUI Integration Interface

**User Story:** As a developer, I want to visualize the semantic graph in my terminal, so that I can explore code relationships interactively.

#### Acceptance Criteria

1. THE TUI_Adapter SHALL render the Semantic_Graph as an interactive node-and-edge visualization
2. THE TUI_Adapter SHALL support keyboard navigation between connected nodes
3. THE TUI_Adapter SHALL display node details including name, type, location, and attributes
4. THE TUI_Adapter SHALL highlight Construct_Mappings with visual indicators
5. THE TUI_Adapter SHALL display Uncertainty_Markers with warning colors
6. THE TUI_Adapter SHALL support filtering the graph view by language, file path, or node type
7. THE TUI_Adapter SHALL support exporting the current view as a text-based graph representation

### Requirement 13: CLI Command Interface

**User Story:** As a developer, I want to invoke refactor engine operations from the command line, so that I can script and automate analysis tasks.

#### Acceptance Criteria

1. THE CodexEngine SHALL accept an "analyze" command that triggers full Semantic_Graph construction
2. THE CodexEngine SHALL accept a "map" command that runs Cross_Language_Mapper on the current graph
3. THE CodexEngine SHALL accept a "check" command that detects and reports cross-language inconsistencies
4. THE CodexEngine SHALL accept a "plan" command that generates a Refactor_Plan for a specified operation
5. THE CodexEngine SHALL accept a "query" command that executes Memory_Store queries and displays results
6. THE CodexEngine SHALL accept a "diff" command that generates and displays diffs for a Refactor_Plan
7. WHEN a command completes, THE CodexEngine SHALL report execution time and affected entity counts

### Requirement 14: Incremental Index Updates

**User Story:** As a developer, I want the system to update the semantic graph incrementally, so that re-indexing is fast after small changes.

#### Acceptance Criteria

1. WHEN a file is modified, THE Refactor_Engine SHALL identify affected nodes in the Semantic_Graph
2. THE Refactor_Engine SHALL remove outdated nodes and edges for the modified file
3. THE Refactor_Engine SHALL re-parse the modified file and create new nodes and edges
4. THE Refactor_Engine SHALL update edges from other files that reference the modified file
5. THE Refactor_Engine SHALL recompute Construct_Mappings that involve affected nodes
6. THE Refactor_Engine SHALL complete incremental updates within 500 milliseconds for single-file changes
7. THE Refactor_Engine SHALL maintain Semantic_Graph consistency after incremental updates

### Requirement 15: Safe Refactor Preconditions

**User Story:** As a developer, I want the system to validate preconditions before refactoring, so that unsafe operations are prevented.

#### Acceptance Criteria

1. WHEN generating a Refactor_Plan, THE Refactor_Engine SHALL verify that all referenced nodes exist in the Semantic_Graph
2. THE Refactor_Engine SHALL verify that renamed entities do not conflict with existing names in the same scope
3. THE Refactor_Engine SHALL verify that type changes maintain compatibility with all usage sites
4. THE Refactor_Engine SHALL verify that moved entities remain accessible from all reference sites
5. IF a precondition fails, THEN THE Refactor_Engine SHALL mark the affected step with an Uncertainty_Marker and include the failure reason
6. THE Refactor_Engine SHALL compute a safety score for each Refactor_Plan based on precondition validation results
7. THE Refactor_Engine SHALL refuse to generate diffs for Refactor_Plans with safety scores below a configurable threshold

### Requirement 16: Parser Round-Trip Validation

**User Story:** As a developer, I want to ensure parsers correctly handle all language constructs, so that semantic analysis is accurate.

#### Acceptance Criteria

1. THE AST_Parser SHALL provide a pretty-printer for each supported language that formats Semantic_Nodes back to source code
2. WHEN a source file is parsed, THE AST_Parser SHALL generate a Semantic_Node representation
3. WHEN a Semantic_Node is pretty-printed, THE AST_Parser SHALL generate syntactically valid source code
4. WHEN pretty-printed source is re-parsed, THE AST_Parser SHALL produce an equivalent Semantic_Node representation
5. THE Refactor_Engine SHALL validate round-trip correctness for all parsed files during indexing
6. IF round-trip validation fails, THEN THE Refactor_Engine SHALL log the failure with file location and node details
7. THE Refactor_Engine SHALL report round-trip validation statistics including success rate and failure locations

### Requirement 17: Error Recovery and Partial Analysis

**User Story:** As a developer, I want the system to continue analysis despite parse errors, so that partial results are available for large codebases.

#### Acceptance Criteria

1. IF a file fails to parse, THEN THE Refactor_Engine SHALL log the error and continue indexing other files
2. WHEN a parse error occurs, THE Refactor_Engine SHALL create a placeholder node in the Semantic_Graph with error metadata
3. THE Refactor_Engine SHALL exclude nodes with parse errors from Cross_Language_Mapper analysis
4. THE Refactor_Engine SHALL include parse error counts in indexing statistics
5. THE Refactor_Engine SHALL provide a command to list all files with parse errors
6. WHEN a file with previous parse errors is modified, THE Refactor_Engine SHALL retry parsing during incremental update
7. THE Refactor_Engine SHALL maintain Semantic_Graph consistency when some files have parse errors

### Requirement 18: Configuration and Extensibility

**User Story:** As a developer, I want to configure analysis behavior and extend language support, so that the system adapts to project needs.

#### Acceptance Criteria

1. THE Refactor_Engine SHALL load configuration from a local file specifying enabled languages and analysis rules
2. THE Refactor_Engine SHALL support disabling specific languages through configuration
3. THE Refactor_Engine SHALL support custom Construct_Mapping rules defined in configuration files
4. THE Refactor_Engine SHALL support configurable confidence thresholds for Cross_Language_Mapper
5. THE Refactor_Engine SHALL support configurable safety score thresholds for Refactor_Plan generation
6. THE Refactor_Engine SHALL provide a plugin interface for adding new language parsers
7. WHEN configuration changes, THE Refactor_Engine SHALL reload settings without requiring process restart

### Requirement 19: Performance and Scalability

**User Story:** As a developer, I want the system to handle large monorepos efficiently, so that analysis completes in reasonable time.

#### Acceptance Criteria

1. THE Refactor_Engine SHALL index codebases with up to 1 million lines of code within 5 minutes
2. THE Refactor_Engine SHALL support Semantic_Graphs with up to 100,000 nodes without performance degradation
3. THE Refactor_Engine SHALL use parallel processing for independent file parsing operations
4. THE Refactor_Engine SHALL limit memory usage to 2 GB for codebases with up to 1 million lines
5. THE Memory_Store SHALL support concurrent read operations without blocking
6. THE Refactor_Engine SHALL provide progress reporting during long-running indexing operations
7. THE Refactor_Engine SHALL support cancellation of in-progress indexing operations

### Requirement 20: Debuggability and Observability

**User Story:** As a developer, I want detailed logging and introspection capabilities, so that I can debug analysis issues and understand system behavior.

#### Acceptance Criteria

1. THE Refactor_Engine SHALL log all indexing operations with timestamps and affected file paths
2. THE Refactor_Engine SHALL log all Construct_Mapping decisions with confidence scores and reasoning
3. THE Refactor_Engine SHALL log all Refactor_Plan generation steps with precondition evaluation results
4. THE Refactor_Engine SHALL provide a command to export the Semantic_Graph in a human-readable format
5. THE Refactor_Engine SHALL provide a command to export Construct_Mappings with detailed similarity metrics
6. THE Refactor_Engine SHALL support configurable log levels for different subsystems
7. THE Refactor_Engine SHALL include performance metrics in logs for all major operations
