// Polls the daemon and broadcasts state changes to the tree, dashboard and status bar.
import * as vscode from 'vscode';
import { DaemonClient, ProjectInfo } from './client';
import { EffectiveConfig, getConfig } from './config';

export type DaemonState = 'unknown' | 'online' | 'offline';

export interface Snapshot {
  state: DaemonState;
  projects: ProjectInfo[];
  error?: string;
}

export class Store implements vscode.Disposable {
  private readonly emitter = new vscode.EventEmitter<Snapshot>();
  readonly onDidChange = this.emitter.event;

  private snapshot: Snapshot = { state: 'unknown', projects: [] };
  private lastSerialized = '';
  private timer: NodeJS.Timeout | undefined;
  private inFlight = false;
  private disposables: vscode.Disposable[] = [];

  config: EffectiveConfig;
  client: DaemonClient;

  constructor() {
    this.config = getConfig();
    this.client = this.createClient();

    this.disposables.push(
      vscode.workspace.onDidChangeConfiguration((e) => {
        if (e.affectsConfiguration('conc')) {
          this.reloadConfig();
        }
      }),
      vscode.window.onDidChangeWindowState(() => this.schedule()),
    );
  }

  get current(): Snapshot {
    return this.snapshot;
  }

  start(): void {
    void this.refresh();
  }

  reloadConfig(): void {
    this.config = getConfig();
    this.client = this.createClient();
    void this.refresh();
  }

  /** Polls immediately and re-arms the timer. */
  async refresh(): Promise<void> {
    if (this.inFlight) {
      return;
    }
    this.inFlight = true;
    clearTimeout(this.timer);
    try {
      const projects = await this.client.getProjectsInfo();
      this.publish({ state: 'online', projects });
    } catch (e) {
      this.publish({ state: 'offline', projects: [], error: (e as Error).message });
    } finally {
      this.inFlight = false;
      this.schedule();
    }
  }

  private schedule(): void {
    clearTimeout(this.timer);
    // Back off while VS Code is not focused, the user can't see it anyway.
    const focused = vscode.window.state.focused;
    const interval = focused ? this.config.refreshInterval : Math.max(5000, this.config.refreshInterval);
    this.timer = setTimeout(() => void this.refresh(), interval);
  }

  private publish(next: Snapshot): void {
    const serialized = JSON.stringify(next);
    if (serialized === this.lastSerialized) {
      return;
    }
    this.lastSerialized = serialized;
    this.snapshot = next;
    void vscode.commands.executeCommand(
      'setContext',
      'conc.daemonState',
      next.state === 'online' && next.projects.length > 0 ? 'hasProjects' : next.state,
    );
    this.emitter.fire(next);
  }

  private createClient(): DaemonClient {
    return new DaemonClient(this.config.socketPath, () =>
      this.config.useCallerEnv ? JSON.stringify(process.env) : '{}',
    );
  }

  dispose(): void {
    clearTimeout(this.timer);
    this.emitter.dispose();
    this.disposables.forEach((d) => d.dispose());
  }
}
