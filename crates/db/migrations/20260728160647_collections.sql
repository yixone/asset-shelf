CREATE TABLE collections (
    id              BIGINT          NOT NULL,

    name            VARCHAR(255)    NOT NULL,
    description     VARCHAR(2048)   NOT NULL,

    created_at      TIMESTAMPTZ     NOT NULL,

    PRIMARY KEY (id)
);

CREATE INDEX coll_name_idx ON collections(name);

CREATE TABLE collection_assets (
    id              BIGINT          NOT NULL,

    collection_id   BIGINT          NOT NULL,
    asset_id        BIGINT          NOT NULL,

    added_at        TIMESTAMPTZ     NOT NULL,

    PRIMARY KEY (id),

    FOREIGN KEY (collection_id) REFERENCES collections(id)  ON DELETE CASCADE,
    FOREIGN KEY (asset_id)      REFERENCES assets(id)       ON DELETE CASCADE
);

CREATE INDEX coll_assets_asset_idx  ON collection_assets(asset_id);
CREATE INDEX coll_assets_coll_idx   ON collection_assets(collection_id);
