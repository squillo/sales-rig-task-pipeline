-- Schema for rig-task-pipeline SQLite persistence
--
-- Revision History
-- - 2025-11-14T16:22:30Z @AI: Add initial tasks table for Task entity persistence.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    assignee TEXT NULL,
    due_date TEXT NULL,
    status TEXT NOT NULL,
    source_transcript_id TEXT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    enhancements_json TEXT NULL,
    comprehension_tests_json TEXT NULL
);
