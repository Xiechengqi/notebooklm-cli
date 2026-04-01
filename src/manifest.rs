use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ParamSpec {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    pub name: &'static str,
    pub category: &'static str,
    pub wave: u8,
    pub execution_mode: &'static str,
    pub summary: &'static str,
    pub requires_auth: bool,
    pub params: Vec<ParamSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolSpec {
    pub name: &'static str,
    pub command: &'static str,
    pub read_only: bool,
    pub requires_auth: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillStep {
    pub r#use: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub requires_auth: bool,
    pub steps: Vec<SkillStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeSpec {
    pub binary: &'static str,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerDefaults {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModelSpec {
    pub mode: &'static str,
    pub cookie_name: &'static str,
    pub bearer_format: &'static str,
    pub first_run_requires_password_setup: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentBrowserSpec {
    pub binding: &'static str,
    pub binary_auto_detect: bool,
    pub multi_account: bool,
    pub default_session_name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescribeManifest {
    pub site: SiteSpec,
    pub runtime: RuntimeSpec,
    pub server_defaults: ServerDefaults,
    pub auth_model: AuthModelSpec,
    pub agent_browser: AgentBrowserSpec,
    pub commands: Vec<CommandSpec>,
    pub mcp_tools: Vec<ToolSpec>,
    pub skills: Vec<SkillSpec>,
}

pub fn build_manifest(config_path: String, host: String, port: u16) -> DescribeManifest {
    DescribeManifest {
        site: SiteSpec {
            id: "notebooklm",
            name: "NotebookLM CLI",
            version: env!("CARGO_PKG_VERSION"),
        },
        runtime: RuntimeSpec {
            binary: "notebooklm-cli",
            config_path,
        },
        server_defaults: ServerDefaults {
            base_url: format!("http://{host}:{port}"),
            host,
            port,
        },
        auth_model: AuthModelSpec {
            mode: "shared-password",
            cookie_name: "notebooklm_cli_token",
            bearer_format: "Authorization: Bearer <password>",
            first_run_requires_password_setup: true,
        },
        agent_browser: AgentBrowserSpec {
            binding: "cli",
            binary_auto_detect: true,
            multi_account: true,
            default_session_name: "notebooklm-cli",
        },
        commands: command_specs(),
        mcp_tools: tool_specs(),
        skills: skill_specs(),
    }
}

pub fn command_specs() -> Vec<CommandSpec> {
    vec![
        // Wave 1: read-only basics
        CommandSpec {
            name: "status",
            category: "read",
            wave: 1,
            execution_mode: "rpc-first",
            summary: "Check NotebookLM page availability and login state",
            requires_auth: true,
            params: vec![],
        },
        CommandSpec {
            name: "list",
            category: "read",
            wave: 1,
            execution_mode: "rpc-first",
            summary: "List all notebooks",
            requires_auth: true,
            params: vec![],
        },
        CommandSpec {
            name: "get",
            category: "read",
            wave: 1,
            execution_mode: "rpc-first",
            summary: "Get notebook metadata (emoji, source count, timestamps)",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "notebook_id",
                kind: "string",
                required: false,
                description: "Notebook ID (defaults to current)",
            }],
        },
        CommandSpec {
            name: "summary",
            category: "read",
            wave: 1,
            execution_mode: "rpc-first",
            summary: "Get the summary block from a notebook",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "notebook_id",
                kind: "string",
                required: false,
                description: "Notebook ID (defaults to current)",
            }],
        },
        CommandSpec {
            name: "source_list",
            category: "read",
            wave: 1,
            execution_mode: "rpc-first",
            summary: "List sources in a notebook",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "notebook_id",
                kind: "string",
                required: false,
                description: "Notebook ID (defaults to current)",
            }],
        },
        CommandSpec {
            name: "source_get",
            category: "read",
            wave: 1,
            execution_mode: "rpc-first",
            summary: "Get a single source by ID or title",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "source",
                kind: "string",
                required: true,
                description: "Source ID or title substring",
            }],
        },
        // Wave 2: read-only deep
        CommandSpec {
            name: "source_fulltext",
            category: "read",
            wave: 2,
            execution_mode: "rpc-first",
            summary: "Get full text content of a source",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "source",
                kind: "string",
                required: true,
                description: "Source ID or title substring",
            }],
        },
        CommandSpec {
            name: "source_guide",
            category: "read",
            wave: 2,
            execution_mode: "rpc-first",
            summary: "Get guide summary and keywords for a source",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "source",
                kind: "string",
                required: true,
                description: "Source ID or title substring",
            }],
        },
        CommandSpec {
            name: "history",
            category: "read",
            wave: 2,
            execution_mode: "rpc-first",
            summary: "List conversation history threads",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "notebook_id",
                kind: "string",
                required: false,
                description: "Notebook ID (defaults to current)",
            }],
        },
        CommandSpec {
            name: "note_list",
            category: "read",
            wave: 2,
            execution_mode: "ui-first",
            summary: "List saved notes from the Studio panel",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "notebook_id",
                kind: "string",
                required: false,
                description: "Notebook ID (defaults to current)",
            }],
        },
        CommandSpec {
            name: "note_get",
            category: "read",
            wave: 2,
            execution_mode: "ui-first",
            summary: "Get content of a specific note",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "note",
                kind: "string",
                required: true,
                description: "Note title or ID",
            }],
        },
    ]
}

pub fn tool_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "notebooklm_status",
            command: "status",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_list",
            command: "list",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_get",
            command: "get",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_summary",
            command: "summary",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_source_list",
            command: "source_list",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_source_get",
            command: "source_get",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_source_fulltext",
            command: "source_fulltext",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_source_guide",
            command: "source_guide",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_history",
            command: "history",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_note_list",
            command: "note_list",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "notebooklm_note_get",
            command: "note_get",
            read_only: true,
            requires_auth: true,
        },
    ]
}

pub fn skill_specs() -> Vec<SkillSpec> {
    vec![
        SkillSpec {
            name: "research_notebook",
            summary: "Get notebook overview: summary + source list + conversation history",
            requires_auth: true,
            steps: vec![
                SkillStep { r#use: "summary" },
                SkillStep {
                    r#use: "source_list",
                },
                SkillStep { r#use: "history" },
            ],
        },
        SkillSpec {
            name: "deep_read_source",
            summary: "Deep-read a source: guide summary + full text",
            requires_auth: true,
            steps: vec![
                SkillStep {
                    r#use: "source_guide",
                },
                SkillStep {
                    r#use: "source_fulltext",
                },
            ],
        },
        SkillSpec {
            name: "notebook_overview",
            summary: "Global overview: list all notebooks and check status",
            requires_auth: true,
            steps: vec![
                SkillStep { r#use: "list" },
                SkillStep { r#use: "status" },
            ],
        },
    ]
}
