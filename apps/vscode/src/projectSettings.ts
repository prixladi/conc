// Port of crates/project-settings: reads a conc.json, validates it and produces
// the JSON payload expected by PROJECT-UPSERT.
import * as fs from 'fs';
import * as path from 'path';

type EnvValue = string | number;

interface ServiceSettings {
  name: string;
  pwd?: string;
  command: string[];
  env?: Record<string, EnvValue>;
}

interface ProjectSettings {
  name: string;
  cwd?: string;
  services: ServiceSettings[];
  env?: Record<string, EnvValue>;
}

const NAME_RE = /^[\p{L}\p{N}_-]+$/u;

export function loadProjectSettings(settingsPath: string): { name: string; json: string } {
  let file = settingsPath;
  if (fs.statSync(file).isDirectory()) {
    file = path.join(file, 'conc.json');
  }

  let raw: unknown;
  try {
    raw = JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch (e) {
    throw new Error(`Unable to parse '${file}': ${(e as Error).message}`);
  }

  const settings = validate(raw);
  const dir = path.dirname(file);

  const out = {
    name: settings.name,
    cwd: resolveCwd(dir, settings.cwd ?? ''),
    services: settings.services.map((s) => {
      const service: Record<string, unknown> = { name: s.name };
      if (s.pwd !== undefined) {
        service.pwd = s.pwd;
      }
      service.command = s.command;
      const env = normalizeEnv(s.env);
      if (Object.keys(env).length) {
        service.env = env;
      }
      return service;
    }),
  } as Record<string, unknown>;

  const env = normalizeEnv(settings.env);
  if (Object.keys(env).length) {
    out.env = env;
  }

  return { name: settings.name, json: JSON.stringify(out) };
}

function validate(raw: unknown): ProjectSettings {
  if (typeof raw !== 'object' || raw === null) {
    throw new Error('settings must be a JSON object');
  }
  const s = raw as Partial<ProjectSettings>;
  if (typeof s.name !== 'string' || !s.name) {
    throw new Error('project name is empty');
  }
  if (!NAME_RE.test(s.name)) {
    throw new Error("project name is invalid, characters must be alphanumeric or '-' or '_'");
  }
  if (s.cwd !== undefined && typeof s.cwd !== 'string') {
    throw new Error("'cwd' must be a string");
  }
  if (!Array.isArray(s.services) || s.services.length === 0) {
    throw new Error('no service was specified');
  }
  const seen = new Set<string>();
  for (const service of s.services) {
    if (typeof service?.name !== 'string' || !service.name) {
      throw new Error('service name is empty');
    }
    if (!NAME_RE.test(service.name)) {
      throw new Error(
        `service '${service.name}' is invalid, characters must be alphanumeric or '-' or '_'`,
      );
    }
    if (
      !Array.isArray(service.command) ||
      service.command.length === 0 ||
      service.command.some((c) => typeof c !== 'string')
    ) {
      throw new Error(`service '${service.name}' has an empty or invalid command`);
    }
    if (seen.has(service.name)) {
      throw new Error(`service '${service.name}' is declared more than once`);
    }
    seen.add(service.name);
  }
  return s as ProjectSettings;
}

function resolveCwd(settingsDir: string, cwd: string): string {
  if (!cwd) {
    return settingsDir;
  }
  return path.isAbsolute(cwd) ? cwd : path.join(settingsDir, cwd);
}

function normalizeEnv(env: Record<string, EnvValue> | undefined): Record<string, string> {
  const result: Record<string, string> = {};
  for (const key of Object.keys(env ?? {}).sort()) {
    const value = env![key];
    if (typeof value !== 'string' && typeof value !== 'number') {
      throw new Error(`env variable '${key}' must be a string or a number`);
    }
    result[key] = String(value);
  }
  return result;
}
