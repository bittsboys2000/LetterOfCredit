-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Audit Logs Table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    event_type TEXT NOT NULL,
    actor_id UUID NOT NULL,
    resource_id UUID, -- Can be LC ID, Doc ID, etc.
    details JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
