import type { Kysely } from "kysely";

export type DataStore = Kysely<DB.Schema>;

export interface User {
	id: number;
	username: string;
	status: DB.UserStatus;
	auth_key: string;
	created_at: string;
}

export interface IUserRepo {
	select(userId: number): Promise<User | null>;
	insert(user: User): Promise<void>;
	getUsersByStatus(status: DB.UserStatus): Promise<User[]>;
	updateStatus(userId: number, status: DB.UserStatus): Promise<void>;
}
