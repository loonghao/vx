# Node.js Ecosystem

vx provides comprehensive support for the Node.js ecosystem.

## Supported Tools

| Tool | Description |
|------|-------------|
| `node` | Node.js runtime |
| `npm` | Node.js package manager |
| `npx` | Node.js package runner |
| `pnpm` | Fast, disk space efficient package manager |
| `yarn` | JavaScript package manager |
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

vx supports Yarn 1.x (Classic). For Yarn 2.x+ (Berry), please use corepack which is bundled with Node.js.

### Yarn 1.x (Classic)

```bash
# Install Yarn 1.x
vx install yarn 1.22.19
vx yarn --version

# Usage
vx yarn install
vx yarn build
vx yarn add react
```

### Yarn 2.x+ (Berry)

Yarn 2.x+ (Berry) is not directly installable via vx. Instead, use corepack which is bundled with Node.js:

```bash
# Install Node.js first
vx install node 20

# Enable corepack (provides Yarn 2.x+)
vx node -e "require('child_process').execSync('corepack enable', {stdio: 'inherit'})"

# Or use npx to run Yarn
vx npx yarn@2.4.3 --version
```

For more details, see the [Yarn official documentation](https://yarnpkg.com/getting-started/install).

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

[scripts]
dev = "pnpm run dev"
build = "pnpm run build"
test = "pnpm test"
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
