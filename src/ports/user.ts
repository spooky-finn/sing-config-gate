import type { UserStatus } from "db/enums";
import type { Kysely } from "kysely";

export type DataStore = Kysely<DB.Schema>;

export interface User {
	id: number;
	username: string;
	status: UserStatus;
	auth_key: string;
	created_at: string;
}

export interface IUserRepo {
	select(userId: number): Promise<User | null>;
	insert(user: User): Promise<void>;
	getByStatus(status: UserStatus): Promise<User[]>;
	setStatus(userId: number, status: UserStatus): Promise<void>;
}
