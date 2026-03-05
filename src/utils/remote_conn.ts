import { exec } from "node:child_process";
import { promisify } from "node:util";

const execAsync = promisify(exec);

export interface Remote {
	user: string;
	sshKeyPath: string;
	host: string;
}

interface CommandResult {
	success: boolean;
	stdout?: string;
	stderr?: string;
	error?: Error;
}

export interface RemoteCommandOptions {
	user: string;
	keyFile: string;
	host: string;
	timeout?: number;
}

export class RemoteConn {
	constructor(private readonly config: Remote) {}

	async run(cmd: string, cwd?: string) {
		try {
			const result = await this.remoteCmd(cmd, cwd, {
				user: this.config.user,
				keyFile: this.config.sshKeyPath,
				host: this.config.host,
			});
			if (!result.success) {
				throw new Error(
					`Failed to execute remote command: ${result.error?.message}`,
				);
			}
			const stdout = result.stdout?.trim();
			if (!stdout) {
				throw new Error("No output from remote command");
			}
			return stdout;
		} catch (error) {
			process.exit(1);
		}
	}

	async remoteCmd(
		command: string,
		directory: string = "",
		options: RemoteCommandOptions,
	): Promise<CommandResult> {
		const { keyFile: sshKeyPath, timeout } = options;
		let sshCommand = `ssh -i ${sshKeyPath} ${options.user}@${options.host}`;
		if (timeout) {
			sshCommand += ` -o ConnectTimeout=${timeout / 1000}`;
		}
		if (directory) {
			sshCommand += ` "cd ${directory} && ${command}"`;
		} else {
			sshCommand += ` "${command}"`;
		}
		return await this.cmd(sshCommand);
	}

	async cmd(command: string): Promise<CommandResult> {
		try {
			const { stdout, stderr } = await execAsync(command);
			return {
				success: true,
				stdout: stdout || undefined,
				stderr: stderr || undefined,
			};
		} catch (error) {
			console.error(error);
			throw error;
		}
	}
}
