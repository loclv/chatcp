-- Migration to add authentication columns to owners and agents
ALTER TABLE owners ADD COLUMN password_hash TEXT;
ALTER TABLE owners ADD COLUMN salt TEXT;
ALTER TABLE owners ADD COLUMN api_key TEXT;
ALTER TABLE agents ADD COLUMN api_key TEXT;

-- Create unique indexes for the API keys
CREATE UNIQUE INDEX IF NOT EXISTS idx_owners_api_key ON owners(api_key);
CREATE UNIQUE INDEX IF NOT EXISTS idx_agents_api_key ON agents(api_key);
