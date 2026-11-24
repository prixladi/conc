// Sidebar tree: projects -> services, with inline actions driven by contextValue.
import * as vscode from 'vscode';
import { ProjectInfo, ServiceInfo, ServiceStatus } from './client';
import { describeService, formatTimestamp, runningCount } from './format';
import { Store } from './store';

export type Node =
  | { kind: 'project'; project: ProjectInfo }
  | { kind: 'service'; project: string; service: ServiceInfo };

const STATUS_ICON: Record<ServiceStatus, vscode.ThemeIcon> = {
  RUNNING: new vscode.ThemeIcon('pass-filled', new vscode.ThemeColor('testing.iconPassed')),
  EXITED: new vscode.ThemeIcon('error', new vscode.ThemeColor('testing.iconFailed')),
  STOPPED: new vscode.ThemeIcon('circle-slash', new vscode.ThemeColor('disabledForeground')),
  IDLE: new vscode.ThemeIcon('circle-outline', new vscode.ThemeColor('disabledForeground')),
};

export class ProjectsTree implements vscode.TreeDataProvider<Node>, vscode.Disposable {
  private readonly emitter = new vscode.EventEmitter<Node | undefined>();
  readonly onDidChangeTreeData = this.emitter.event;
  readonly view: vscode.TreeView<Node>;
  private ticker: NodeJS.Timeout | undefined;
  private disposables: vscode.Disposable[] = [];

  constructor(private readonly store: Store) {
    this.view = vscode.window.createTreeView('conc.projects', {
      treeDataProvider: this,
      showCollapseAll: true,
    });
    this.disposables.push(
      this.view,
      store.onDidChange(() => {
        this.updateBadge();
        this.emitter.fire(undefined);
      }),
    );
    // Keep uptimes fresh while the view is visible and something is running.
    this.ticker = setInterval(() => {
      if (this.view.visible && store.current.projects.some((p) => runningCount(p) > 0)) {
        this.emitter.fire(undefined);
      }
    }, 1000);
  }

  getChildren(node?: Node): Node[] {
    const projects = this.store.current.projects;
    if (!node) {
      return projects.map((project) => ({ kind: 'project', project }));
    }
    if (node.kind === 'project') {
      return node.project.services.map((service) => ({
        kind: 'service',
        project: node.project.name,
        service,
      }));
    }
    return [];
  }

  getTreeItem(node: Node): vscode.TreeItem {
    return node.kind === 'project' ? projectItem(node.project) : serviceItem(node.project, node.service);
  }

  private updateBadge(): void {
    const projects = this.store.current.projects;
    const running = projects.reduce((n, p) => n + runningCount(p), 0);
    this.view.badge = running ? { value: running, tooltip: `${running} running services` } : undefined;
  }

  dispose(): void {
    clearInterval(this.ticker);
    this.emitter.dispose();
    this.disposables.forEach((d) => d.dispose());
  }
}

function projectItem(p: ProjectInfo): vscode.TreeItem {
  const running = runningCount(p);
  const total = p.services.length;
  const item = new vscode.TreeItem(p.name, vscode.TreeItemCollapsibleState.Expanded);
  item.id = `p:${p.name}`;
  item.description = `${running}/${total} running`;

  const anyExited = p.services.some((s) => s.status === 'EXITED');
  if (running === total && total > 0) {
    item.iconPath = new vscode.ThemeIcon('folder-active', new vscode.ThemeColor('testing.iconPassed'));
    item.contextValue = 'project-allRunning';
  } else if (running > 0) {
    item.iconPath = new vscode.ThemeIcon(
      'folder-active',
      new vscode.ThemeColor(anyExited ? 'testing.iconFailed' : 'testing.iconQueued'),
    );
    item.contextValue = 'project-anyRunning';
  } else {
    item.iconPath = new vscode.ThemeIcon('folder');
    item.contextValue = 'project-noneRunning';
  }
  item.tooltip = `${p.name}\n${running} of ${total} services running`;
  return item;
}

function serviceItem(project: string, s: ServiceInfo): vscode.TreeItem {
  const item = new vscode.TreeItem(s.name, vscode.TreeItemCollapsibleState.None);
  item.id = `s:${project}/${s.name}`;
  item.iconPath = STATUS_ICON[s.status];
  item.description = describeService(s) + (s.status === 'RUNNING' && s.pid ? ` · pid ${s.pid}` : '');
  item.contextValue = `service-${s.status}`;

  const md = new vscode.MarkdownString(undefined, true);
  md.appendMarkdown(`**${project} / ${s.name}**\n\n`);
  md.appendMarkdown(`| | |\n|---|---|\n`);
  md.appendMarkdown(`| Status | ${s.status} |\n`);
  md.appendMarkdown(`| PID | ${s.pid || '-'} |\n`);
  md.appendMarkdown(`| Started | ${formatTimestamp(s.startTime)} |\n`);
  md.appendMarkdown(`| Stopped | ${formatTimestamp(s.stopTime)} |\n`);
  md.appendMarkdown(`| Log | \`${s.logfilePath || '-'}\` |\n`);
  item.tooltip = md;

  item.command = {
    command: 'conc.openDashboard',
    title: 'Show in Dashboard',
    arguments: [{ project, service: s.name }],
  };
  return item;
}
