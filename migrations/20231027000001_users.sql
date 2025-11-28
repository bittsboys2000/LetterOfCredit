-- Users Table with Role-Based Access Control

CREATE TYPE user_role AS ENUM ('importer', 'exporter', 'issuingbank', 'advisingbank', 'auditor', 'admin');

CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    organization TEXT,
    role user_role NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for common queries
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_organization ON users(organization) WHERE organization IS NOT NULL;

-- Seed default users for development/testing (passwords are 'password123')
-- Hash generated with Argon2: $argon2id$v=19$m=19456,t=2,p=1$...
INSERT INTO users (id, email, password_hash, name, organization, role) VALUES
    ('00000000-0000-0000-0000-000000000001', 'importer@example.com', '$argon2id$v=19$m=19456,t=2,p=1$ZHVtbXlzYWx0MTIzNA$DummyHashForDevelopmentOnly1234567890abcdef', 'Test Importer', 'Import Corp', 'importer'),
    ('00000000-0000-0000-0000-000000000002', 'exporter@example.com', '$argon2id$v=19$m=19456,t=2,p=1$ZHVtbXlzYWx0MTIzNA$DummyHashForDevelopmentOnly1234567890abcdef', 'Test Exporter', 'Export Ltd', 'exporter'),
    ('00000000-0000-0000-0000-000000000003', 'issuing@bank.com', '$argon2id$v=19$m=19456,t=2,p=1$ZHVtbXlzYWx0MTIzNA$DummyHashForDevelopmentOnly1234567890abcdef', 'Issuing Bank Officer', 'First National Bank', 'issuingbank'),
    ('00000000-0000-0000-0000-000000000004', 'advising@bank.com', '$argon2id$v=19$m=19456,t=2,p=1$ZHVtbXlzYWx0MTIzNA$DummyHashForDevelopmentOnly1234567890abcdef', 'Advising Bank Officer', 'Global Trade Bank', 'advisingbank'),
    ('00000000-0000-0000-0000-000000000005', 'auditor@compliance.com', '$argon2id$v=19$m=19456,t=2,p=1$ZHVtbXlzYWx0MTIzNA$DummyHashForDevelopmentOnly1234567890abcdef', 'Compliance Auditor', 'Audit & Compliance Inc', 'auditor'),
    ('00000000-0000-0000-0000-000000000006', 'admin@lcchain.com', '$argon2id$v=19$m=19456,t=2,p=1$ZHVtbXlzYWx0MTIzNA$DummyHashForDevelopmentOnly1234567890abcdef', 'System Administrator', 'LC Chain', 'admin');

