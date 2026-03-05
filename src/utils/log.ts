import { pino } from "pino";

let timestampDisabled = false;
let logInstance: ReturnType<typeof pino>;

export function initLogger(level = "info", disableTimestamp?: string) {
	if (!logInstance) {
		console.log("Initializing logger with level:", level);
		timestampDisabled = disableTimestamp === "true";
		const ignoreScript = ["pid", "hostname", "level-label"];

		if (timestampDisabled) {
			ignoreScript.push("time");
		}

		logInstance = pino({
			level,
			transport: {
				targets: [
					{
						target: "pino-pretty",
						options: {
							colorize: true,
							ignore: ignoreScript.join(","),
							translateTime: "dd-mm-yy HH:MM:ss",
						},
					},
				],
			},
			timestamp: pino.stdTimeFunctions.isoTime,
		});
	}
	return logInstance;
}

// Export proxy object that warns if not initialized
export const log = new Proxy({} as ReturnType<typeof pino>, {
	get(_target, prop) {
		if (!logInstance) {
			throw new Error("Logger not initialized. Call initLogger() first.");
		}
		return logInstance[prop as keyof ReturnType<typeof pino>];
	},
});
