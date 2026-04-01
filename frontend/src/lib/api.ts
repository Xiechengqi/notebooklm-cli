import type {
  AccountEntry,
  ApiResponse,
  AppConfig,
  BootstrapInfo,
  CommandSpec,
  ExecutionRecord,
  SkillSpec,
  ToolSpec,
} from './types';

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, options);
  if (res.status === 401) {
    window.location.href = '/login';
    throw new Error('Unauthorized');
  }
  return res.json();
}

export async function bootstrap(): Promise<BootstrapInfo> {
  return request('/api/bootstrap');
}

export async function login(password: string): Promise<ApiResponse<{ ok: boolean }>> {
  return request('/api/login', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ password }),
  });
}

export async function logout(): Promise<ApiResponse<{ logged_out: boolean }>> {
  return request('/api/logout', { method: 'POST' });
}

export async function setupPassword(password: string): Promise<ApiResponse<{ configured: boolean }>> {
  return request('/api/setup/password', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ password }),
  });
}

export async function getConfig(): Promise<ApiResponse<AppConfig>> {
  return request('/api/config');
}

export async function updateConfig(config: AppConfig): Promise<ApiResponse<{ saved: boolean }>> {
  return request('/api/config', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(config),
  });
}

export async function getCommands(): Promise<ApiResponse<CommandSpec[]>> {
  return request('/api/commands');
}

export async function getHistory(): Promise<ApiResponse<ExecutionRecord[]>> {
  return request('/api/history');
}

export async function getMcpTools(): Promise<ApiResponse<ToolSpec[]>> {
  return request('/api/mcp/tools');
}

export async function getSkills(): Promise<ApiResponse<SkillSpec[]>> {
  return request('/api/skills');
}

export async function executeCommand(
  command: string,
  params: Record<string, unknown>,
): Promise<ApiResponse<unknown>> {
  return request('/api/execute/' + encodeURIComponent(command), {
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
  return request('/api/password/change', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ new_password: newPassword }),
  });
}

export async function getCdpPorts(): Promise<ApiResponse<{ ports: string[] }>> {
  return request('/api/cdp-ports');
}

export async function updateCdpPorts(ports: string[]): Promise<ApiResponse<{ ports: string[] }>> {
  return request('/api/cdp-ports', {
    method: 'PUT',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ ports }),
  });
}

export async function refreshCdpPorts(): Promise<ApiResponse<{ refreshing: boolean }>> {
  return request('/api/cdp-ports/refresh', { method: 'POST' });
}

export async function getAccounts(): Promise<ApiResponse<AccountEntry[]>> {
  return request('/api/accounts');
}
