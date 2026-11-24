// Thin client for the conc daemon socket protocol.
// Mirrors crates/daemon-client: requests and responses are lists of arguments
// separated by '\x11', terminated by '\0'.
import * as fs from 'fs';
import * as net from 'net';

const ARG_SEPARATOR = '\x11';
const REQUEST_TIMEOUT_MS = 30_000;

export type ServiceStatus = 'IDLE' | 'RUNNING' | 'STOPPED' | 'EXITED';
export type RestartType = 'Hot' | 'Cold';

export interface ServiceInfo {
  name: string;
  status: ServiceStatus;
  pid: number;
  /** Unix timestamp in seconds, 0 when not set. */
  startTime: number;
  /** Unix timestamp in seconds, 0 when not set. */
  stopTime: number;
  logfilePath: string;
}

export interface ProjectInfo {
  name: string;
  services: ServiceInfo[];
}

export type DaemonErrorKind =
  | 'socket'
  | 'client'
  | 'daemon'
  | 'malformed'
  | 'project_not_found'
  | 'service_not_found';

export class DaemonError extends Error {
  constructor(
    public readonly kind: DaemonErrorKind,
    message: string,
  ) {
    super(message);
  }
}

export class DaemonClient {
  constructor(
    public readonly socketPath: string,
    private readonly getEnv: () => string,
  ) {}

  async isAlive(): Promise<boolean> {
    try {
      if (!fs.statSync(this.socketPath).isSocket()) {
        return false;
      }
      return (await this.send('')) === '';
    } catch {
      return false;
    }
  }

  async getProjectsInfo(): Promise<ProjectInfo[]> {
    const parts = await this.request(['PROJECTS-INFO']);
    const projects: ProjectInfo[] = [];
    let expectName = true;
    for (const line of parts.slice(1)) {
      if (line === '') {
        expectName = true;
        continue;
      }
      if (expectName) {
        projects.push({ name: line, services: [] });
        expectName = false;
        continue;
      }
      if (projects.length === 0) {
        throw malformed(parts);
      }
      projects[projects.length - 1].services.push(parseServiceInfo(line, parts));
    }
    return projects;
  }

  async getProjectInfo(project: string): Promise<ProjectInfo> {
    return parseProjectInfo(await this.request(['PROJECT-INFO', project]));
  }

  async getProjectSettings(project: string): Promise<string> {
    const parts = await this.request(['PROJECT-SETTINGS', project]);
    if (parts.length !== 2) {
      throw malformed(parts);
    }
    return parts[1];
  }

  async upsertProject(settingsJson: string): Promise<ProjectInfo> {
    return parseProjectInfo(await this.request(['PROJECT-UPSERT', settingsJson]));
  }

  async startProject(project: string): Promise<ProjectInfo> {
    return parseProjectInfo(await this.request(['PROJECT-START', project, this.getEnv()]));
  }

  async restartProject(project: string, restart: RestartType): Promise<ProjectInfo> {
    return parseProjectInfo(
      await this.request(['PROJECT-RESTART', project, restart.toUpperCase(), this.getEnv()]),
    );
  }

  async stopProject(project: string): Promise<ProjectInfo> {
    return parseProjectInfo(await this.request(['PROJECT-STOP', project]));
  }

  async removeProject(project: string): Promise<void> {
    await this.request(['PROJECT-REMOVE', project]);
  }

  async clearProjectLogs(project: string): Promise<void> {
    await this.request(['PROJECT-CLEAR-LOGS', project]);
  }

  async startService(project: string, service: string): Promise<ServiceInfo> {
    return parseSingleService(
      await this.request(['SERVICE-START', project, service, this.getEnv()]),
    );
  }

  async restartService(project: string, service: string, restart: RestartType): Promise<ServiceInfo> {
    return parseSingleService(
      await this.request(['SERVICE-RESTART', project, service, restart.toUpperCase(), this.getEnv()]),
    );
  }

