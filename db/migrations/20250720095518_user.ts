import { type Kysely, sql } from "kysely";

export async function up(db: Kysely<DB.Schema>): Promise<void> {
	await db.schema
		.createTable("user")
		.addColumn("id", "integer", (col) => col.primaryKey())
		.addColumn("username", "text", (col) => col.notNull())
		.addColumn("auth_key", "text", (col) => col.notNull())
		.addColumn("created_at", "timestamp", (col) =>
			col.notNull().defaultTo(sql`CURRENT_TIMESTAMP`),
		)
		.addColumn("status", "text", (col) => col.notNull())
		.execute();
}

export async function down(db: Kysely<DB.Schema>): Promise<void> {
	await db.schema.dropTable("user").execute();
}
