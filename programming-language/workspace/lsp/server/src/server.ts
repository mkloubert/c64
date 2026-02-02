/*
Cobra64 - A concept for a modern Python-like compiler creating C64 binaries

Copyright (C) 2026 Marcel Joachim Kloubert <marcel@kloubert.dev>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
*/

import {
    createConnection,
    TextDocuments,
    ProposedFeatures,
    InitializeParams,
    InitializeResult,
    TextDocumentSyncKind,
    DidChangeConfigurationNotification,
    Diagnostic,
    DiagnosticSeverity as LspDiagnosticSeverity,
    Range,
    Position,
    TextDocumentPositionParams,
    Hover,
    Definition,
    Location,
    DocumentSymbolParams,
    DocumentSymbol,
    SymbolKind,
    CompletionItem,
    CompletionParams,
    SignatureHelp,
    SignatureHelpParams,
    ReferenceParams,
    RenameParams,
    WorkspaceEdit,
    TextEdit,
    PrepareRenameParams,
    DocumentHighlight,
    DocumentHighlightKind,
    DocumentHighlightParams,
    WorkspaceSymbolParams,
    WorkspaceSymbol,
    SemanticTokensParams,
    SemanticTokens,
    SemanticTokensRangeParams,
    CodeActionParams,
    CodeAction,
    FoldingRangeParams,
    FoldingRange,
    InlayHintParams,
    InlayHint,
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';
import { tokenize, Token } from './lexer';
import { parse, Program } from './parser';
import { analyze, AnalyzerResult } from './analyzer';
import {
    getHover,
    getDefinition,
    DocumentAnalysis,
    findReferences,
    getWordAtPosition,
} from './features';
import { TokenType, KEYWORDS, TYPE_KEYWORDS } from './lexer';
import { COBRA64_BUILTINS } from '../../shared/types';
import {
    getCompletions,
    resolveCompletionItem,
    getSignatureHelp,
} from './completion';
import {
    buildSemanticTokens,
    createSemanticTokensLegend,
} from './semantic-tokens';
import { getInlayHints } from './inlay-hints';
import { getCodeActions } from './code-actions';
import { getFoldingRanges } from './folding-ranges';
import {
    CompilerDiagnostic,
    DiagnosticSeverity,
    offsetToPosition,
    Symbol as Cobra64Symbol,
} from '../../shared/types';

// Create a connection for the server using IPC
const connection = createConnection(ProposedFeatures.all);

// Create a document manager for text documents
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

// Server configuration
interface Cobra64Settings {
    maxNumberOfProblems: number;
}

const defaultSettings: Cobra64Settings = { maxNumberOfProblems: 100 };
let globalSettings: Cobra64Settings = defaultSettings;

// Cache for document settings
const documentSettings: Map<string, Thenable<Cobra64Settings>> = new Map();

// Cache for parsed documents
interface DocumentCache {
    version: number;
    program: Program | null;
    symbols: Cobra64Symbol[];
    analyzerResult: AnalyzerResult | null;
    tokens: Token[];
}
const documentCache: Map<string, DocumentCache> = new Map();

// Debounce timers for validation
const validationTimers: Map<string, NodeJS.Timeout> = new Map();
const VALIDATION_DELAY_MS = 100;

// Has configuration capability
let hasConfigurationCapability = false;
let hasWorkspaceFolderCapability = false;

/**
 * Handle server initialization.
 */
connection.onInitialize((params: InitializeParams): InitializeResult => {
    const capabilities = params.capabilities;

    // Check client capabilities
    hasConfigurationCapability = !!(
        capabilities.workspace && !!capabilities.workspace.configuration
    );
    hasWorkspaceFolderCapability = !!(
        capabilities.workspace && !!capabilities.workspace.workspaceFolders
    );

    const result: InitializeResult = {
        capabilities: {
            // Full text sync - send entire document on change
            textDocumentSync: TextDocumentSyncKind.Incremental,

            // Phase 3: Hover and Go-to-Definition
            hoverProvider: true,
            definitionProvider: true,
            documentSymbolProvider: true,

            // Phase 4: Completion and Signature Help
            completionProvider: {
                resolveProvider: true,
                triggerCharacters: [':', '(', ','],
            },
            signatureHelpProvider: {
                triggerCharacters: ['(', ','],
                retriggerCharacters: [','],
            },

            // Phase 5: Extended Features
            referencesProvider: true,
            workspaceSymbolProvider: true,
            renameProvider: {
                prepareProvider: true,
            },
            documentHighlightProvider: true,

            // Semantic Tokens
            semanticTokensProvider: {
                legend: createSemanticTokensLegend(),
                full: true,
                range: true,
            },

            // Phase 6: Polish Features
            inlayHintProvider: true,
            codeActionProvider: {
                codeActionKinds: ['quickfix', 'refactor.rewrite'],
            },
            foldingRangeProvider: true,
        },
    };

    if (hasWorkspaceFolderCapability) {
        result.capabilities.workspace = {
            workspaceFolders: {
                supported: true,
            },
        };
    }

    return result;
});

/**
 * Handle post-initialization setup.
 */
connection.onInitialized(() => {
    if (hasConfigurationCapability) {
        // Register for configuration changes
        connection.client.register(
            DidChangeConfigurationNotification.type,
            undefined
        );
    }

    connection.console.log('Cobra64 Language Server initialized');
});

/**
 * Handle configuration changes.
 */
connection.onDidChangeConfiguration((change) => {
    if (hasConfigurationCapability) {
        // Clear cached settings
        documentSettings.clear();
    } else {
        globalSettings = (change.settings.cobra64 || defaultSettings) as Cobra64Settings;
    }

    // Revalidate all open documents
    documents.all().forEach(doc => scheduleValidation(doc));
});

/**
 * Get settings for a document.
 */
function getDocumentSettings(resource: string): Thenable<Cobra64Settings> {
    if (!hasConfigurationCapability) {
        return Promise.resolve(globalSettings);
    }

    let result = documentSettings.get(resource);
    if (!result) {
        result = connection.workspace.getConfiguration({
            scopeUri: resource,
            section: 'cobra64',
        });
        documentSettings.set(resource, result);
    }
    return result;
}

/**
 * Handle document open.
 */
documents.onDidOpen((event) => {
    connection.console.log(`Document opened: ${event.document.uri}`);
    scheduleValidation(event.document);
});

/**
 * Handle document changes with debouncing.
 */
documents.onDidChangeContent((change) => {
    scheduleValidation(change.document);
});

/**
 * Handle document close.
 */
documents.onDidClose((event) => {
    documentSettings.delete(event.document.uri);
    documentCache.delete(event.document.uri);

    // Cancel any pending validation
    const timer = validationTimers.get(event.document.uri);
    if (timer) {
        clearTimeout(timer);
        validationTimers.delete(event.document.uri);
    }

    // Clear diagnostics for closed document
    connection.sendDiagnostics({ uri: event.document.uri, diagnostics: [] });
});

/**
 * Schedule validation with debouncing.
 */
function scheduleValidation(textDocument: TextDocument): void {
    const uri = textDocument.uri;

    // Cancel any pending validation
    const existingTimer = validationTimers.get(uri);
    if (existingTimer) {
        clearTimeout(existingTimer);
    }

    // Schedule new validation
    const timer = setTimeout(() => {
        validationTimers.delete(uri);
        validateTextDocument(textDocument);
    }, VALIDATION_DELAY_MS);

    validationTimers.set(uri, timer);
}

/**
 * Validate a text document and send diagnostics.
 */
async function validateTextDocument(textDocument: TextDocument): Promise<void> {
    const settings = await getDocumentSettings(textDocument.uri);
    const text = textDocument.getText();

    connection.console.log(
        `Validating document: ${textDocument.uri} (${text.length} chars)`
    );

    const allDiagnostics: CompilerDiagnostic[] = [];

    // Step 1: Tokenize
    const lexerResult = tokenize(text);
    allDiagnostics.push(...lexerResult.diagnostics);

    // Step 2: Parse (even if lexer had errors, try to parse for more diagnostics)
    let program: Program | null = null;
    let analyzerResult: AnalyzerResult | null = null;

    if (lexerResult.tokens.length > 0) {
        const parserResult = parse(lexerResult.tokens);
        allDiagnostics.push(...parserResult.diagnostics);
        program = parserResult.program;

        // Step 3: Analyze (if parsing succeeded)
        if (program) {
            analyzerResult = analyze(program);
            allDiagnostics.push(...analyzerResult.diagnostics);
        }
    }

    // Update cache
    documentCache.set(textDocument.uri, {
        version: textDocument.version,
        program,
        symbols: analyzerResult?.symbols || [],
        analyzerResult,
        tokens: lexerResult.tokens,
    });

    // Convert to LSP diagnostics
    const lspDiagnostics = convertDiagnostics(
        allDiagnostics,
        text,
        settings.maxNumberOfProblems
    );

    // Send diagnostics to client
    connection.sendDiagnostics({
        uri: textDocument.uri,
        diagnostics: lspDiagnostics,
    });
}

/**
 * Convert compiler diagnostics to LSP diagnostics.
 */
function convertDiagnostics(
    diagnostics: CompilerDiagnostic[],
    source: string,
    maxProblems: number
): Diagnostic[] {
    const result: Diagnostic[] = [];

    for (let i = 0; i < diagnostics.length && i < maxProblems; i++) {
        const diag = diagnostics[i];
        const range = spanToRange(source, diag.span);

        result.push({
            severity: convertSeverity(diag.severity),
            range,
            message: diag.message,
            code: diag.code,
            source: 'cobra64',
        });
    }

    return result;
}

/**
 * Convert span (byte offsets) to LSP Range (line/character).
 */
function spanToRange(source: string, span: { start: number; end: number }): Range {
    const startPos = offsetToPosition(source, span.start);
    const endPos = offsetToPosition(source, span.end);

    return {
        start: Position.create(startPos.line, startPos.character),
        end: Position.create(endPos.line, endPos.character),
    };
}

/**
 * Convert diagnostic severity to LSP severity.
 */
function convertSeverity(severity: DiagnosticSeverity): LspDiagnosticSeverity {
    switch (severity) {
        case DiagnosticSeverity.Error:
            return LspDiagnosticSeverity.Error;
        case DiagnosticSeverity.Warning:
            return LspDiagnosticSeverity.Warning;
        case DiagnosticSeverity.Information:
            return LspDiagnosticSeverity.Information;
        case DiagnosticSeverity.Hint:
            return LspDiagnosticSeverity.Hint;
        default:
            return LspDiagnosticSeverity.Error;
    }
}

/**
 * Get document analysis for features.
 */
function getDocumentAnalysis(uri: string): DocumentAnalysis | null {
    const cache = documentCache.get(uri);
    if (!cache) return null;

    return {
        program: cache.program,
        analyzerResult: cache.analyzerResult,
        tokens: cache.tokens,
    };
}

// ============================================================================
// Phase 3: Hover Provider
// ============================================================================

connection.onHover((params: TextDocumentPositionParams): Hover | null => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return null;

    const analysis = getDocumentAnalysis(params.textDocument.uri);
    if (!analysis) return null;

    return getHover(document, params.position, analysis);
});

