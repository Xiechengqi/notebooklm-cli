export interface ParamSpec {
  name: string;
  kind: string;
  required: boolean;
  description: string;
}

export interface CommandSpec {
  name: string;
  category: string;
  wave: number;
  execution_mode: string;
  summary: string;
  requires_auth: boolean;
  params: ParamSpec[];
}

export interface ToolSpec {
  name: string;
  command: string;
  read_only: boolean;
  requires_auth: boolean;
}

export interface SkillSpec {
  name: string;
  summary: string;
  requires_auth: boolean;
  steps: { use: string }[];
}

export interface ServerConfig {
  host: string;
  port: number;
}

export interface AuthConfig {
  password: string;
  password_changed: boolean;
}

export interface AgentBrowserConfig {
  binary: string;
  session_name: string;
  timeout_secs: number;
}

export interface AccountEntry {
  cdp_port: string;
  email: string;
  display_name: string;
  online: boolean;
  last_checked: number;
}

export interface VncConfig {
  url: string;
  embed: boolean;
}

export interface AppConfig {
  server: ServerConfig;
  agent_browser: AgentBrowserConfig;
  vnc: VncConfig;
}

export interface ExecutionRecord {
  timestamp: number;
  source: string;
  command: string;
  ok: boolean;
  summary: string;
}

export interface BootstrapInfo {
  first_run: boolean;
  password_required: boolean;
  server: { host: string; port: number };
  agent_browser: { binary: string; detected: boolean };
  cdp: { ports: string[]; online: number; offline: number };
  preview: PreviewSyncStatus;
  vnc: { configured: boolean };
}

export interface PreviewNoteEntry {
  id: number;
  cdp_port: string;
  google_account: string;
  notebook_id: string;
  notebook_title: string;
  note_key: string;
  note_title: string;
  content: string;
  content_preview: string;
  fetched_at: number;
  created_at: number;
}

export interface PreviewSyncStatus {
  running: boolean;
  last_started_at: number | null;
  last_finished_at: number | null;
  last_error: string | null;
  last_added: number;
  last_skipped: number;
  last_failed_ports: number;
}

export interface ApiResponse<T> {
  ok: boolean;
  data: T;
  command?: string;
  error?: string;
}
