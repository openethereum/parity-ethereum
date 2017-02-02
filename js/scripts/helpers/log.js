import chalk from 'chalk';

// INFO Logging helper
export function info (log) {
  console.log(chalk.blue(`INFO:\t${log}`));
}

// WARN Logging helper
export function warn (log) {
  console.warn(chalk.yellow(`WARN:\t${log}`));
}

// ERROR Logging helper
export function error (log) {
  console.error(chalk.red(`ERROR:\t${log}`));
}
