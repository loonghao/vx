# Node.js 生态系统

vx 提供对 Node.js 生态系统的全面支持。

## 支持的工具

| 工具 | 描述 |
|------|------|
| `node` | Node.js 运行时 |
| `npm` | Node.js 包管理器 |
| `npx` | Node.js 包运行器 |
| `pnpm` | 快速、高效的包管理器 |
| `yarn` | JavaScript 包管理器（支持所有版本） |
| `bun` | Bun JavaScript 运行时 |

## Node.js

### 安装

```bash
vx install node 20
vx install node lts
vx install node latest
```

### 版本说明

```bash
node 20          # 最新 20.x.x
node 20.10       # 最新 20.10.x
node 20.10.0     # 精确版本
node lts         # 最新 LTS
node latest      # 最新稳定版
```

### 使用示例

```bash
# 运行 Node.js
vx node --version
vx node script.js
vx node -e "console.log('Hello')"

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

## npm

npm 包含在 Node.js 中。

```bash
vx npm install
vx npm run build
vx npm test
```

## npx

npx 包含在 Node.js 中。

```bash
vx npx create-react-app my-app
vx npx eslint .
vx npx prettier --write .
```

## pnpm

```bash
# 安装 pnpm
vx install pnpm latest

# 使用
vx pnpm install
vx pnpm run build
vx pnpm add react
```

## Yarn

vx 提供对**所有 Yarn 版本**的无缝支持 - 包括 Yarn 1.x（Classic）和 Yarn 2.x+（Berry/Modern）。

### Yarn 版本支持

| 版本 | 类型 | 安装方式 |
|------|------|----------|
| 1.x | Classic | 从 GitHub releases 直接下载 |
| 2.x | Berry | 通过 corepack 自动管理 |
| 3.x | Berry | 通过 corepack 自动管理 |
| 4.x | Berry | 通过 corepack 自动管理 |

### Yarn 1.x (Classic)

Yarn 1.x 由 vx 直接下载和安装：

```bash
# 安装 Yarn 1.x
vx install yarn 1.22.22

# 检查版本
vx yarn@1.22.22 --version
# 输出: 1.22.22

# 使用
vx yarn install
vx yarn build
vx yarn add react
```

### Yarn 2.x+ (Berry/Modern)

Yarn 2.x 及更高版本**通过 corepack 自动管理** - vx 透明处理这一切：

```bash
# 直接使用任意 Yarn 2.x+ 版本
vx yarn@2.4.3 --version    # Berry 2.x
# 输出: 2.4.3

vx yarn@3.6.0 --version    # Berry 3.x
# 输出: 3.6.0

vx yarn@4.0.0 --version    # Berry 4.x
# 输出: 4.0.0

vx yarn@4.12.0 --version   # 最新 Berry
# 输出: 4.12.0
```

**工作原理：**

1. 当您请求 Yarn 2.x+ 时，vx 自动：
   - 确保 Node.js（包含 corepack）已安装
   - 如果尚未启用，则启用 corepack
   - 通过 `corepack prepare yarn@<version> --activate` 准备指定的 Yarn 版本
   - 通过 corepack 执行 Yarn

2. 您无需手动启用 corepack 或运行任何设置命令 - vx 透明处理一切。

### 在项目中使用 Yarn

```bash
# 使用 Yarn Berry 创建新项目
mkdir my-project && cd my-project
vx yarn@4.0.0 init

# 安装依赖
vx yarn@4.0.0 install

# 添加包
vx yarn@4.0.0 add react

# 运行脚本
vx yarn@4.0.0 build
```

### Yarn 版本锁定

对于使用 Yarn Berry 的项目，vx 遵循 `package.json` 中的 `packageManager` 字段：

```json
{
  "name": "my-project",
  "packageManager": "yarn@4.0.0"
}
```

当此字段存在时，corepack 会自动确保使用正确的版本。

## Bun

Bun 是一个多合一的 JavaScript 运行时和工具包。

```bash
# 安装 bun
vx install bun latest

# 使用
vx bun run script.ts
vx bun install
vx bun build ./index.ts
```

## 项目配置

```toml
[tools]
node = "20"
pnpm = "latest"
yarn = "4.0.0"  # Yarn Berry 无缝支持

[scripts]
dev = "yarn run dev"
build = "yarn run build"
test = "yarn test"
```

## 常见工作流

### 创建 React 应用

```bash
vx npx create-react-app my-app
cd my-app
vx npm start
```

### Next.js 项目

```bash
vx npx create-next-app@latest my-app
cd my-app
vx npm run dev
```

### Vite 项目

```bash
vx npm create vite@latest my-app
cd my-app
vx npm install
vx npm run dev
```

### Yarn Berry 项目

```bash
# 使用 Yarn 4.x 创建新项目
mkdir my-yarn-project && cd my-yarn-project
vx yarn@4.0.0 init
vx yarn@4.0.0 add react react-dom
vx yarn@4.0.0 dev
```