  async stopService(project: string, service: string): Promise<ServiceInfo> {
    return parseSingleService(await this.request(['SERVICE-STOP', project, service]));
  }

  async clearServiceLogs(project: string, service: string): Promise<void> {
    await this.request(['SERVICE-CLEAR-LOGS', project, service]);
  }

  private async request(args: string[]): Promise<string[]> {
    let response: string;
    try {
      response = await this.send(args.join(ARG_SEPARATOR));
    } catch (e) {
      throw new DaemonError(
        'socket',
        `Unable to communicate with the daemon socket '${this.socketPath}': ${(e as Error).message}`,
      );
    }
    const parts = response.split(ARG_SEPARATOR);
    if (parts[0] === 'OK') {
      return parts;
    }
    throw toError(parts);
  }

  private send(message: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const chunks: Buffer[] = [];
      const socket = net.createConnection(this.socketPath);
      socket.setTimeout(REQUEST_TIMEOUT_MS);
      socket.on('connect', () => {
        socket.end(Buffer.concat([Buffer.from(message, 'utf8'), Buffer.from([0])]));
      });
      socket.on('data', (chunk) => chunks.push(chunk));
      socket.on('timeout', () => socket.destroy(new Error('request timed out')));
      socket.on('error', reject);
      socket.on('close', (hadError) => {
        if (hadError) {
          return;
        }
        let res = Buffer.concat(chunks).toString('utf8');
        if (res.endsWith('\0')) {
          res = res.slice(0, -1);
        }
        resolve(res);
      });
    });
  }
}

function parseServiceInfo(line: string, raw: string[]): ServiceInfo {
  const p = line.split(' ');
  const statuses: ServiceStatus[] = ['IDLE', 'RUNNING', 'STOPPED', 'EXITED'];
  if (p.length < 6 || !statuses.includes(p[1] as ServiceStatus)) {
    throw malformed(raw);
  }
  const pid = Number(p[2]);
  const startTime = Number(p[3]);
  const stopTime = Number(p[4]);
  if ([pid, startTime, stopTime].some((n) => !Number.isFinite(n))) {
    throw malformed(raw);
  }
  // Log path is the remainder, so paths with spaces survive.
  const logfile = p.slice(5).join(' ');
  return {
    name: p[0],
    status: p[1] as ServiceStatus,
    pid,
    startTime,
    stopTime,
    logfilePath: logfile === '-' ? '' : logfile,
  };
}

function parseProjectInfo(parts: string[]): ProjectInfo {
  if (parts.length < 2 || parts[1].includes(' ')) {
    throw malformed(parts);
  }
  return {
    name: parts[1],
    services: parts.slice(2).map((l) => parseServiceInfo(l, parts)),
  };
}

function parseSingleService(parts: string[]): ServiceInfo {
  if (parts.length !== 2) {
    throw malformed(parts);
  }
  return parseServiceInfo(parts[1], parts);
}

function malformed(parts: string[]): DaemonError {
  return new DaemonError('malformed', `Unable to parse daemon response: ${parts.join(' ')}`);
}

function toError(parts: string[]): DaemonError {
  const raw = parts.join(' ');
  if (parts[0] !== 'ERROR') {
    return malformed(parts);
  }
  const name = parts[1] ?? '';
  if (name === 'project_not_found') {
    return new DaemonError('project_not_found', 'Project was not found.');
  }
  if (name === 'service_not_found') {
    return new DaemonError('service_not_found', 'Service was not found in the project.');
  }
  if (
    name.startsWith('settings.') ||
    name.startsWith('env.') ||
    name === 'unknown_command' ||
    name === 'invalid_argument_count'
  ) {
    return new DaemonError('client', `Request rejected by the daemon: ${raw}`);
  }
  if (name.startsWith('unknown-code-') || name === 'driver_error' || name === 'manager_error') {
    return new DaemonError('daemon', `Daemon error, check its logs for more info: ${raw}`);
  }
  return new DaemonError('daemon', `Daemon error: ${raw}`);
}