// ============================================================================
// Phase 3: Go-to-Definition Provider
// ============================================================================

connection.onDefinition((params: TextDocumentPositionParams): Definition | null => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return null;

    const analysis = getDocumentAnalysis(params.textDocument.uri);
    if (!analysis) return null;

    return getDefinition(document, params.position, analysis);
});

// ============================================================================
// Phase 3: Document Symbols Provider (for Outline view)
// ============================================================================

connection.onDocumentSymbol((params: DocumentSymbolParams): DocumentSymbol[] => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return [];

    const cache = documentCache.get(params.textDocument.uri);
    if (!cache || !cache.program) return [];

    const text = document.getText();
    const symbols: DocumentSymbol[] = [];

    for (const item of cache.program.items) {
        if (item.kind === 'FunctionDef') {
            const range = spanToRange(text, item.span);
            // Selection range is just the function name
            const nameStart = offsetToPosition(text, item.span.start);
            const nameEnd = { line: nameStart.line, character: nameStart.character + item.name.length + 4 }; // "def " + name

            const children: DocumentSymbol[] = [];

            // Add parameters as children
            for (const param of item.params) {
                const paramRange = spanToRange(text, param.span);
                children.push({
                    name: param.name,
                    kind: SymbolKind.Variable,
                    range: paramRange,
                    selectionRange: paramRange,
                    detail: param.type,
                });
            }

            symbols.push({
                name: item.name,
                kind: SymbolKind.Function,
                range,
                selectionRange: {
                    start: Position.create(nameStart.line, nameStart.character),
                    end: Position.create(nameEnd.line, nameEnd.character),
                },
                detail: item.returnType ? `-> ${item.returnType}` : undefined,
                children: children.length > 0 ? children : undefined,
            });
        } else if (item.kind === 'VarDecl') {
            const range = spanToRange(text, item.span);
            symbols.push({
                name: item.name,
                kind: SymbolKind.Variable,
                range,
                selectionRange: range,
                detail: item.type || undefined,
            });
        } else if (item.kind === 'ConstDecl') {
            const range = spanToRange(text, item.span);
            symbols.push({
                name: item.name,
                kind: SymbolKind.Constant,
                range,
                selectionRange: range,
                detail: item.type || undefined,
            });
        }
    }

    return symbols;
});

