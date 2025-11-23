-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Documents Table
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    ipfs_cid TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    encryption_iv TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Letter of Credits Table
CREATE TYPE lc_status AS ENUM ('draft', 'submitted', 'review', 'approved', 'rejected', 'closed');

CREATE TABLE letter_of_credits (
    id UUID PRIMARY KEY,
    applicant_id UUID NOT NULL,
    beneficiary_id UUID NOT NULL,
    issuing_bank_id UUID NOT NULL,
    advising_bank_id UUID NOT NULL,
    amount DECIMAL NOT NULL,
    currency TEXT NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL,
    status lc_status NOT NULL DEFAULT 'draft',
    document_ids UUID[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Audit Logs Table (for Audit Service, but putting here for simplicity if sharing DB, though plan said separate DB. 
-- If separate DB, this should go to a different migration file for the audit DB. 
-- I will assume separate DBs as per plan, so I will create a separate migration file for audit later.)
