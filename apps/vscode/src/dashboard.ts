// Webview dashboard: project cards with per-service health and a live log viewer.
import * as crypto from 'crypto';
import * as fs from 'fs';
import * as vscode from 'vscode';
import { Store } from './store';

export interface ServiceRef {
  project: string;
  service: string;
}

// Commands the webview is allowed to trigger.
const ALLOWED_COMMANDS = new Set([
  'conc.refresh',
  'conc.upsertProject',
  'conc.startProject',
  'conc.stopProject',
  'conc.restartProject',
  'conc.restartProjectHot',
  'conc.restartProjectCold',
  'conc.clearProjectLogs',
  'conc.showProjectSettings',
  'conc.removeProject',
  'conc.startService',
  'conc.stopService',
  'conc.restartService',
  'conc.openServiceLog',
  'conc.tailServiceLog',
  'conc.clearServiceLogs',
]);

const MAX_LOG_READ_BYTES = 512 * 1024;

type InboundMessage =
  | { type: 'ready' }
  | { type: 'command'; command: string; project: string; service?: string }
  | { type: 'select'; project: string; service: string }
  | { type: 'unselect' };

export class Dashboard implements vscode.Disposable {
  private static instance: Dashboard | undefined;

  private readonly disposables: vscode.Disposable[] = [];
  private selected: ServiceRef | undefined;
  private watchedFile: string | undefined;
  private lastLog = '';

  static show(context: vscode.ExtensionContext, store: Store, select?: ServiceRef): void {
    if (Dashboard.instance) {
      Dashboard.instance.panel.reveal(undefined, true);
    } else {
      Dashboard.instance = new Dashboard(context, store);
    }
    if (select) {
      Dashboard.instance.select(select);
    }
  }

  private constructor(
    context: vscode.ExtensionContext,
    private readonly store: Store,
    private readonly panel = vscode.window.createWebviewPanel(
      'conc.dashboard',
      'Conc Dashboard',
      vscode.ViewColumn.Active,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'media')],
      },
    ),
  ) {
    panel.iconPath = vscode.Uri.joinPath(context.extensionUri, 'media', 'conc.svg');
    panel.webview.html = this.html(context);

    this.disposables.push(
      panel.onDidDispose(() => this.dispose()),
      panel.webview.onDidReceiveMessage((m: InboundMessage) => this.onMessage(m)),
      store.onDidChange(() => {
        this.postState();
        this.ensureLogWatch();
      }),
      vscode.workspace.onDidChangeConfiguration((e) => {
        if (e.affectsConfiguration('conc')) {
          this.postState();
          this.lastLog = '';
          this.pushLog();
        }
      }),
    );
  }

  private onMessage(m: InboundMessage): void {
    switch (m.type) {
      case 'ready':
        this.postState();
        this.lastLog = '';
        this.pushLog();
        break;
      case 'command':
        if (ALLOWED_COMMANDS.has(m.command)) {
          void vscode.commands.executeCommand(m.command, { project: m.project, service: m.service });
        }
        break;
      case 'select':
        this.select({ project: m.project, service: m.service });
        break;
      case 'unselect':
        this.select(undefined);
        break;
    }
  }

  private select(ref: ServiceRef | undefined): void {
    this.selected = ref;
    this.lastLog = '';
    void this.panel.webview.postMessage({ type: 'selected', selected: ref ?? null });
    this.ensureLogWatch();
    this.pushLog();
  }

  private postState(): void {
    const { current, config } = this.store;
    void this.panel.webview.postMessage({
      type: 'state',
      snapshot: current,
      socketPath: config.socketPath,
      restartBehavior: config.restartBehavior,
      selected: this.selected ?? null,
    });
  }

  private currentLogPath(): string | undefined {
    if (!this.selected) {
      return undefined;
    }
    const project = this.store.current.projects.find((p) => p.name === this.selected!.project);
    return project?.services.find((s) => s.name === this.selected!.service)?.logfilePath || undefined;
  }

  /** Watches the selected service's log file, swapping watchers when the selection changes. */
  private ensureLogWatch(): void {
    const file = this.currentLogPath();
    if (file === this.watchedFile) {
      return;
    }
    if (this.watchedFile) {
      fs.unwatchFile(this.watchedFile);
    }
    this.watchedFile = file;
    if (file) {
      fs.watchFile(file, { interval: 500 }, () => this.pushLog());
    }
  }

  private pushLog(): void {
    if (!this.selected) {
      return;
    }
    const file = this.currentLogPath();
    let text = '';
    let truncated = false;
    let error: string | undefined;
    if (!file) {
      error = 'This service has no log file.';
    } else {
      try {
        ({ text, truncated } = readTail(file, this.store.config.logLines));
      } catch (e) {
        const code = (e as NodeJS.ErrnoException).code;
        error = code === 'ENOENT' ? 'No log output yet.' : `Unable to read log: ${(e as Error).message}`;
      }
    }
    const key = `${file}\0${text}\0${error ?? ''}`;
    if (key === this.lastLog) {
      return;
    }
    this.lastLog = key;
    void this.panel.webview.postMessage({
      type: 'log',
      ref: this.selected,
      file: file ?? null,
      text,
      truncated,
      error: error ?? null,
    });
  }

  private html(context: vscode.ExtensionContext): string {
    const webview = this.panel.webview;
    const nonce = crypto.randomBytes(16).toString('base64');
    const media = (f: string) => webview.asWebviewUri(vscode.Uri.joinPath(context.extensionUri, 'media', f));
    return /* html */ `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource}; img-src ${webview.cspSource} data:; script-src 'nonce-${nonce}';">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<link rel="stylesheet" href="${media('dashboard.css')}">
<title>Conc Dashboard</title>
</head>
<body>
<div id="app"></div>
<script nonce="${nonce}" src="${media('dashboard.js')}"></script>
</body>
</html>`;
  }

  dispose(): void {
    Dashboard.instance = undefined;
    if (this.watchedFile) {
      fs.unwatchFile(this.watchedFile);
    }
    this.disposables.forEach((d) => d.dispose());
    this.panel.dispose();
  }
}

function readTail(file: string, lines: number): { text: string; truncated: boolean } {
  const fd = fs.openSync(file, 'r');
  try {
    const size = fs.fstatSync(fd).size;
    const length = Math.min(size, MAX_LOG_READ_BYTES);
    const buffer = Buffer.alloc(length);
    fs.readSync(fd, buffer, 0, length, size - length);
    let all = buffer.toString('utf8').split('\n');
    let truncated = size > length;
    if (truncated) {
      all = all.slice(1); // first line is likely partial
    }
    if (all.length > lines) {
      all = all.slice(-lines);
      truncated = true;
    }
    return { text: all.join('\n'), truncated };
  } finally {
    fs.closeSync(fd);
  }
}