// ============================================================================
// Phase 4: Completion Provider
// ============================================================================

connection.onCompletion((params: CompletionParams): CompletionItem[] => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return [];

    const analysis = getDocumentAnalysis(params.textDocument.uri);
    if (!analysis) {
        // Return basic completions even without analysis
        return getCompletions(document, params.position, {
            program: null,
            analyzerResult: null,
            tokens: [],
        });
    }

    return getCompletions(document, params.position, analysis);
});

connection.onCompletionResolve((item: CompletionItem): CompletionItem => {
    return resolveCompletionItem(item);
});

// ============================================================================
// Phase 4: Signature Help Provider
// ============================================================================

connection.onSignatureHelp((params: SignatureHelpParams): SignatureHelp | null => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return null;

    const analysis = getDocumentAnalysis(params.textDocument.uri);
    if (!analysis) return null;

    return getSignatureHelp(document, params.position, analysis);
});

// ============================================================================
// Phase 5: Find All References
// ============================================================================

connection.onReferences((params: ReferenceParams): Location[] => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return [];

    const analysis = getDocumentAnalysis(params.textDocument.uri);
    if (!analysis) return [];

    return findReferences(
        document,
        params.position,
        analysis,
        params.context.includeDeclaration
    );
});

// ============================================================================
// Phase 5: Rename Symbol
// ============================================================================

