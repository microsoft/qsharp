import { log } from "qsharp-lang";
import * as vscode from "vscode";

export class CircuitEditorProvider implements vscode.CustomTextEditorProvider {
  private static readonly viewType = "qsharp-webview.circuit";

  public static register(context: vscode.ExtensionContext): vscode.Disposable {
    log.info("Registering CircuitEditorProvider");
    const provider = new CircuitEditorProvider(context);
    const providerRegistration = vscode.window.registerCustomEditorProvider(
      CircuitEditorProvider.viewType,
      provider,
    );
    return providerRegistration;
  }

  constructor(private readonly context: vscode.ExtensionContext) {
    log.info("Constructing CircuitEditorProvider");
  }

  public async resolveCustomTextEditor(
    document: vscode.TextDocument,
    webviewPanel: vscode.WebviewPanel,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _token: vscode.CancellationToken,
  ): Promise<void> {
    log.info("Resolving CircuitEditorProvider");
    console.log("Resolving CircuitEditorProvider");

    // Setup initial content for the webview
    webviewPanel.webview.options = {
      enableScripts: true,
    };
    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

    webviewPanel.webview.onDidReceiveMessage((e) => {
      switch (e.command) {
        case "alert":
          vscode.window.showErrorMessage(e.text);
          return;
        case "update":
          this.updateTextDocument(document, e.text);
          return;
        case "read":
          updateWebview();
          return;
      }
    });

    const updateWebview = () => {
      const circuit = this.getDocumentAsJson(document);
      const filename = document.fileName.split(/\\|\//).pop()?.split(".")[0];

      const props = {
        title: `${filename} Circuit`,
        targetProfile: "",
        simulated: false,
        calculating: false,
        circuit,
      };

      const message = {
        command: "circuit",
        props,
      };
      webviewPanel.webview.postMessage(message);
    };

    const changeDocumentSubscription = vscode.workspace.onDidChangeTextDocument(
      (e) => {
        if (e.document.uri.toString() === document.uri.toString()) {
          updateWebview();
        }
      },
    );

    // Make sure we get rid of the listener when our editor is closed.
    webviewPanel.onDidDispose(() => {
      changeDocumentSubscription.dispose();
    });

    webviewPanel.webview.onDidReceiveMessage((e) => {
      if (e.type === "add") {
        const edit = new vscode.WorkspaceEdit();
        edit.insert(
          document.uri,
          new vscode.Position(0, 0),
          "Hello from circuitEditor.ts!\n",
        );
        vscode.workspace.applyEdit(edit);
      }
    });

    updateWebview();
  }

  private _getHtmlForWebview(webview: vscode.Webview): string {
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(
        this.context.extensionUri,
        "src",
        "webview",
        "editor.js",
      ),
    );
    const styleUri = webview.asWebviewUri(
      vscode.Uri.joinPath(
        this.context.extensionUri,
        "src",
        "webview",
        "style.css",
      ),
    );
    return `
      <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <title>Q#</title>
          <link rel="stylesheet" href="${styleUri}" />
          <script src="${scriptUri}"></script>
        </head>
        <body>
        </body>
      </html>`;
  }

  private getHtmlForWebview(webview: vscode.Webview): string {
    const extensionUri = this.context.extensionUri;

    function getUri(pathList: string[]) {
      return webview.asWebviewUri(
        vscode.Uri.joinPath(extensionUri, ...pathList),
      );
    }

    const katexCss = getUri(["out", "katex", "katex.min.css"]);
    const githubCss = getUri(["out", "katex", "github-markdown-dark.css"]);
    const webviewCss = getUri(["out", "webview", "webview.css"]);
    const scriptUri = getUri(["out", "webview", "editor.js"]);
    const resourcesUri = getUri(["resources"]);
    return `
      <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <title>Q#</title>
          <link rel="stylesheet" href="${githubCss}" />
          <link rel="stylesheet" href="${katexCss}" />
          <link rel="stylesheet" href="${webviewCss}" />
          <script src="${scriptUri}"></script>
          <script>
            window.resourcesUri = "${resourcesUri.toString()}";
          </script>
        </head>
        <body>
        </body>
      </html>`;
  }

  /**
   * Try to get a current document as json text.
   */
  private getDocumentAsJson(document: vscode.TextDocument): any {
    const text = document.getText();
    if (text.trim().length === 0) {
      return {};
    }

    try {
      return JSON.parse(text);
    } catch {
      throw new Error(
        "Could not get document as json. Content is not valid json",
      );
    }
  }

  /**
   * Write out the json to a given document.
   */
  private updateTextDocument(document: vscode.TextDocument, circuit: string) {
    // Short-circuit if there are no changes to be made.
    if (circuit == document.getText()) {
      return;
    }

    const edit = new vscode.WorkspaceEdit();

    // Just replace the entire document every time for this example extension.
    // A more complete extension should compute minimal edits instead.
    edit.replace(
      document.uri,
      new vscode.Range(0, 0, document.lineCount, 0),
      circuit,
    );

    return vscode.workspace.applyEdit(edit);
  }
}
