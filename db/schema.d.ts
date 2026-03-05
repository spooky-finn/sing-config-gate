import { Selectable } from "kysely";

declare global {
	namespace DB {
		interface Schema {
			user: UserTable;
		}

		enum UserStatus {
			New = 0,
			Accepted = 1,
			Rejected = 2,
		}

		export interface UserTable {
			id: number;
			username: string;
			auth_key: string;
			created_at: string;
			status: UserStatus;
		}

		export type User = Selectable<UserTable>;
	}
}
