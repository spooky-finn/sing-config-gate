import { logger } from "#root/ioc.js";
import type { IUserRepo } from "#root/ports/user.js";
import { UserStatus } from "db/enums";
import type TelegramBot from "node-telegram-bot-api";

export class InvationCmd {
	private static opCode = "invate-confirm";
	constructor(
		readonly userId: number,
		readonly status: UserStatus,
	) { }

	static parse(text: string) {
		const data = JSON.parse(text);
		if (data.opcode !== InvationCmd.opCode) {
			throw Error(`InvationCmd: Wrong operation code: received ${data.opcode}`);
		}
		if (!Object.values(UserStatus).includes(data.status)) {
			throw Error("Invalid user status");
		}
		if (typeof data.userId !== "number" || Number.isNaN(data.userId)) {
			throw Error("Invalid user id");
		}
		return new InvationCmd(data.userId, data.status);
	}

	toString() {
		return JSON.stringify({
			opcode: InvationCmd.opCode,
			userId: this.userId,
			status: this.status,
		});
	}
}

export class AdminService {
	constructor(
		private readonly userRepo: IUserRepo,
		readonly adminId: string,
	) { }

	isAdminCallback(msg: TelegramBot.CallbackQuery): InvationCmd | false {
		if (msg.from?.id?.toString() !== this.adminId || !msg.data) {
			return false;
		}

		try {
			return InvationCmd.parse(msg.data);
		} catch (error) {
			logger.error(error, "fail to parse admin command");
			return false;
		}
	}

	async handleAdminCallback(cmd: InvationCmd) {
		await this.userRepo.setStatus(cmd.userId, cmd.status);
		logger.info(cmd, "adming callback handled");
	}
}
