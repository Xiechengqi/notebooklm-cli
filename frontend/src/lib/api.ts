import type {
  AccountEntry,
  ApiResponse,
  AppConfig,
  BootstrapInfo,
  CommandSpec,
  ExecutionRecord,
  PreviewNoteEntry,
  PreviewSyncStatus,
  SkillSpec,
  ToolSpec,
} from './types';

function isApiEnvelope<T>(value: unknown): value is ApiResponse<T> {
  return typeof value === 'object' && value !== null && 'ok' in value && 'data' in value;
}

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, options);
  if (res.status === 401) {
    window.location.href = '/login';
    throw new Error('Unauthorized');
  }
  const json = await res.json();
  return json as T;
}

async function requestApi<T>(url: string, options?: RequestInit): Promise<ApiResponse<T>> {
  const json = await request<unknown>(url, options);
  if (isApiEnvelope<T>(json)) {
    return json;
  }
  return {
    ok: true,
    data: json as T,
  };
}

export async function bootstrap(): Promise<BootstrapInfo> {
  return request('/api/bootstrap');
}

export async function login(password: string): Promise<ApiResponse<{ ok: boolean }>> {
  return requestApi('/api/login', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ password }),
  });
}

export async function logout(): Promise<ApiResponse<{ logged_out: boolean }>> {
  return requestApi('/api/logout', { method: 'POST' });
}

export async function setupPassword(password: string): Promise<ApiResponse<{ configured: boolean }>> {
  return requestApi('/api/setup/password', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ password }),
  });
}

export async function getConfig(): Promise<ApiResponse<AppConfig>> {
  return requestApi('/api/config');
}

export async function updateConfig(config: AppConfig): Promise<ApiResponse<{ saved: boolean }>> {
  return requestApi('/api/config', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(config),
  });
}

export async function getCommands(): Promise<ApiResponse<CommandSpec[]>> {
  return requestApi('/api/commands');
}

export async function getHistory(): Promise<ApiResponse<ExecutionRecord[]>> {
  return requestApi('/api/history');
}

export async function getMcpTools(): Promise<ApiResponse<ToolSpec[]>> {
  return requestApi('/api/mcp/tools');
}

export async function getSkills(): Promise<ApiResponse<SkillSpec[]>> {
  return requestApi('/api/skills');
}

export async function executeCommand(
  command: string,
  params: Record<string, unknown>,
): Promise<ApiResponse<unknown>> {
  return requestApi('/api/execute/' + encodeURIComponent(command), {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ params, format: 'json' }),
  });
}

export async function callMcpTool(
  toolName: string,
  args: Record<string, unknown>,
): Promise<unknown> {
  return request('/mcp', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 'console',
      method: 'tools/call',
      params: { name: toolName, arguments: args },
    }),
  });
}

export async function changePassword(
  newPassword: string,
): Promise<ApiResponse<{ password_changed: boolean }>> {
  return requestApi('/api/password/change', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ password: newPassword }),
  });
}

export async function getCdpPorts(): Promise<ApiResponse<{ ports: string[] }>> {
  return requestApi('/api/cdp-ports');
}

export async function updateCdpPorts(ports: string[]): Promise<ApiResponse<{ ports: string[] }>> {
  return requestApi('/api/cdp-ports', {
    method: 'PUT',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ ports }),
  });
}

export async function refreshCdpPorts(): Promise<ApiResponse<{ refreshing: boolean }>> {
  return requestApi('/api/cdp-ports/refresh', { method: 'POST' });
}

export async function getAccounts(): Promise<ApiResponse<AccountEntry[]>> {
  return requestApi('/api/accounts');
}

export async function getPreviewNotes(): Promise<ApiResponse<PreviewNoteEntry[]>> {
  return requestApi('/api/preview');
}

export async function getPreviewStatus(): Promise<ApiResponse<PreviewSyncStatus>> {
  return requestApi('/api/preview/status');
}

export async function triggerPreviewSync(): Promise<
  ApiResponse<{ added: number; skipped: number; failed_ports: number }>
> {
  return requestApi('/api/preview/sync', { method: 'POST' });
}
