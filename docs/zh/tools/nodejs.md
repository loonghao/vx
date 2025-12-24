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

## 项目配置

```toml
[tools]
node = "20"

[scripts]
dev = "npm run dev"
build = "npm run build"
test = "npm test"
```
