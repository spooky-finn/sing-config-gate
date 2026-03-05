import type { DataStore, IUserRepo, User } from "#root/ports/user.js";

export class UserRepo implements IUserRepo {
	constructor(private readonly db: DataStore) {}

	async select(id: number): Promise<User | null> {
		const user = await this.db
			.selectFrom("user")
			.where("id", "=", id)
			.selectAll()
			.executeTakeFirst();
		if (!user) {
			return null;
		}
		return user;
	}

	async insert(user: User): Promise<void> {
		await this.db.insertInto("user").values(user).execute();
	}

	async getUsersByStatus(status: DB.UserStatus): Promise<User[]> {
		return this.db
			.selectFrom("user")
			.where("status", "=", status)
			.selectAll()
			.execute();
	}

	async updateStatus(id: number, status: DB.UserStatus): Promise<void> {
		await this.db
			.updateTable("user")
			.set({ status })
			.where("id", "=", id)
			.execute();
	}
}
