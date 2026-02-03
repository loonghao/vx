# Node.js Ecosystem

vx provides comprehensive support for the Node.js ecosystem.

## Supported Tools

| Tool | Description |
|------|-------------|
| `node` | Node.js runtime |
| `npm` | Node.js package manager |
| `npx` | Node.js package runner |
| `pnpm` | Fast, disk space efficient package manager |
| `yarn` | JavaScript package manager (all versions) |
| `bun` | All-in-one JavaScript runtime |

## Node.js

### Installation

```bash
vx install node 20
vx install node lts
vx install node latest
```

### Version Specifiers

```bash
node 20          # Latest 20.x.x
node 20.10       # Latest 20.10.x
node 20.10.0     # Exact version
node lts         # Latest LTS
node latest      # Latest stable
```

### Usage

```bash
vx node --version
vx node script.js
vx node -e "console.log('Hello')"
```

## npm

npm is included with Node.js.

```bash
vx npm install
vx npm run build
vx npm test
```

## npx

npx is included with Node.js.

```bash
vx npx create-react-app my-app
vx npx eslint .
vx npx prettier --write .
```

## pnpm

```bash
# Install pnpm
vx install pnpm latest

# Usage
vx pnpm install
vx pnpm run build
vx pnpm add react
```

## Yarn

vx provides seamless support for **all Yarn versions** - both Yarn 1.x (Classic) and Yarn 2.x+ (Berry/Modern).

### Yarn Version Support

| Version | Type | Installation Method |
|---------|------|---------------------|
| 1.x | Classic | Direct download from GitHub releases |
| 2.x | Berry | Via corepack (auto-managed) |
| 3.x | Berry | Via corepack (auto-managed) |
| 4.x | Berry | Via corepack (auto-managed) |

### Yarn 1.x (Classic)

Yarn 1.x is directly downloaded and installed by vx:

```bash
# Install Yarn 1.x
vx install yarn 1.22.22

# Check version
vx yarn@1.22.22 --version
# Output: 1.22.22

# Usage
vx yarn install
vx yarn build
vx yarn add react
```

### Yarn 2.x+ (Berry/Modern)

Yarn 2.x and later versions are **automatically managed via corepack** - vx handles this transparently:

```bash
# Use any Yarn 2.x+ version directly
vx yarn@2.4.3 --version    # Berry 2.x
# Output: 2.4.3

vx yarn@3.6.0 --version    # Berry 3.x
# Output: 3.6.0

vx yarn@4.0.0 --version    # Berry 4.x
# Output: 4.0.0

vx yarn@4.12.0 --version   # Latest Berry
# Output: 4.12.0
```

**How it works:**

1. When you request Yarn 2.x+, vx automatically:
   - Ensures Node.js (with corepack) is installed
   - Enables corepack if not already enabled
   - Prepares the specific Yarn version via `corepack prepare yarn@<version> --activate`
   - Executes Yarn through corepack

2. You don't need to manually enable corepack or run any setup commands - vx handles everything transparently.

### Using Yarn in Projects

```bash
# Create a new project with Yarn Berry
mkdir my-project && cd my-project
vx yarn@4.0.0 init

# Install dependencies
vx yarn@4.0.0 install

# Add packages
vx yarn@4.0.0 add react

# Run scripts
vx yarn@4.0.0 build
```

### Yarn Version Pinning

For projects using Yarn Berry, vx respects the `packageManager` field in `package.json`:

```json
{
  "name": "my-project",
  "packageManager": "yarn@4.0.0"
}
```

When this field exists, corepack ensures the correct version is used automatically.

## Bun

Bun is an all-in-one JavaScript runtime and toolkit.

```bash
# Install bun
vx install bun latest

# Usage
vx bun run script.ts
vx bun install
vx bun build ./index.ts
```

## Project Configuration

```toml
[tools]
node = "20"
pnpm = "latest"
yarn = "4.0.0"  # Yarn Berry works seamlessly

[scripts]
dev = "yarn run dev"
build = "yarn run build"
test = "yarn test"
```

## Common Workflows

### Create React App

```bash
vx npx create-react-app my-app
cd my-app
vx npm start
```

### Next.js Project

```bash
vx npx create-next-app@latest my-app
cd my-app
vx npm run dev
```

### Vite Project

```bash
vx npm create vite@latest my-app
cd my-app
vx npm install
vx npm run dev
```

### Yarn Berry Project

```bash
# Create a new project with Yarn 4.x
mkdir my-yarn-project && cd my-yarn-project
vx yarn@4.0.0 init
vx yarn@4.0.0 add react react-dom
vx yarn@4.0.0 dev
```
