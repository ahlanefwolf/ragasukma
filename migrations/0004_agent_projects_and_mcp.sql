-- 0004_agent_projects_and_mcp.sql
-- Agent Workspace: Projects -> Sessions -> Messages, Prompt Skills, & MCP Support

PRAGMA foreign_keys = ON;

-- ============================================================================
-- 1. PROMPT SKILLS (Reusable agent persona, prompt instructions, system rules)
-- ============================================================================
CREATE TABLE IF NOT EXISTS prompt_skills (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE COLLATE NOCASE,
    description TEXT NOT NULL DEFAULT '',
    system_prompt TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '[]',             -- JSON Array: ["coding", "rust", "review"]
    is_active INTEGER NOT NULL DEFAULT 1,        -- 1 = active, 0 = disabled
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_prompt_skills_slug ON prompt_skills(slug);
CREATE INDEX IF NOT EXISTS idx_prompt_skills_is_active ON prompt_skills(is_active);

-- ============================================================================
-- 2. MCP SERVERS (Registry MCP yang bisa dipanggil agent)
-- ============================================================================
CREATE TABLE IF NOT EXISTS mcp_servers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    transport TEXT NOT NULL DEFAULT 'stdio',     -- 'stdio', 'sse', 'http'
    command TEXT,                                -- e.g. 'npx'
    args TEXT NOT NULL DEFAULT '[]',             -- JSON Array: ["-y", "@modelcontextprotocol/server-filesystem"]
    env TEXT NOT NULL DEFAULT '{}',              -- JSON Object: {"ROOT_DIR": "/path/to/project"}
    url TEXT,                                    -- Endpoint jika transport 'sse'/'http'
    is_enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL
);

-- ============================================================================
-- 3. PROJECTS (Workspace level isolasi)
-- ============================================================================
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    root_path TEXT,                              -- Optional: path direktori lokal untuk MCP filesystem
    custom_instructions TEXT,                    -- System prompt tambahan khusus project ini
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at DESC);

-- Relasi Many-to-Many: Project <-> Prompt Skills
CREATE TABLE IF NOT EXISTS project_prompt_skills (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL REFERENCES prompt_skills(id) ON DELETE CASCADE,
    priority INTEGER NOT NULL DEFAULT 0,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (project_id, skill_id)
);

CREATE INDEX IF NOT EXISTS idx_proj_skills_proj ON project_prompt_skills(project_id);

-- Relasi Many-to-Many: Project <-> MCP Servers
CREATE TABLE IF NOT EXISTS project_mcp_servers (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    server_id TEXT NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (project_id, server_id)
);

CREATE INDEX IF NOT EXISTS idx_proj_mcp_proj ON project_mcp_servers(project_id);

-- ============================================================================
-- 4. SESSIONS (Eksekusi Task / Thread spesifik)
-- ============================================================================
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'idle',         -- 'idle', 'running', 'waiting_tool', 'completed', 'failed'
    model TEXT,
    temperature REAL DEFAULT 0.7,
    source_chat_id TEXT,                         -- Optional: merujuk ke chat_history(id) on-demand
    metadata TEXT NOT NULL DEFAULT '{}',         -- JSON flexible config
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_project_id ON sessions(project_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at DESC);

-- ============================================================================
-- 5. MESSAGES (Agent message logs, tool calls, tool responses & reasoning)
-- ============================================================================
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    sequence_num INTEGER NOT NULL,
    role TEXT NOT NULL,                          -- 'user', 'assistant', 'system', 'tool'
    content TEXT,
    thought TEXT,                                -- Chain of thought / reasoning model
    tool_calls TEXT,                             -- JSON Array format MCP/OpenAI/Anthropic tool calls
    tool_call_id TEXT,                           -- Referensi tool_calls saat role = 'tool'
    status TEXT NOT NULL DEFAULT 'success',      -- 'streaming', 'success', 'error'
    token_usage_input INTEGER DEFAULT 0,
    token_usage_output INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_messages_session_seq ON messages(session_id, sequence_num);
CREATE INDEX IF NOT EXISTS idx_messages_session_created ON messages(session_id, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_messages_role ON messages(role);
