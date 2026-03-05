import fs from "node:fs";
import path from "node:path";
import {
	FileMigrationProvider,
	type Kysely,
	type MigrationResultSet,
	Migrator,
} from "kysely";
import { log } from "./log.js";

async function main(
	db: Kysely<any>,
	migrationsPath: string,
	downgrade = false,
) {
	log.info("Migrator: checking for migrations");

	const provider = new FileMigrationProvider({
		fs: fs.promises,
		migrationFolder: migrationsPath,
		path,
	});
	const migrator = new Migrator({
		db,
		provider,
	});

	let res: MigrationResultSet;
	if (downgrade) {
		res = await migrator.migrateDown();
	} else {
		res = await migrator.migrateToLatest();
	}

	const { error, results } = res;
	const action = downgrade ? "reverted" : "applied";
	results?.forEach((it) => {
		if (it.status === "Success") {
			log.info(`Migration "${it.migrationName}" was ${action}`);
		} else if (it.status === "Error") {
			log.error(`Failed to execute migration "${it.migrationName}"`);
		}
	});

	if (error) {
		log.error(error, "Failed to migrate");
		process.exit(1);
	}
}

export async function migrateDB(
	db: Kysely<any>,
	migrationsPath: string,
	downgrade = false,
) {
	try {
		await main(db, migrationsPath, downgrade);
		log.info("Migrator finished");
	} catch (err) {
		log.error(err, "Migration failed:");
		process.exit(1);
	}
}
