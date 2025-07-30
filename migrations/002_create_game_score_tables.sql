-- Create game table
CREATE TABLE game (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hex_id TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME,
    
    -- Check constraints
    CHECK (hex_id GLOB '[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]'),
    CHECK (length(name) > 0 AND length(name) <= 255)
);

-- Create score table
CREATE TABLE score (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_hex_id TEXT NOT NULL,
    score TEXT NOT NULL,
    score_val REAL NOT NULL,
    user_name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    extra TEXT,
    submitted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME,
    
    -- Foreign key constraint
    FOREIGN KEY (game_hex_id) REFERENCES game(hex_id),
    
    -- Check constraints
    CHECK (length(user_name) > 0 AND length(user_name) <= 100),
    CHECK (length(user_id) > 0 AND length(user_id) <= 255)
);

-- Game table indexes
CREATE UNIQUE INDEX idx_game_hex_id ON game(hex_id);
CREATE INDEX idx_game_created_at_desc ON game(created_at DESC);
CREATE INDEX idx_game_deleted_at ON game(deleted_at) WHERE deleted_at IS NULL;

-- Score table indexes for efficient pagination and sorting
CREATE INDEX idx_score_game_score_desc ON score(game_hex_id, score_val DESC, id) WHERE deleted_at IS NULL;
CREATE INDEX idx_score_game_date_desc ON score(game_hex_id, submitted_at DESC, id) WHERE deleted_at IS NULL;
CREATE INDEX idx_score_game_user_asc ON score(game_hex_id, user_name COLLATE NOCASE ASC, id) WHERE deleted_at IS NULL;
CREATE INDEX idx_score_user_id ON score(user_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_score_deleted_at ON score(deleted_at) WHERE deleted_at IS NULL;

-- Trigger to soft-delete scores when game is soft-deleted
CREATE TRIGGER soft_delete_game_scores
    AFTER UPDATE OF deleted_at ON game
    WHEN NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL
BEGIN
    UPDATE score 
    SET deleted_at = NEW.deleted_at 
    WHERE game_hex_id = NEW.hex_id AND deleted_at IS NULL;
END;

-- Trigger to restore scores when game is restored
CREATE TRIGGER restore_game_scores
    AFTER UPDATE OF deleted_at ON game
    WHEN NEW.deleted_at IS NULL AND OLD.deleted_at IS NOT NULL
BEGIN
    UPDATE score 
    SET deleted_at = NULL 
    WHERE game_hex_id = NEW.hex_id AND deleted_at = OLD.deleted_at;
END;