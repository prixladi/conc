import * as vscode from 'vscode';
import { ServiceStatus } from './client';
import { registerCommands } from './commands';
import { runningCount } from './format';
import { Store } from './store';
import { ProjectsTree } from './tree';

export function activate(context: vscode.ExtensionContext): void {
  const store = new Store();
  const tree = new ProjectsTree(store);
  context.subscriptions.push(store, tree, createStatusBar(store), watchForExits(store));
  registerCommands(context, store);
  store.start();
}

export function deactivate(): void {}

function createStatusBar(store: Store): vscode.Disposable {
  const item = vscode.window.createStatusBarItem('conc.status', vscode.StatusBarAlignment.Left, 50);
  item.name = 'Conc';
  item.command = 'conc.openDashboard';

  const update = () => {
    const { state, projects } = store.current;
    if (state === 'offline') {
      item.text = '$(debug-disconnect) conc';
      item.tooltip = `Conc daemon is not reachable at ${store.config.socketPath}`;
      item.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
    } else {
      const running = projects.reduce((n, p) => n + runningCount(p), 0);
      const total = projects.reduce((n, p) => n + p.services.length, 0);
      const exited = projects.reduce((n, p) => n + p.services.filter((s) => s.status === 'EXITED').length, 0);
      item.text = `$(server-process) ${running}/${total}` + (exited ? ` $(error) ${exited}` : '');
      item.tooltip = `Conc: ${running} of ${total} services running${exited ? `, ${exited} exited` : ''}\nClick to open the dashboard`;
      item.backgroundColor = undefined;
    }
    item.show();
  };

  const sub = store.onDidChange(update);
  return vscode.Disposable.from(item, sub);
}

/** Notifies when a service that was running exits on its own. */
function watchForExits(store: Store): vscode.Disposable {
  let previous = new Map<string, ServiceStatus>();
  return store.onDidChange(({ state, projects }) => {
    if (state !== 'online') {
      return;
    }
    const next = new Map<string, ServiceStatus>();
    for (const p of projects) {
      for (const s of p.services) {
        const key = `${p.name}/${s.name}`;
        next.set(key, s.status);
        const notify = vscode.workspace.getConfiguration('conc').get<boolean>('notifyOnExit', true);
        if (notify && previous.get(key) === 'RUNNING' && s.status === 'EXITED') {
          const ref = { project: p.name, service: s.name };
          void vscode.window
            .showWarningMessage(`Conc: service ${key} exited.`, 'Show Logs', 'Restart')
            .then((choice) => {
              if (choice === 'Show Logs') {
                void vscode.commands.executeCommand('conc.openDashboard', ref);
              } else if (choice === 'Restart') {
                void vscode.commands.executeCommand('conc.startService', ref);
              }
            });
        }
      }
    }
    previous = next;
  });
}