connection.onPrepareRename((params: PrepareRenameParams): Range | null => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return null;

    const wordInfo = getWordAtPosition(document, params.position);
    if (!wordInfo) return null;

    const { word, range } = wordInfo;

    // Cannot rename keywords or types
    if (KEYWORDS.has(word) || TYPE_KEYWORDS.has(word) || word === 'true' || word === 'false') {
        return null;
    }

    // Cannot rename built-in functions
    const builtin = COBRA64_BUILTINS.find(b => b.name === word);
    if (builtin) {
        return null;
    }

    return range;
});

connection.onRenameRequest((params: RenameParams): WorkspaceEdit | null => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return null;

    const analysis = getDocumentAnalysis(params.textDocument.uri);
    if (!analysis) return null;

    const wordInfo = getWordAtPosition(document, params.position);
    if (!wordInfo) return null;

    const { word } = wordInfo;
    const newName = params.newName;

    // Validate new name
    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(newName)) {
        return null;
    }

    // Cannot rename to a keyword
    if (KEYWORDS.has(newName) || TYPE_KEYWORDS.has(newName) || newName === 'true' || newName === 'false') {
        return null;
    }

    // Find all references
    const references = findReferences(document, params.position, analysis, true);

    if (references.length === 0) {
        return null;
    }

    // Create workspace edit
    const changes: { [uri: string]: TextEdit[] } = {};
    changes[params.textDocument.uri] = references.map(ref => ({
        range: ref.range,
        newText: newName,
    }));

    return { changes };
});

// ============================================================================
// Phase 5: Document Highlighting
// ============================================================================

connection.onDocumentHighlight((params: DocumentHighlightParams): DocumentHighlight[] => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return [];

    const analysis = getDocumentAnalysis(params.textDocument.uri);
    if (!analysis) return [];

    const wordInfo = getWordAtPosition(document, params.position);
    if (!wordInfo) return [];

    const { word } = wordInfo;
    const text = document.getText();
    const highlights: DocumentHighlight[] = [];

    // Find definition span (for write highlight)
    let definitionSpan: { start: number; end: number } | null = null;

    if (analysis.analyzerResult) {
        const symbol = analysis.analyzerResult.symbols.find(s => s.name === word);
        if (symbol) {
            definitionSpan = symbol.definitionSpan;
        }
    }

    // Find all occurrences of the word in tokens
    for (const token of analysis.tokens) {
        if (token.type === TokenType.Identifier && token.value === word) {
            const range = spanToRange(text, token.span);

            // Determine if this is a write (definition/assignment) or read
            let kind: DocumentHighlightKind = DocumentHighlightKind.Read;

            // Check if this is the definition
            if (definitionSpan &&
                token.span.start === definitionSpan.start &&
                token.span.end === definitionSpan.end) {
                kind = DocumentHighlightKind.Write;
            }

            highlights.push({
                range,
                kind,
            });
        }
    }

    return highlights;
});

