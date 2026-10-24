-- 0004_agent_projects_and_mcp.sql
-- Agent Workspace: Projects, Prompt Skills, MCP Support, Goals, Evidence Cache, & Verification History

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

-- ============================================================================
-- 6. GOALS (Goal Store: Demand, Criteria, Status & Phase Transitions)
-- ============================================================================
CREATE TABLE IF NOT EXISTS goals (
    id TEXT PRIMARY KEY NOT NULL,                                     -- e.g. 'G1', 'G2', or UUID
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,       -- Relates to active thread session
    parent_goal_id TEXT REFERENCES goals(id) ON DELETE SET NULL,     -- Hierarchy / antecedent demand if any
    title TEXT NOT NULL,                                              -- Short title
    demand TEXT NOT NULL,                                             -- Original user demand
    success_criteria TEXT NOT NULL DEFAULT '[]',                      -- JSON Array of criteria
    constraints TEXT NOT NULL DEFAULT '[]',                           -- JSON Array of constraints
    phase TEXT NOT NULL DEFAULT 'discovery',                          -- 'discovery', 'action', 'verification', 'concluded'
    status TEXT NOT NULL DEFAULT 'in_progress',                       -- 'in_progress', 'completed', 'failed'
    target_files TEXT NOT NULL DEFAULT '[]',                          -- JSON Array of locked target files
    discovery_count INTEGER NOT NULL DEFAULT 0,                       -- Discovery calls counter (budget 3-4)
    conclusion TEXT,                                                  -- Summary of why goal criteria met/failed
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_goals_project_id ON goals(project_id);
CREATE INDEX IF NOT EXISTS idx_goals_session_id ON goals(session_id);
CREATE INDEX IF NOT EXISTS idx_goals_parent_id ON goals(parent_goal_id);
CREATE INDEX IF NOT EXISTS idx_goals_phase ON goals(phase);
CREATE INDEX IF NOT EXISTS idx_goals_status ON goals(status);
CREATE INDEX IF NOT EXISTS idx_goals_created_at ON goals(created_at DESC);

-- ============================================================================
-- 7. EVIDENCE CACHE (Hasil eksplorasi workspace/file lintas demand)
-- ============================================================================
CREATE TABLE IF NOT EXISTS evidence_cache (
    id TEXT PRIMARY KEY NOT NULL,                                     -- UUID
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,                                          -- Path file project e.g. 'src/auth/service.rs'
    file_version TEXT NOT NULL,                                       -- Hash SHA256 atau file mtime saat dibaca
    start_line INTEGER,                                               -- Rentang baris awal
    end_line INTEGER,                                                 -- Rentang baris akhir
    symbol_name TEXT,                                                 -- e.g. 'validate_token' / class / function
    summary TEXT NOT NULL,                                            -- Ringkasan temuan file / fungsi
    raw_snippet TEXT,                                                 -- Potongan kode relevan (opsional)
    is_stale INTEGER NOT NULL DEFAULT 0,                              -- 1 jika file asli berubah (perlu validasi ulang)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_evidence_proj_file ON evidence_cache(project_id, file_path);
CREATE INDEX IF NOT EXISTS idx_evidence_is_stale ON evidence_cache(is_stale);

-- ============================================================================
-- 8. GOAL EVIDENCE (Context Builder: Mapping & Filter Relevansi Evidence per Goal)
-- ============================================================================
CREATE TABLE IF NOT EXISTS goal_evidence (
    goal_id TEXT NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    evidence_id TEXT NOT NULL REFERENCES evidence_cache(id) ON DELETE CASCADE,
    relevance_score REAL DEFAULT 1.0,                                 -- Nilai relevansi terhadap goal ini
    is_active INTEGER NOT NULL DEFAULT 1,                             -- 1 = disertakan ke prompt LLM, 0 = dikecualikan
    PRIMARY KEY (goal_id, evidence_id)
);

CREATE INDEX IF NOT EXISTS idx_goal_evidence_active ON goal_evidence(goal_id, is_active);

-- ============================================================================
-- 9. GOAL VERIFICATIONS (Verification Loop History & Hasil Audit Test)
-- ============================================================================
CREATE TABLE IF NOT EXISTS goal_verifications (
    id TEXT PRIMARY KEY NOT NULL,                                     -- UUID
    goal_id TEXT NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    iteration INTEGER NOT NULL DEFAULT 1,                             -- Iterasi loop ke-n
    test_command TEXT NOT NULL,                                       -- Perintah e.g. 'cargo test test_login'
    test_output TEXT NOT NULL,                                        -- Output hasil eksekusi
    is_passed INTEGER NOT NULL,                                       -- 1 = lolos kriteria, 0 = gagal
    feedback TEXT,                                                    -- Evaluasi kegagalan untuk discovery/action berikutnya
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_goal_verif_goal_iter ON goal_verifications(goal_id, iteration);
