# TopClaw

TopClaw is a Rust-based AI agent runtime for local and remote AI workflows.

## Quick Start

### Ubuntu

```bash
git clone https://github.com/jackfly8/TopClaw.git
cd TopClaw
./bootstrap.sh --install-system-deps --install-rust --prefer-prebuilt
topclaw status
topclaw agent -m "Hello!"
```

This path installs standard prerequisites, installs Rust when missing, prefers a prebuilt binary first, and starts onboarding automatically.

### macOS (Apple Silicon)

Install Apple developer tools first:

```bash
xcode-select --install
```

Clone and bootstrap TopClaw:

```bash
git clone https://github.com/jackfly8/TopClaw.git
cd TopClaw
./bootstrap.sh --install-system-deps --install-rust --prefer-prebuilt
topclaw status
topclaw agent -m "Hello!"
```

This path installs standard prerequisites, installs Rust when missing, prefers a prebuilt binary first, and starts onboarding automatically.

## What Bootstrap Does

Recommended first-run command:

```bash
./bootstrap.sh --install-system-deps --install-rust --prefer-prebuilt
```

What those flags do:

1. install missing system dependencies when possible
2. install Rust if it is not already present
3. try a prebuilt `topclaw` binary first, then fall back to source build if needed
4. start the onboarding wizard

During onboarding, the default path is now:

- choose your AI provider
- enter the provider API key if needed
- choose a channel such as Telegram or Discord
- enter the channel token and allowed user info

Everything else can be changed later in `config.toml`.

## Fast Path

If you already have an API key and want a minimal setup:

```bash
topclaw onboard --api-key "sk-..." --provider openrouter
```

## First Commands

After onboarding, these are the most useful first commands:

```bash
topclaw status
topclaw agent -m "Hello!"
topclaw gateway
```

## Uninstall

To remove the TopClaw binary and service artifacts:

```bash
./topclaw_uninstall.sh
```

To remove TopClaw completely, including `~/.topclaw` config, logs, auth profiles, and workspace data:

```bash
./topclaw_uninstall.sh --purge
```

## Documentation Map

- Getting started: [`docs/getting-started/README.md`](docs/getting-started/README.md)
- Commands and config: [`docs/reference/README.md`](docs/reference/README.md)
- Operations and troubleshooting: [`docs/operations/README.md`](docs/operations/README.md)
- Full docs hub: [`docs/README.md`](docs/README.md)

### Other Platforms

- Windows: run `.\bootstrap.ps1`
- Lower-resource machines: `./bootstrap.sh --prefer-prebuilt`
