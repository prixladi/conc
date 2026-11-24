import * as path from 'path';
import * as vscode from 'vscode';
import { RestartType } from './client';
import { Dashboard, ServiceRef } from './dashboard';
import { loadProjectSettings } from './projectSettings';
import { Store } from './store';
import { Node } from './tree';

/** Anything a command can be invoked with: a tree node, a dashboard ref, or nothing (palette). */
type Target = Node | { project: string; service?: string } | undefined;

function projectOf(t: Target): string | undefined {
  if (!t) {
    return undefined;
  }
  if ('kind' in t) {
    return t.kind === 'project' ? t.project.name : t.project;
  }
  return t.project || undefined;
}

function serviceOf(t: Target): ServiceRef | undefined {
  if (!t) {
    return undefined;
  }
  if ('kind' in t) {
    return t.kind === 'service' ? { project: t.project, service: t.service.name } : undefined;
  }
  return t.project && t.service ? { project: t.project, service: t.service } : undefined;
}

export function registerCommands(context: vscode.ExtensionContext, store: Store): void {
  const pickProject = async (t: Target): Promise<string | undefined> => {
    const fromTarget = projectOf(t);
    if (fromTarget) {
      return fromTarget;
    }
    const names = store.current.projects.map((p) => p.name);
    if (!names.length) {
      void vscode.window.showInformationMessage('Conc: no projects registered.');
      return undefined;
    }
    return vscode.window.showQuickPick(names, { placeHolder: 'Select a project' });
  };

  const pickService = async (t: Target): Promise<ServiceRef | undefined> => {
    const fromTarget = serviceOf(t);
    if (fromTarget) {
      return fromTarget;
    }
    const items = store.current.projects.flatMap((p) =>
      p.services.map((s) => ({
        label: `${p.name} / ${s.name}`,
        description: s.status.toLowerCase(),
        ref: { project: p.name, service: s.name },
      })),
    );
    if (!items.length) {
      void vscode.window.showInformationMessage('Conc: no services registered.');
      return undefined;
    }
    return (await vscode.window.showQuickPick(items, { placeHolder: 'Select a service' }))?.ref;
  };

  const findService = (ref: ServiceRef) =>
    store.current.projects.find((p) => p.name === ref.project)?.services.find((s) => s.name === ref.service);

  /** Runs a daemon call with a progress indicator, reports errors, then refreshes state. */
  const run = async <T>(title: string, fn: () => Promise<T>): Promise<T | undefined> => {
    try {
      return await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Window, title: `Conc: ${title}` },
        fn,
      );
    } catch (e) {
      void vscode.window.showErrorMessage(`Conc: ${title} failed. ${(e as Error).message}`);
      return undefined;
    } finally {
      void store.refresh();
    }
  };

  const projectCmd =
    (title: string, fn: (project: string) => Promise<unknown>) =>
    async (t: Target) => {
      const project = await pickProject(t);
      if (project) {
        await run(`${title} ${project}`, () => fn(project));
      }
    };

  const serviceCmd =
    (title: string, fn: (ref: ServiceRef) => Promise<unknown>) =>
    async (t: Target) => {
      const ref = await pickService(t);
      if (ref) {
        await run(`${title} ${ref.project}/${ref.service}`, () => fn(ref));
      }
    };

  const restartProject = (type?: RestartType) =>
    projectCmd('Restart', (p) => store.client.restartProject(p, type ?? store.config.restartBehavior));

  const register = (id: string, fn: (...args: any[]) => unknown) =>
    context.subscriptions.push(vscode.commands.registerCommand(id, fn));

  register('conc.refresh', () => store.refresh());

  register('conc.openDashboard', (ref?: ServiceRef) =>
    Dashboard.show(context, store, ref?.project && ref?.service ? ref : undefined),
  );

  register('conc.upsertProject', async (arg?: unknown) => {
    const file = await resolveSettingsFile(arg);
    if (!file) {
      return;
    }
    let settings: { name: string; json: string };
    try {
      settings = loadProjectSettings(file);
    } catch (e) {
      void vscode.window.showErrorMessage(`Conc: invalid project settings. ${(e as Error).message}`);
      return;
    }
    const info = await run(`Upsert ${settings.name}`, () => store.client.upsertProject(settings.json));
    if (info) {
      void vscode.window.showInformationMessage(
        `Conc: project '${info.name}' upserted with ${info.services.length} service(s).`,
      );
    }
  });

  register('conc.startProject', projectCmd('Start', (p) => store.client.startProject(p)));
  register('conc.stopProject', projectCmd('Stop', (p) => store.client.stopProject(p)));
  register('conc.restartProject', restartProject());
  register('conc.restartProjectHot', restartProject('Hot'));
  register('conc.restartProjectCold', restartProject('Cold'));
  register('conc.clearProjectLogs', projectCmd('Clear logs of', (p) => store.client.clearProjectLogs(p)));

  register('conc.removeProject', async (t: Target) => {
    const project = await pickProject(t);
    if (!project) {
      return;
    }
    const answer = await vscode.window.showWarningMessage(
      `Remove project '${project}'? Its services will be stopped and its logs deleted.`,
      { modal: true },
      'Remove',
    );
    if (answer === 'Remove') {
      await run(`Remove ${project}`, () => store.client.removeProject(project));
    }
  });

  register('conc.showProjectSettings', async (t: Target) => {
    const project = await pickProject(t);
    if (!project) {
      return;
    }
    const json = await run(`Load settings of ${project}`, () => store.client.getProjectSettings(project));
    if (json === undefined) {
      return;
    }
    let content = json;
    try {
      content = JSON.stringify(JSON.parse(json), null, 2);
    } catch {
      /* show raw */
    }
    const doc = await vscode.workspace.openTextDocument({ language: 'json', content });
    await vscode.window.showTextDocument(doc, { preview: true });
  });

  register('conc.startService', serviceCmd('Start', (r) => store.client.startService(r.project, r.service)));
  register('conc.stopService', serviceCmd('Stop', (r) => store.client.stopService(r.project, r.service)));
  register(
    'conc.restartService',
    serviceCmd('Restart', (r) =>
      store.client.restartService(r.project, r.service, store.config.restartBehavior),
    ),
  );
  register(
    'conc.clearServiceLogs',
    serviceCmd('Clear logs of', (r) => store.client.clearServiceLogs(r.project, r.service)),
  );

  register('conc.openServiceLog', async (t: Target) => {
    const ref = await pickService(t);
    const file = ref && findService(ref)?.logfilePath;
    if (!ref) {
      return;
    }
    if (!file) {
      void vscode.window.showInformationMessage(`Conc: ${ref.project}/${ref.service} has no log file yet.`);
      return;
    }
    try {
      const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(file));
      const editor = await vscode.window.showTextDocument(doc, { preview: true });
      const end = doc.lineAt(doc.lineCount - 1).range.end;
      editor.selection = new vscode.Selection(end, end);
      editor.revealRange(new vscode.Range(end, end));
    } catch (e) {
      void vscode.window.showErrorMessage(`Conc: unable to open log. ${(e as Error).message}`);
    }
  });

  register('conc.tailServiceLog', async (t: Target) => {
    const ref = await pickService(t);
    const file = ref && findService(ref)?.logfilePath;
    if (!ref) {
      return;
    }
    if (!file) {
      void vscode.window.showInformationMessage(`Conc: ${ref.project}/${ref.service} has no log file yet.`);
      return;
    }
    const [cmd, ...args] = store.config.logViewCommand;
    const terminal = vscode.window.createTerminal({
      name: `conc: ${ref.project}/${ref.service}`,
      shellPath: cmd,
      shellArgs: [...args, file],
      iconPath: new vscode.ThemeIcon('output'),
    });
    terminal.show();
  });
}

async function resolveSettingsFile(arg: unknown): Promise<string | undefined> {
  if (arg instanceof vscode.Uri) {
    return arg.fsPath;
  }

  const active = vscode.window.activeTextEditor?.document;
  if (active && path.basename(active.fileName) === 'conc.json') {
    if (active.isDirty) {
      await active.save();
    }
    return active.fileName;
  }

  const found = await vscode.workspace.findFiles('**/conc.json', '**/{node_modules,target,.git}/**', 50);
  if (found.length === 1) {
    return found[0].fsPath;
  }
  if (found.length > 1) {
    const pick = await vscode.window.showQuickPick(
      found.map((uri) => ({ label: vscode.workspace.asRelativePath(uri), uri })),
      { placeHolder: 'Select a conc.json to upsert' },
    );
    return pick?.uri.fsPath;
  }

  const picked = await vscode.window.showOpenDialog({
    canSelectMany: false,
    openLabel: 'Upsert',
    filters: { 'Conc settings': ['json'] },
  });
  return picked?.[0]?.fsPath;
}
