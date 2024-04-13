const util = require('node:util');
const exec = util.promisify(require('node:child_process').exec);
const fs = require('node:fs/promises');

const get_whoami = async () => {
	let whoami = await exec("whoami");
	if (whoami.stderr) {
		console.error(`Error getting current user | ${whoami.stderr}`);
		process.exit(1);
	}
	//strip newlines
	return (whoami.stdout.indexOf("\n") != -1) ? whoami.stdout.substring(0, whoami.stdout.indexOf("\n")) : whoami.stdout
};

const get_repo_root = async () => {
	let repo_root = await exec("git rev-parse --show-toplevel");
	if (repo_root.stderr) {
		console.error(`Error getting repo root | ${repo_root.stderr}`);
		process.exit(1);
	}
	return repo_root.stdout = (repo_root.stdout.indexOf("\n") != -1) ? repo_root.stdout.substring(0, repo_root.stdout.indexOf("\n")) : repo_root.stdout
};

const get_lookup = async () => {
	let f = await fs.readFile('./lookup.json', {encoding: "utf8"});
	let obj = JSON.parse(f);
	return obj
};

(async () => {

	const username = await get_whoami();
	const repo_root = await get_repo_root();
	const lookup = await get_lookup();

	for (let x = 0; x < lookup.length; x++) {
		// If the current repo path starts with a known lookup entry, set git config.email to corresp email
		if (repo_root.startsWith(`/home/${username}/${lookup[x].dir}`)) {
			const res = await exec (`git config user.email "${lookup[x].email}"`);
			if (res.stderr) {
				console.error(`Error setting git config user email | ${res.stderr}`);
				process.exit(1);
			}
			console.log(`Git config email updated to ${lookup[x].email}`);
			process.exit(0);
		}
	}

	console.log(`Directory doesnt match any known entries. Leaving git config email as default`);
	process.exit(0);
})();


