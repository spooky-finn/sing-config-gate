import { z } from "zod";

export const appEnvSchema = z.object({
	// Telegram bot token (required)
	TgBotToken: z.string(),
	// Telegram admin user ID (required)
	TgAdminId: z.string(),
	// Client configuration endpoint URL (required)
	ClientConfigEndpoint: z.string(),
	// Database file location (required)
	DbLocation: z.string(),
	// Log level (optional, defaults to "info")
	LogLevel: z.string().optional(),
	// Whether to disable timestamp in logs (optional)
	LogDisableTimestamp: z.string().optional(),
});

// Deployment environment variables
export const deployEnvSchema = z.object({
	// Deployment host (required)
	DeployHost: z.string(),
	// SSH key file path (required)
	DeployKeyfile: z.string(),
	// SSH user (required)
	DeployUser: z.string(),
	// Command to execute on remote host (required)
	DeployCommand: z.string(),
	// Working directory on remote host (required)
	DeployCwd: z.string(),
});

export type AppEnv = z.infer<typeof appEnvSchema>;
export type DeployEnv = z.infer<typeof deployEnvSchema>;

// Validates and returns application environment variables
export function validateAppEnv(): AppEnv {
	const env = {
		TgBotToken: process.env.TG_BOT_TOKEN!,
		TgAdminId: process.env.TG_ADMIN_ID!,
		ClientConfigEndpoint: process.env.CLIENT_CONFIG_ENDPOINT!,
		DbLocation: process.env.DB_LOCATION!,
		LogLevel: process.env.LOG_LEVEL,
		LogDisableTimestamp: process.env.LOG_DISABLE_TIMESTAMP,
	} satisfies AppEnv;
	const result = appEnvSchema.safeParse(env);
	if (!result.success) {
		console.error("Environment validation failed:");
		result.error.issues.forEach((error) => {
			console.error(
				`- ${error.path.join(".")}: ${error.message} (got: ${error.input})`,
			);
		});
		process.exit(1);
	}
	return result.data;
}

/**
 * Validates and returns deployment environment variables
 */
export function validateDeployEnv(): DeployEnv {
	const env = {
		DeployHost: process.env.DEPLOY_HOST,
		DeployKeyfile: process.env.DEPLOY_KEYFILE,
		DeployUser: process.env.DEPLOY_USER,
		DeployCommand: process.env.DEPLOY_COMMAND,
		DeployCwd: process.env.DEPLOY_CWD,
	};
	const result = deployEnvSchema.safeParse(env);
	if (!result.success) {
		console.error("Deployment environment validation failed:");
		result.error.issues.forEach((error) => {
			console.error(
				`- ${error.path.join(".")}: ${error.message} (got: ${error.input})`,
			);
		});
		process.exit(1);
	}
	return result.data;
}
