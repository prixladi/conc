// Resolves effective configuration. VS Code settings win, then ~/.conc/conf.json
// (shared with the cli and gui, see crates/app-config), then built-in defaults.
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import * as vscode from 'vscode';
import { RestartType } from './client';

interface UserAppConfig {
  use_caller_env?: boolean;
  daemon_socket_path?: string;
  log_view_command?: string[];
  default_service_restart_behavior?: RestartType;
}

export interface EffectiveConfig {
  socketPath: string;
  useCallerEnv: boolean;
  logViewCommand: string[];
  restartBehavior: RestartType;
  refreshInterval: number;
  logLines: number;
}

function readUserAppConfig(): UserAppConfig {
  const file = path.join(os.homedir(), '.conc', 'conf.json');
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8')) as UserAppConfig;
  } catch {
    return {};
  }
}

export function getConfig(): EffectiveConfig {
  const user = readUserAppConfig();
  const vs = vscode.workspace.getConfiguration('conc');

  const socketSetting = vs.get<string>('socketPath', '').trim();
  const restartSetting = vs.get<string>('defaultRestartBehavior', '');
  const callerEnvSetting = vs.get<boolean | null>('useCallerEnv', null);

  return {
    socketPath: expandHome(
      socketSetting || user.daemon_socket_path || path.join(os.homedir(), '.conc', 'run', 'conc.sock'),
    ),
    useCallerEnv: callerEnvSetting ?? user.use_caller_env ?? true,
    logViewCommand: user.log_view_command?.length ? user.log_view_command : ['less', '-R', '+GF'],
    restartBehavior:
      restartSetting === 'Hot' || restartSetting === 'Cold'
        ? restartSetting
        : (user.default_service_restart_behavior ?? 'Cold'),
    refreshInterval: Math.max(250, vs.get<number>('refreshInterval', 1000)),
    logLines: Math.max(50, vs.get<number>('logLines', 500)),
  };
}

function expandHome(p: string): string {
  return p.startsWith('~/') ? path.join(os.homedir(), p.slice(2)) : p;
}
