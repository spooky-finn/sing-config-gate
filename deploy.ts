import { validateDeployEnv } from "./src/env.js";
import { type Remote, RemoteConn } from "./src/utils/remote_conn.js";
(async () => {
	// Validate environment variables using typia
	const env = validateDeployEnv();
	const config: Remote = {
		host: env.DeployHost,
		sshKeyPath: env.DeployKeyfile,
		user: env.DeployUser,
	};
	const result = await new RemoteConn(config).run(
		env.DeployCommand,
		env.DeployCwd,
	);
	console.log(result);
})();
