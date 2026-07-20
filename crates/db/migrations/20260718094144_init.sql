CREATE TABLE assets (
    id              BIGINT          NOT NULL,
    state           VARCHAR(32)     NOT NULL,
    media_id        VARCHAR(32)     NOT NULL,
    
    created_at      TIMESTAMPTZ     NOT NULL,
    deleted_at      TIMESTAMPTZ,

    title           VARCHAR(255),
    caption         VARCHAR(512),
    source_url      VARCHAR(512),

    width           INTEGER,
    height          INTEGER,

    accent_color    INTEGER,

    PRIMARY KEY (id),
    FOREIGN KEY (media_id) REFERENCES media(id)
);

CREATE INDEX idx_asset_media ON assets(media_id);

CREATE TABLE asset_features (
    asset_id        BIGINT          NOT NULL,

    p_hash          BIGINT,
    a_hash          BIGINT,

    aspect_ratio    REAL,

    PRIMARY KEY (asset_id),
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE media (
    id              VARCHAR(32)     NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL,

    PRIMARY KEY (id)
);

CREATE TABLE media_files (
    id              VARCHAR(32)     NOT NULL,
    storage_path    VARCHAR(512)    NOT NULL,

    media_id        VARCHAR(32)     NOT NULL,
    variant         VARCHAR(64)     NOT NULL,

    created_at      TIMESTAMPTZ     NOT NULL,

    size_bytes      BIGINT          NOT NULL,
    mimetype        VARCHAR(128)    NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE    
);

CREATE INDEX idx_media_files_media          ON media_files(media_id);
CREATE INDEX idx_media_files_media_variant  ON media_files(media_id, variant);
