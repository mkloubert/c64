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

import * as path from 'path';
import { ExtensionContext, workspace } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

/**
 * Activates the Cobra64 language extension.
 */
export function activate(context: ExtensionContext): void {
    // Path to the server module
    const serverModule = context.asAbsolutePath(
        path.join('out', 'server', 'src', 'server.js')
    );

    // Server options - run the server as a Node.js module
    const serverOptions: ServerOptions = {
        run: {
            module: serverModule,
            transport: TransportKind.ipc,
        },
        debug: {
            module: serverModule,
            transport: TransportKind.ipc,
            options: {
                execArgv: ['--nolazy', '--inspect=6009'],
            },
        },
    };

    // Client options - register for Cobra64 documents
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'cobra64' }],
        synchronize: {
            // Notify server about file changes to .cb64 files
            fileEvents: workspace.createFileSystemWatcher('**/*.cb64'),
        },
        outputChannelName: 'Cobra64 Language Server',
    };

    // Create and start the language client
    client = new LanguageClient(
        'cobra64',
        'Cobra64 Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client (and server)
    client.start();

    console.log('Cobra64 Language Extension activated');
}

/**
 * Deactivates the extension.
 */
export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
