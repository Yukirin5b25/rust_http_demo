-- Your SQL goes here
CREATE TABLE "shortlink"(
	"id" SERIAL PRIMARY KEY,
	"hash" VARCHAR NOT NULL,
	"url" VARCHAR NOT NULL,
	"expire_at" TIMESTAMP NOT NULL,
	UNIQUE(hash)
);
alter table "shortlink" alter id type BIGINT;

