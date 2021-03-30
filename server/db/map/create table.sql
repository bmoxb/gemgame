CREATE TABLE IF NOT EXISTS map (
    single_row BOOLEAN PRIMARY KEY DEFAULT TRUE,
    seed INTEGER NOT NULL,
    CONSTRAINT single_row_constraint CHECK (single_row)
)
