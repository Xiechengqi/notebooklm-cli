type Lang = 'en' | 'zh';

const dict = {
  nav: {
    en: { console: 'Console', commands: 'Commands', mcp: 'MCP', cdp: 'CDP', skills: 'Skills', docs: 'Docs', settings: 'Settings', history: 'History', logout: 'Logout', login: 'Login', setup_password: 'Setup Password' },
    zh: { console: '控制台', commands: '命令', mcp: 'MCP', cdp: 'CDP', skills: '技能', docs: '文档', settings: '设置', history: '历史', logout: '登出', login: '登录', setup_password: '设置密码' },
  },
  theme: {
    en: { light: 'Light', dark: 'Dark', auto: 'Auto' },
    zh: { light: '亮色', dark: '暗色', auto: '自动' },
  },
  home: {
    en: {
      title: 'Console', tagline: 'Local NotebookLM automation control plane backed by ', tagline_suffix: '.',
      service_status: 'Service Status', agent_browser: 'Agent Browser', quick_actions: 'Quick Actions',
      recent_executions: 'Recent Executions', not_set: 'not set',
      action_commands: 'Run notebook, source, note, and history read commands',
      action_mcp: 'Review MCP tools and auth model',
      action_settings: 'Adjust server, agent-browser, and auth settings',
      dt_api: 'API', dt_config: 'Config', dt_binary: 'Binary', dt_cdp: 'CDP Ports',
      binary_not_set: 'Not installed. Install from', cdp_not_set: 'No ports configured.',
      cdp_online: ' online', cdp_offline: ' offline', cdp_manage: 'Manage',
      claude_code_title: 'Add to Claude Code',
      claude_code_subtitle: 'One command to register notebooklm-cli as a Claude Code MCP server — gives Claude access to all NotebookLM commands.',
      claude_code_step1: 'Copy the command below and run it in your terminal:',
      claude_code_replace: 'Replace <PASSWORD> with your console password. Restart Claude Code after adding.',
      claude_code_badge: 'MCP Integration',
      claude_code_auth_title: 'Authentication',
      claude_code_auth_desc: 'Use the Console password as the Bearer token. Go to Settings to change it.',
      copy: 'Copy', copied: 'Copied!',
    },
    zh: {
      title: '控制台', tagline: '基于 ', tagline_suffix: ' 的本地 NotebookLM 自动化控制面板。',
      service_status: '服务状态', agent_browser: 'Agent Browser', quick_actions: '快捷操作',
      recent_executions: '最近执行', not_set: '未设置',
      action_commands: '运行 notebook、source、note 和 history 只读命令',
      action_mcp: '查看 MCP 工具和认证模型',
      action_settings: '调整服务器、agent-browser 和认证设置',
      dt_api: 'API', dt_config: '配置', dt_binary: '可执行文件', dt_cdp: 'CDP 端口',
      binary_not_set: '未安装，从以下地址安装', cdp_not_set: '未配置，请前往 CDP 页面添加端口。',
      cdp_online: ' 在线', cdp_offline: ' 离线', cdp_manage: '管理',
      claude_code_title: '添加到 Claude Code',
      claude_code_subtitle: '一条命令将 notebooklm-cli 注册为 Claude Code MCP server — 让 Claude 访问所有 NotebookLM 命令。',
      claude_code_step1: '复制以下命令并在终端运行：',
      claude_code_replace: '将 <PASSWORD> 替换为控制台密码。添加后重启 Claude Code 生效。',
      claude_code_badge: 'MCP 集成',
      claude_code_auth_title: '认证方式',
      claude_code_auth_desc: '使用控制台密码作为 Bearer token。前往设置页面可以修改密码。',
      copy: '复制', copied: '已复制！',
    },
  },
  login: {
    en: { title: 'Login', description: 'Use the Console password. The same credential also works as API and MCP Bearer token.', password: 'Password', submit: 'Login' },
    zh: { title: '登录', description: '使用控制台密码。同一凭据也可用作 API 和 MCP Bearer token。', password: '密码', submit: '登录' },
  },
  setup_password: {
    en: { title: 'Setup Password', description: 'First run requires a password. This password will also act as the API and MCP Bearer token.', password: 'Password', submit: 'Save Password' },
    zh: { title: '设置密码', description: '首次运行需要设置密码。此密码也将用作 API 和 MCP Bearer token。', password: '密码', submit: '保存密码' },
  },
  commands: {
    en: { title: 'Command Runner', description: 'Run any registered command through the same API used by CLI and MCP mappings.', command_label: 'Command', execute: 'Execute', running: ' Running…', registered: 'Commands', account: 'Account', no_accounts: 'No accounts found. Add CDP ports and refresh.' },
    zh: { title: '命令执行器', description: '通过 CLI 和 MCP 映射使用的同一 API 运行任何已注册命令。', command_label: '命令', execute: '执行', running: ' 执行中…', registered: '命令列表', account: '账号', no_accounts: '未发现账号，请前往 CDP 页面添加端口并刷新。' },
  },
  mcp: {
    en: { title: 'MCP', description: 'All MCP tools use the same password as Console and API.', endpoint: 'Endpoint: ', tool_label: 'Tool', arguments_label: 'Arguments (JSON)', call_tool: 'Call Tool', tools_heading: 'Tools', account: 'Account', no_accounts: 'No accounts found. Add CDP ports and refresh.' },
    zh: { title: 'MCP', description: '所有 MCP 工具使用与控制台和 API 相同的密码。', endpoint: '端点：', tool_label: '工具', arguments_label: '参数 (JSON)', call_tool: '调用工具', tools_heading: '工具列表', account: '账号', no_accounts: '未发现账号，请前往 CDP 页面添加端口并刷新。' },
  },
  settings: {
    en: { title: 'Settings', server: 'Server', host: 'Host', port: 'Port', agent_browser: 'Agent Browser', binary: 'Binary', timeout: 'Timeout (seconds)', not_detected: 'Not detected', vnc: 'VNC', url: 'URL', save: 'Save Config', reset: 'Reset', change_password: 'Change Password', new_password: 'New Password', confirm_password: 'Confirm Password', password_mismatch: 'Passwords do not match', new_password_required: 'New password is required' },
    zh: { title: '设置', server: '服务器', host: '主机', port: '端口', agent_browser: 'Agent Browser', binary: '可执行文件', timeout: '超时（秒）', not_detected: '未检测到', vnc: 'VNC', url: 'URL', save: '保存配置', reset: '重置', change_password: '修改密码', new_password: '新密码', confirm_password: '确认密码', password_mismatch: '两次输入的密码不一致', new_password_required: '请输入新密码' },
  },
  docs: {
    en: { title: 'Docs', description: 'Shared source of truth for commands, MCP tools, and skills.', command: 'Command', category: 'Category', mode: 'Mode', summary: 'Summary' },
    zh: { title: '文档', description: '命令、MCP 工具和技能的统一参考。', command: '命令', category: '分类', mode: '模式', summary: '摘要' },
  },
  skills: {
    en: { skills_title: 'Skills', skills_description: 'Predefined multi-step workflows that chain commands together for complex tasks.' },
    zh: { skills_title: '技能', skills_description: '将多个命令链接在一起执行复杂任务的预定义工作流。' },
  },
  components: {
    en: { no_executions: 'No commands have been executed yet.', when: 'When', source: 'Source', command: 'Command', status: 'Status', summary_heading: 'Summary', just_now: 'just now', status_ok: 'ok', status_err: 'error', minutes_ago: 'm ago', hours_ago: 'h ago', days_ago: 'd ago' },
    zh: { no_executions: '尚无已执行的命令。', when: '时间', source: '来源', command: '命令', status: '状态', summary_heading: '摘要', just_now: '刚刚', status_ok: '成功', status_err: '错误', minutes_ago: '分钟前', hours_ago: '小时前', days_ago: '天前' },
  },
  history: {
    en: { title: 'Execution History', description: 'Full log of all command executions across CLI, API, and MCP.' },
    zh: { title: '执行历史', description: '所有通过 CLI、API 和 MCP 执行的命令完整记录。' },
  },
  cdp: {
    en: {
      title: 'CDP Ports',
      description: 'Manage Chrome DevTools Protocol ports. Each port maps to a logged-in Google account.',
      ports_heading: 'Managed Ports',
      accounts_heading: 'Discovered Accounts',
      col_port: 'Port',
      col_account: 'Account',
      col_name: 'Name',
      col_status: 'Status',
      col_last_checked: 'Last Checked',
      add_placeholder: 'e.g. 9222',
      add_button: 'Add',
      refresh: 'Refresh Now',
      refreshing: 'Refreshing…',
      remove: 'Remove',
      online: 'online',
      offline: 'offline',
      pending: 'pending',
      never: 'never',
      no_ports: 'No ports configured. Add a port above.',
      no_accounts: 'No accounts discovered yet. Add ports and click Refresh Now.',
    },
    zh: {
      title: 'CDP 端口',
      description: '管理 Chrome DevTools Protocol 端口，每个端口对应一个已登录的 Google 账号。',
      ports_heading: '已管理端口',
      accounts_heading: '已发现账号',
      col_port: '端口',
      col_account: '账号',
      col_name: '名称',
      col_status: '状态',
      col_last_checked: '最后检测',
      add_placeholder: '例如 9222',
      add_button: '添加',
      refresh: '立即刷新',
      refreshing: '刷新中…',
      remove: '删除',
      online: '在线',
      offline: '离线',
      pending: '待检测',
      never: '从未',
      no_ports: '未配置端口，请在上方添加。',
      no_accounts: '尚未发现账号，请添加端口并点击立即刷新。',
    },
  },
} as const;

export type Translations = typeof dict;
export type Section = keyof Translations;

export function t(lang: Lang) {
  return {
    nav: dict.nav[lang],
    theme: dict.theme[lang],
    home: dict.home[lang],
    login: dict.login[lang],
    setup_password: dict.setup_password[lang],
    commands: dict.commands[lang],
    mcp: dict.mcp[lang],
    settings: dict.settings[lang],
    docs: dict.docs[lang],
    skills: dict.skills[lang],
    components: dict.components[lang],
    history: dict.history[lang],
    cdp: dict.cdp[lang],
  };
}

export type { Lang };
