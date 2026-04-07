#!/usr/bin/env bun

import { fileURLToPath } from 'bun';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';

function color(text: string, value: string) {
  const start = Bun.color(value, 'ansi') ?? '';
  const reset = start ? '\x1b[0m' : '';
  return `${start}${text}${reset}`;
}

function bold(text: string) {
  return `\x1b[1m${text}\x1b[22m`;
}

function dim(text: string) {
  return `\x1b[2m${text}\x1b[22m`;
}

const args = process.argv.slice(2);

function help() {
  const commands = [
    {
      name: 'run',
      desc: 'Run a JS/TS file in uzumaki runtime',
      args: './index.tsx [...args]',
    },
    {
      name: 'pack',
      desc: 'Package a bundled app into a standalone executable',
      args: '[dist]',
    },
  ];

  const nameWidth = Math.max(...commands.map((cmd) => cmd.name.length));
  const argsWidth = Math.max(...commands.map((cmd) => cmd.args?.length ?? 0));

  const commandLines = commands
    .map((cmd) => {
      const name = bold(color(cmd.name.padEnd(nameWidth), '#60a5fa'));
      const argText = dim((cmd.args ?? '').padEnd(argsWidth));
      return `  ${name}  ${argText}  ${cmd.desc}`;
    })
    .join('\n');

  console.log(
    [
      `${bold(color('Uzumaki', '#60a5fa'))} Desktop UI Framework`,
      '',
      `${bold('Usage:')} uzumaki <command> ${dim('[...flags] [...args]')}`,
      '',
      `${bold('Commands:')}`,
      commandLines,
    ].join('\n'),
  );
}

// for uzumaki developement
const BIN_FOLDER = path.resolve(
  path.dirname(fileURLToPath(new URL(import.meta.url))),
  '../../../target',
);

const require = createRequire(import.meta.url);

function getBinaryName() {
  switch (process.platform) {
    case 'win32':
      return 'uzumaki.exe';
    default:
      return 'uzumaki';
  }
}

function resolveTargetBinaryPath() {
  const binaryName = getBinaryName();
  const candidates = [
    path.join(BIN_FOLDER, 'release', binaryName),
    path.join(BIN_FOLDER, 'debug', binaryName),
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return candidates[0]!;
}

function getPlatformPackageName() {
  return `@uzumaki-apps/${process.platform}-${process.arch}`;
}

function resolvePackagedBinaryPath() {
  const packageName = getPlatformPackageName();
  try {
    const mod = require(packageName) as
      | string
      | { default?: string; binaryPath?: string; getBinaryPath?: () => string };

    if (typeof mod === 'string') {
      return mod;
    }

    if (typeof mod?.getBinaryPath === 'function') {
      return mod.getBinaryPath();
    }

    if (typeof mod?.binaryPath === 'string') {
      return mod.binaryPath;
    }

    if (typeof mod?.default === 'string') {
      return mod.default;
    }
  } catch {
    return null;
  }

  return null;
}

function resolveRuntimeBinaryPath() {
  const packagedBinaryPath = resolvePackagedBinaryPath();
  if (packagedBinaryPath && fs.existsSync(packagedBinaryPath)) {
    return packagedBinaryPath;
  }

  return resolveTargetBinaryPath();
}

async function run(entryPoint: string, extraArgs: string[] = []) {
  const binaryPath = resolveRuntimeBinaryPath();

  if (!fs.existsSync(binaryPath)) {
    console.error(
      [
        color('error:', '#ef4444'),
        `native binary not found at ${dim(binaryPath)}`,
      ].join(' '),
    );
    return 1;
  }

  const child = Bun.spawn([binaryPath, entryPoint, ...extraArgs], {
    stdin: 'inherit',
    stdout: 'inherit',
    stderr: 'inherit',
  });

  return await child.exited;
}

async function main() {
  if (!args.length) {
    help();
    return 0;
  }

  const cmd = args[0]!;
  switch (cmd) {
    case 'run': {
      const entryPoint = args[1];
      if (!entryPoint) {
        console.error(`${color('error:', '#ef4444')} entry point not provided`);
        console.error(`usage: ${dim('uzumaki run <entrypoint> [...args]')}`);
        return 1;
      }
      return await run(entryPoint, args.slice(2));
    }

    case 'pack': {
      const binaryPath = resolveRuntimeBinaryPath();
      if (!fs.existsSync(binaryPath)) {
        console.error(
          [
            color('error:', '#ef4444'),
            `native binary not found at ${dim(binaryPath)}`,
          ].join(' '),
        );
        return 1;
      }
      const child = Bun.spawn([binaryPath, 'pack', ...args.slice(1)], {
        stdin: 'inherit',
        stdout: 'inherit',
        stderr: 'inherit',
      });
      return await child.exited;
    }

    default: {
      return await run(cmd, args.slice(1));
    }
  }
}

const exitCode = await main();
if (exitCode !== 0) {
  process.exit(exitCode);
}
