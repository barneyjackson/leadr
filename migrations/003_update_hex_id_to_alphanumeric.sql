-- Update hex_id constraint to allow alphanumeric characters instead of just hex
-- SQLite doesn't support ALTER TABLE to modify CHECK constraints, so we need to recreate the table

-- Create new table with updated constraint
CREATE TABLE game_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hex_id TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME,
    
    -- Updated check constraints - allow alphanumeric characters
    CHECK (hex_id GLOB '[0-9a-z][0-9a-z][0-9a-z][0-9a-z][0-9a-z][0-9a-z]'),
    CHECK (length(name) > 0 AND length(name) <= 255)
);

-- Copy data from old table
INSERT INTO game_new (id, hex_id, name, description, created_at, updated_at, deleted_at)
SELECT id, hex_id, name, description, created_at, updated_at, deleted_at
FROM game;

-- Drop old table
DROP TABLE game;

-- Rename new table
ALTER TABLE game_new RENAME TO game;

-- Recreate indexes
CREATE UNIQUE INDEX idx_game_hex_id ON game(hex_id);
CREATE INDEX idx_game_created_at_desc ON game(created_at DESC);
CREATE INDEX idx_game_deleted_at ON game(deleted_at) WHERE deleted_at IS NULL;

-- Recreate triggers
CREATE TRIGGER soft_delete_game_scores 
AFTER UPDATE OF deleted_at ON game 
WHEN NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL
BEGIN
    UPDATE score 
    SET deleted_at = NEW.deleted_at 
    WHERE game_hex_id = NEW.hex_id AND deleted_at IS NULL;
END;

CREATE TRIGGER restore_game_scores 
AFTER UPDATE OF deleted_at ON game 
WHEN NEW.deleted_at IS NULL AND OLD.deleted_at IS NOT NULL
BEGIN
    UPDATE score 
    SET deleted_at = NULL 
    WHERE game_hex_id = NEW.hex_id AND deleted_at = OLD.deleted_at;
END;