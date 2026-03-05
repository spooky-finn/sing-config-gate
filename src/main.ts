import TelegramBot from "node-telegram-bot-api";
import { UserRepo } from "./adapters/db/user.repo.js";
import { validateAppEnv } from "./env.js";
import { AdminService } from "./service/admin.js";
import { HandleMsgService } from "./service/handle_msg.js";
import { initDB } from "./utils/db.js";
import { initLogger, log } from "./utils/log.js";

async function main() {
	// Validate environment variables using typia
	const env = validateAppEnv();
	initLogger(env.LogLevel, env.LogDisableTimestamp);

	process.on("unhandledRejection", (err) => {
		log.error(err, "Unhandled rejection");
	});

	log.info("Starting server");
	const db = initDB(env.DbLocation);

	const bot = new TelegramBot(env.TgBotToken, { polling: true });
	const userRepo = new UserRepo(db);
	const adminService = new AdminService(userRepo, env.TgAdminId);
	const handleMsgService = new HandleMsgService(bot, userRepo, adminService, {
		clientConfigEndpoint: env.ClientConfigEndpoint,
	});

	bot.on("message", (msg) => {
		handleMsgService.handleMsg(msg);
	});
}

main();
