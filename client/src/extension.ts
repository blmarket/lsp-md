/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import {
  ExtensionContext,
  window,
} from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export async function activate(context: ExtensionContext) {
  const traceOutputChannel = window.createOutputChannel("Lsp-md Language Server trace");
  const command = process.env.SERVER_PATH || "lsp-md";
  const run: Executable = {
    command,
    options: {
      env: {
        ...process.env,
        // eslint-disable-next-line @typescript-eslint/naming-convention
        RUST_LOG: "debug",
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "markdown" }],
    middleware: {
      executeCommand: async (command, args, next) => {
        const resp = await next(command, args);
        switch (command) {
          case "lsp_md/searchSimilar":
            client.info("search similar result:", JSON.stringify(resp));
            break;
          case "lsp_md/keywords":
            client.info("keywords result:", JSON.stringify(resp));
            break;
          default:
            client.info("unknown command:", command);
            break;
        }
        return resp;
      },
    },
    traceOutputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient("lsp-md", "lsp-md language server", serverOptions, clientOptions);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
