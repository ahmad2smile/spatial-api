-- Add migration script here

-- Prep Database
CREATE EXTENSION IF NOT EXISTS postgis;

-- Sensors Table
CREATE TABLE IF NOT EXISTS sensors (
    name TEXT PRIMARY KEY,
    lat DOUBLE PRECISION NOT NULL CHECK (lat BETWEEN -90 AND 90),
    long DOUBLE PRECISION NOT NULL CHECK (long BETWEEN -90 AND 90),
    position GEOGRAPHY(Point, 4326) GENERATED ALWAYS AS (ST_SetSRID(ST_MakePoint(long, lat), 4326)::geography) STORED
);

CREATE TABLE IF NOT EXISTS tags (
    sensor TEXT NOT NULL REFERENCES sensors(name) ON DELETE CASCADE ON UPDATE CASCADE,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    value TEXT NOT NULL CHECK (trim(value) <> ''),
    PRIMARY KEY (sensor, name)
);

CREATE INDEX sensors_pos_idx ON sensors USING GIST (position);
