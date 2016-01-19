#!/usr/bin/env node

'use strict';

const fs = require('fs');
const exec = require('child_process').exec;

// First run 
// $ cargo build |& grep "warning: missing documentation" > missingdocs
const lines = fs.readFileSync('./missingdocs', 'utf8').split('\n');
const pattern = /(.+):([0-9]+):([0-9]+)/;

const errors = lines.map((line) => {
	const parts = line.match(pattern);
	if (!parts || parts.length < 4) {
		console.error('Strange line: ' + line);
		return;
	}
	return {
		path: parts[1],
		line: parts[2],
		col: parts[3]
	};
}).filter((line) => line);

const indexed = errors.reduce((index, error) => {
	if (!index[error.path]) {
		index[error.path] = [];
	}
	index[error.path].push(error);

	return index;
}, {});

for (let path in indexed) {
	let file = fs.readFileSync(path, 'utf8').split('\n');
	let error = indexed[path].sort((a, b) => b.line - a.line);
	let next = () => {
		let err = error.shift();
		if (!err) {
			fs.writeFileSync(path, file.join('\n'), 'utf8');
			return;
		}
		// Process next error
		let tabs = Array(parseInt(err.col, 10)).join('\t');
		get_user(path, err.line, (user) => {
			let line = err.line - 1;
			let comment = `${tabs}/// TODO [${user}] Please document me`;
			if (file[line] !== comment) {
				file.splice(line, 0, comment);
			}
			next();
		});
	};
	next();
}

function get_user (path, line, cb) {
	exec(`git blame ${path}`, (err, stdout, stderr) => {
		if (err) throw err;
		const l = stdout.split('\n')[line];
		const user = l.match(/\(([a-zA-Z ]+?)\s+2/);
		cb(user[1]);
	});
}
