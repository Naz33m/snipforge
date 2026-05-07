import * as vscode from "vscode";

/**
 * SnipForge VS Code Extension
 *
 * Provides snippet management directly within VS Code:
 * - Search snippets via command palette
 * - Save selected code as a snippet
 * - Browse and insert snippets
 *
 * TODO: Connect to the SnipForge core engine (via CLI or SDK).
 */

export function activate(context: vscode.ExtensionContext) {
  console.log("SnipForge extension activated");

  const searchCommand = vscode.commands.registerCommand(
    "snipforge.search",
    async () => {
      const query = await vscode.window.showInputBox({
        prompt: "Search snippets",
        placeHolder: "e.g., validate email python",
      });

      if (query) {
        vscode.window.showInformationMessage(
          `SnipForge: Searching for "${query}"...`,
        );
        // TODO: Invoke snipforge-core search and display results
      }
    },
  );

  const saveCommand = vscode.commands.registerCommand(
    "snipforge.save",
    async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showWarningMessage("No active editor");
        return;
      }

      const selection = editor.selection;
      const selectedText = editor.document.getText(selection);

      if (!selectedText) {
        vscode.window.showWarningMessage("No text selected");
        return;
      }

      const title = await vscode.window.showInputBox({
        prompt: "Snippet title",
        placeHolder: "My awesome snippet",
      });

      if (title) {
        const language = editor.document.languageId;
        vscode.window.showInformationMessage(
          `SnipForge: Saved "${title}" (${language})`,
        );
        // TODO: Invoke snipforge-core to save the snippet
      }
    },
  );

  const listCommand = vscode.commands.registerCommand(
    "snipforge.list",
    async () => {
      vscode.window.showInformationMessage(
        "SnipForge: Browse snippets (coming soon)",
      );
      // TODO: Show a webview or quick pick with all snippets
    },
  );

  const insertCommand = vscode.commands.registerCommand(
    "snipforge.insert",
    async () => {
      vscode.window.showInformationMessage(
        "SnipForge: Insert snippet (coming soon)",
      );
      // TODO: Show a quick pick to select and insert a snippet
    },
  );

  context.subscriptions.push(
    searchCommand,
    saveCommand,
    listCommand,
    insertCommand,
  );
}

export function deactivate() {
  console.log("SnipForge extension deactivated");
}
