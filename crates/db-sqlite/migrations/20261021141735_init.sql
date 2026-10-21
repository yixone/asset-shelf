CREATE TABLE file_groups (
    id                  VARCHAR(32)     NOT NULL,

    PRIMARY KEY (id)
);

CREATE TABLE files (
    group_id            VARCHAR(32)     NOT NULL,
    variant             VARCHAR(64)     NOT NULL,

    created_at          TIMESTAMPTZ     NOT NULL,

    path                VARCHAR(1024)   NOT NULL,

    size_bytes          BIGINT          NOT NULL,
    mime_type           VARCHAR(128)    NOT NULL,

    duration_ms         BIGINT,

    PRIMARY KEY (group_id, variant),

    FOREIGN KEY (group_id) REFERENCES file_groups(id) ON DELETE CASCADE
);

CREATE TABLE assets (
    id                  BIGINT          NOT NULL,

    parent_id           BIGINT,
    duplicate_of        BIGINT,

    file_group_id       VARCHAR(32)     NOT NULL,
    checksum            BLOB            NOT NULL,
    original_file_name  VARCHAR(255),
    original_mime_kind  VARCHAR(128)    NOT NULL,

    state               VARCHAR(32)     NOT NULL,
    state_failure       VARCHAR(512),
    is_offline          BOOLEAN         NOT NULL,

    created_at          TIMESTAMPTZ     NOT NULL,
    updated_at          TIMESTAMPTZ,
    deleted_at          TIMESTAMPTZ,

    PRIMARY KEY (id),

    FOREIGN KEY (file_group_id) REFERENCES file_groups(id),

    CONSTRAINT valid_parent     CHECK (id <> parent_id), -- Not the parent itself
    FOREIGN KEY (parent_id)     REFERENCES assets(id)       ON DELETE SET NULL,

    CONSTRAINT valid_duplicate  CHECK (id <> duplicate_of), -- Not a duplicate itself
    FOREIGN KEY (duplicate_of)  REFERENCES assets(id)       ON DELETE SET NULL
);

CREATE INDEX idx_assets_parent      ON assets(parent_id);
CREATE INDEX idx_assets_checksum    ON assets(checksum);
CREATE INDEX idx_assets_state       ON assets(state);

CREATE TABLE assets_features (
    id                  BIGINT          NOT NULL,

    accent_color        INTEGER,

    width               INTEGER,
    height              INTEGER,

    p_hash              BIGINT,
    a_hash              BIGINT,

    PRIMARY KEY (id),
    FOREIGN KEY (id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE assets_meta (
    id                  BIGINT          NOT NULL,

    title               VARCHAR(255),
    caption             VARCHAR(512),
    source_url          VARCHAR(512),

    PRIMARY KEY (id),
    FOREIGN KEY (id) REFERENCES assets(id) ON DELETE CASCADE
);
