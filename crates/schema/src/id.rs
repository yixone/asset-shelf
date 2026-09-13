//! Strongly typed identifiers for application entities
//!
//! All identifiers are newtypes, used to explicitly indicate the identifier type
//! and prevent the interchangeability of identifiers belonging to different entities

id_type_copy!(AssetId as snowflake::SnowflakeId);
id_type_copy!(CollectionId as snowflake::SnowflakeId);
id_type_copy!(CollectionRelId as snowflake::SnowflakeId);
id_type_copy!(MediaFileId as snowflake::SnowflakeId);

id_type!(MediaId as snowflake::SnowflakeIdStr);
