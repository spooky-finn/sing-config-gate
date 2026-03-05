import { Database } from "bun:sqlite";
import fs from "node:fs";
import path from "node:path";
import { Kysely } from "kysely";
import { BunSqliteDialect } from "kysely-bun-sqlite";
import { migrateDB } from "./migrate.js";

export function initDB(dbLocation: string) {
	const isExist = fs.existsSync(dbLocation);
	if (!isExist) {
		fs.mkdirSync(path.dirname(dbLocation), { recursive: true });
	}

	const bunDb = new Database(dbLocation);
	const db = new Kysely<DB.Schema>({
		dialect: new BunSqliteDialect({
			database: bunDb,
		}),
	});

	const migrationsLocation = path.join(
		__dirname,
		"..",
		"..",
		"db",
		"migrations",
	);
	migrateDB(db, migrationsLocation);
	return db;
}
