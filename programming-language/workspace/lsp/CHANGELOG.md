# Changelog

All notable changes to the Cobra64 Language Support extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-02

### Added

#### Diagnostics

- Real-time error checking for lexer, parser, and semantic errors
- Support for 50+ error codes with descriptive messages
- Errors displayed inline and in Problems panel

#### IntelliSense

- Auto-completion for keywords, types, and built-in functions
- User-defined function and variable completion
- Context-aware type suggestions after `:`
- Snippet support for function definitions and control structures

#### Navigation

- Go to Definition for variables, constants, and functions
- Find All References to locate symbol usages
- Document Symbols for outline view
- Workspace Symbols for cross-file search

#### Hover Information

- Type information for all symbols
- Documentation for built-in functions
- Keyword descriptions with code examples

#### Code Editing

- Signature Help for function calls with parameter highlighting
- Rename Symbol support for all user-defined symbols
- Document Highlighting for symbol occurrences
- Semantic Tokens for enhanced syntax highlighting
- Code Folding for functions, loops, and comment blocks
- Inlay Hints for type inference and parameter names

#### Quick Fixes

- Typo suggestions using Levenshtein distance
- Variable/function stub creation
- Type annotation suggestions

#### Syntax Highlighting

- Full TextMate grammar for Cobra64
- Support for hex ($FF) and binary (%1010) literals
- Escape sequence highlighting in strings
- Comment highlighting

### Technical

- Client/Server architecture using LSP
- TypeScript implementation
- Debounced validation for performance
- Document caching for fast feature lookups
- 64 unit tests covering core functionality
