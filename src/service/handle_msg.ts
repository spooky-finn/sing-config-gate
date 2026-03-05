import type { IUserRepo, User } from "#root/ports/user.js";
import { UserStatus } from "db/enums.js";
import type TelegramBot from "node-telegram-bot-api";
import { randomBytes } from "node:crypto";
import { type AdminService, InvationCmd } from "./admin.js";

interface Config {
	clientConfigEndpoint: string;
}

export class HandleMsgService {
	constructor(
		private readonly bot: TelegramBot,
		private readonly userRepo: IUserRepo,
		private readonly adminService: AdminService,
		private readonly conf: Config,
	) {}

	async handleCallback(msg: TelegramBot.CallbackQuery) {
		if (msg.from?.is_bot) return;
		const isAdminCmd = this.adminService.isAdminCallback(msg);
		if (!isAdminCmd) return;

		await this.adminService.handleAdminCallback(isAdminCmd);
	}

	async handleMsg(msg: TelegramBot.Message) {
		if (msg.from?.is_bot) return;
		const userId = msg.from?.id;
		if (!userId) throw new Error("User ID is required");

		const user = await this.userRepo.select(userId);
		if (!user) {
			await this.register(msg.from!);
		} else {
			await this.sendStatus(user);
		}
	}

	async register(user: TelegramBot.User) {
		const auth_key = randomBytes(32).toString("hex");
		await this.userRepo.insert({
			id: user.id,
			username: user.username || "",
			status: UserStatus.New,
			auth_key,
			created_at: new Date().toISOString(),
		});
		this.sendMessageToAdmin(user, "Новая заявка");
	}

	async sendStatus(user: User) {
		if (user.status == UserStatus.New) {
			this.bot.sendMessage(
				user.id,
				`Администратор скоро рассмотрит вашу заявку`,
			);
			return;
		}
		if (user.status == UserStatus.Accepted) {
			const configURL = this.getConfigLink(user);
			this.bot.sendMessage(
				user.id,
				`Вам одобрен доступ. Ваш файл конфигурации доступен по ссылке: ${configURL}`,
			);
			return;
		}
		if (user.status == UserStatus.Rejected) {
			this.bot.sendMessage(user.id, `Ваша заявка отклонена`);
			return;
		}
	}

	private getConfigLink(user: DB.User) {
		const u = new URL(user.id.toString(), this.conf.clientConfigEndpoint);
		return u.toString();
	}

	private sendMessageToAdmin(user: TelegramBot.User, message: string) {
		const msg = `Новая заявка от ${user.username ?? user.id}: ${message}`;
		this.bot.sendMessage(this.adminService.adminId, msg, {
			reply_markup: {
				inline_keyboard: [
					[
						{
							text: "Accept",
							callback_data: new InvationCmd(
								user.id,
								UserStatus.Accepted,
							).toString(),
						},
						{
							text: "Reject",
							callback_data: new InvationCmd(
								user.id,
								UserStatus.Rejected,
							).toString(),
						},
					],
				],
				one_time_keyboard: true,
				resize_keyboard: true,
			},
		});
	}
}
