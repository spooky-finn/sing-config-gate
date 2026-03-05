import type TelegramBot from "node-telegram-bot-api";
import type { IUserRepo } from "#root/ports/user.js";

const InvationCmdOpCode = "invate_confirm";

export class InvationCmd {
	constructor(
		readonly userId: number,
		readonly status: DB.UserStatus,
	) {}

	static parse(text: string) {
		const [opcode, userId, status] = text.split("_");
		if (opcode !== InvationCmdOpCode) {
			throw Error("Wrong operation code");
		}
		if (!Object.values(DB.UserStatus).includes(status)) {
			throw Error("Invalid user status");
		}
		const uid = Number(userId);
		if (Number.isNaN(uid)) {
			throw Error("User id is NaN");
		}
		return new InvationCmd(uid, status as any);
	}

	toString() {
		return `${InvationCmdOpCode}_${this.userId}_${this.status}`;
	}
}

export class AdminService {
	constructor(
		private readonly userRepo: IUserRepo,
		readonly adminId: string,
	) {}

	async handleAdminCallback(msg: TelegramBot.Message): Promise<boolean> {
		if (msg.from?.id?.toString() !== this.adminId || !msg.text) {
			return false;
		}

		const cmd = InvationCmd.parse(msg.text);
		await this.userRepo.updateStatus(cmd.userId, cmd.status);
		return false;
	}
}
