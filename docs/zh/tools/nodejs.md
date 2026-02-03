# Node.js

vx 支持 Node.js 及其生态系统工具。

## 支持的工具

| 工具 | 描述 |
|------|------|
| `node` | Node.js 运行时 |
| `npm` | Node.js 包管理器 |
| `npx` | Node.js 包运行器 |
| `pnpm` | 快速包管理器 |
| `yarn` | Yarn 包管理器 |
| `bun` | Bun JavaScript 运行时 |

## 使用示例

```bash
# 运行 Node.js
vx node --version
vx node script.js

# 使用 npm
vx npm install
vx npm run build

# 使用 npx
vx npx create-react-app my-app
vx npx eslint .

# 使用 pnpm
vx pnpm install
vx pnpm run dev
```

## 版本管理

```bash
# 安装特定版本
vx install node@20
vx install node@18.19.0

# 使用 LTS 版本
vx install node@lts
```

## Yarn 支持说明

vx 支持 Yarn 1.x (Classic)。对于 Yarn 2.x+ (Berry)，请使用 Node.js 内置的 corepack。

### Yarn 1.x (Classic)

```bash
# 安装 Yarn 1.x
vx install yarn 1.22.19
vx yarn --version

# 使用 Yarn
vx yarn install
vx yarn build
vx yarn add react
```

### Yarn 2.x+ (Berry)

Yarn 2.x+ (Berry) 无法直接通过 vx 安装。请使用 Node.js 内置的 corepack：

```bash
# 首先安装 Node.js
vx install node 20

# 启用 corepack（提供 Yarn 2.x+）
vx node -e "require('child_process').execSync('corepack enable', {stdio: 'inherit'})"

# 或使用 npx 运行 Yarn
vx npx yarn@2.4.3 --version
```

更多详情请参考 [Yarn 官方文档](https://yarnpkg.com/getting-started/install)。

## 项目配置

```toml
[tools]
node = "20"

[scripts]
dev = "npm run dev"
build = "npm run build"
test = "npm test"
```
