-- Initial schema for LEADR API (PostgreSQL)

-- Create game table
CREATE TABLE game (
    id BIGSERIAL PRIMARY KEY,
    hex_id VARCHAR(6) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,

    -- Check constraints
    CHECK (hex_id ~ '^[0-9a-z]{6}$'),
    CHECK (LENGTH(name) > 0 AND LENGTH(name) <= 255)
);

-- Create score table
CREATE TABLE score (
    id BIGSERIAL PRIMARY KEY,
    game_hex_id VARCHAR(6) NOT NULL,
    score TEXT NOT NULL,
    score_val DOUBLE PRECISION NOT NULL,
    user_name VARCHAR(100) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    extra TEXT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,

    -- Foreign key constraint
    FOREIGN KEY (game_hex_id) REFERENCES game(hex_id) ON DELETE CASCADE,

    -- Check constraints
    CHECK (LENGTH(user_name) > 0 AND LENGTH(user_name) <= 100),
    CHECK (LENGTH(user_id) > 0 AND LENGTH(user_id) <= 255)
);

-- Game table indexes
CREATE UNIQUE INDEX idx_game_hex_id ON game(hex_id);
CREATE INDEX idx_game_created_at_desc ON game(created_at DESC);
CREATE INDEX idx_game_deleted_at ON game(deleted_at) WHERE deleted_at IS NULL;

-- Score table indexes for efficient pagination and sorting
CREATE INDEX idx_score_game_score_desc ON score(game_hex_id, score_val DESC, id) WHERE deleted_at IS NULL;
CREATE INDEX idx_score_game_date_desc ON score(game_hex_id, submitted_at DESC, id) WHERE deleted_at IS NULL;
CREATE INDEX idx_score_game_user_asc ON score(game_hex_id, LOWER(user_name), id) WHERE deleted_at IS NULL;
CREATE INDEX idx_score_user_id ON score(user_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_score_deleted_at ON score(deleted_at) WHERE deleted_at IS NULL;

-- Function to soft-delete scores when game is soft-deleted
CREATE OR REPLACE FUNCTION soft_delete_game_scores()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        UPDATE score
        SET deleted_at = NEW.deleted_at
        WHERE game_hex_id = NEW.hex_id AND deleted_at IS NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to soft-delete scores when game is soft-deleted
CREATE TRIGGER soft_delete_game_scores_trigger
    AFTER UPDATE OF deleted_at ON game
    FOR EACH ROW
    EXECUTE FUNCTION soft_delete_game_scores();

-- Function to restore scores when game is restored
CREATE OR REPLACE FUNCTION restore_game_scores()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NULL AND OLD.deleted_at IS NOT NULL THEN
        UPDATE score
        SET deleted_at = NULL
        WHERE game_hex_id = NEW.hex_id AND deleted_at = OLD.deleted_at;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to restore scores when game is restored
CREATE TRIGGER restore_game_scores_trigger
    AFTER UPDATE OF deleted_at ON game
    FOR EACH ROW
    EXECUTE FUNCTION restore_game_scores();