// ============================================================================
// Phase 5: Workspace Symbols
// ============================================================================

connection.onWorkspaceSymbol((params: WorkspaceSymbolParams): WorkspaceSymbol[] => {
    const query = params.query.toLowerCase();
    const result: WorkspaceSymbol[] = [];

    // Search through all cached documents
    for (const [uri, cache] of documentCache) {
        if (!cache.program) continue;

        const document = documents.get(uri);
        if (!document) continue;

        const text = document.getText();

        for (const item of cache.program.items) {
            if (item.kind === 'FunctionDef') {
                if (query === '' || item.name.toLowerCase().includes(query)) {
                    result.push({
                        name: item.name,
                        kind: SymbolKind.Function,
                        location: {
                            uri,
                            range: spanToRange(text, item.span),
                        },
                    });
                }
            } else if (item.kind === 'VarDecl') {
                if (query === '' || item.name.toLowerCase().includes(query)) {
                    result.push({
                        name: item.name,
                        kind: SymbolKind.Variable,
                        location: {
                            uri,
                            range: spanToRange(text, item.span),
                        },
                    });
                }
            } else if (item.kind === 'ConstDecl') {
                if (query === '' || item.name.toLowerCase().includes(query)) {
                    result.push({
                        name: item.name,
                        kind: SymbolKind.Constant,
                        location: {
                            uri,
                            range: spanToRange(text, item.span),
                        },
                    });
                }
            }
        }
    }

    return result;
});

// ============================================================================
// Phase 5: Semantic Tokens
// ============================================================================

connection.onRequest('textDocument/semanticTokens/full', (params: SemanticTokensParams): SemanticTokens => {
    const document = documents.get(params.textDocument.uri);
    if (!document) {
        return { data: [] };
    }

    const cache = documentCache.get(params.textDocument.uri);
    if (!cache) {
        return { data: [] };
    }

    const data = buildSemanticTokens(document, cache.tokens, cache.analyzerResult);
    return { data };
});

connection.onRequest('textDocument/semanticTokens/range', (params: SemanticTokensRangeParams): SemanticTokens => {
    const document = documents.get(params.textDocument.uri);
    if (!document) {
        return { data: [] };
    }

    const cache = documentCache.get(params.textDocument.uri);
    if (!cache) {
        return { data: [] };
    }

    // For now, return full tokens - range filtering can be optimized later
    const data = buildSemanticTokens(document, cache.tokens, cache.analyzerResult);
    return { data };
});

// ============================================================================
// Phase 6: Inlay Hints
// ============================================================================

connection.onRequest('textDocument/inlayHint', (params: InlayHintParams): InlayHint[] => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return [];

    const cache = documentCache.get(params.textDocument.uri);
    if (!cache) return [];

    return getInlayHints(document, params.range, cache.program, cache.analyzerResult);
});

// ============================================================================
// Phase 6: Code Actions
// ============================================================================

connection.onCodeAction((params: CodeActionParams): CodeAction[] => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return [];

    const cache = documentCache.get(params.textDocument.uri);
    if (!cache) return [];

    return getCodeActions(
        document,
        params.range,
        params.context.diagnostics,
        cache.program,
        cache.analyzerResult
    );
});

// ============================================================================
// Phase 6: Folding Ranges
// ============================================================================

connection.onFoldingRanges((params: FoldingRangeParams): FoldingRange[] => {
    const document = documents.get(params.textDocument.uri);
    if (!document) return [];

    const cache = documentCache.get(params.textDocument.uri);
    if (!cache) return [];

    return getFoldingRanges(document, cache.program, cache.tokens);
});

/**
 * Get cached document info.
 */
export function getDocumentCacheExport(uri: string): DocumentCache | undefined {
    return documentCache.get(uri);
}

// Listen for document events
documents.listen(connection);

// Start listening on the connection
connection.listen();
