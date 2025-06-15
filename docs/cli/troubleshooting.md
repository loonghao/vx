# 故障排除指南

VX 常见问题的诊断和解决方案。

## 🔧 常见问题

### 工具安装失败

#### 网络连接问题
```bash
# 检查网络连接
vx --verbose install node@18.17.0

# 使用代理
export HTTP_PROXY="http://proxy:8080"
export HTTPS_PROXY="https://proxy:8080"
vx install node@18.17.0

# 使用镜像源
vx config set mirrors.node "https://npmmirror.com/mirrors/node/"
vx install node@18.17.0
```

#### 权限问题
```bash
# 检查安装目录权限
ls -la ~/.vx/tools/

# 修复权限
chmod -R 755 ~/.vx/tools/
mkdir -p ~/.vx/tools/
vx install node@18.17.0
```

#### 磁盘空间不足
```bash
# 检查磁盘空间
df -h ~/.vx/

# 清理缓存
vx cleanup --cache-only

# 清理未使用的工具
vx global cleanup
```

### 虚拟环境问题

#### 环境创建失败
```bash
# 检查虚拟环境目录
ls -la ~/.vx/venvs/

# 清理并重新创建
vx venv remove myproject --force
vx venv create myproject --tools node@18.17.0

# 检查权限
chmod -R 755 ~/.vx/venvs/
```

#### 激活失败
```bash
# 检查环境是否存在
vx venv list

# 手动激活
eval "$(vx venv activate myproject)"

# 检查环境变量
echo $VX_VENV
```

#### 工具版本错误
```bash
# 检查环境配置
vx venv list --verbose

# 重新添加工具
vx venv remove-tool myproject node
vx venv add myproject node@18.17.0
```

### 配置问题

#### 配置文件语法错误
```bash
# 验证配置文件
vx config validate

# 检查具体错误
vx config validate --local --verbose

# 重置配置
mv ~/.config/vx/config.toml ~/.config/vx/config.toml.backup
vx config init
```

#### 配置不生效
```bash
# 检查配置层次
vx config --sources

# 检查环境变量
env | grep VX_

# 重新加载配置
vx config validate
```

#### 项目配置冲突
```bash
# 显示有效配置
vx config show

# 检查项目配置
vx config show --local

# 重新初始化项目配置
rm .vx.toml
vx init
```

### 版本管理问题

#### 版本不存在
```bash
# 列出可用版本
vx list node

# 搜索版本
vx search node --version 18

# 刷新版本缓存
vx update --refresh-cache
```

#### 版本切换失败
```bash
# 检查已安装版本
vx list node --installed-only

# 安装目标版本
vx install node@20.10.0

# 切换版本
vx switch node@20.10.0
```

#### 版本冲突
```bash
# 检查版本要求
vx config get tools.node

# 更新版本要求
vx config set tools.node "^18.0.0" --local

# 重新同步
vx sync
```

## 🔍 诊断工具

### 系统信息
```bash
# 显示系统信息
vx --version
vx config show
vx list --status

# 检查环境
env | grep VX_
echo $PATH
```

### 详细日志
```bash
# 启用详细日志
export VX_VERBOSE=true
vx node --version

# 单次命令启用
vx --verbose install node@18.17.0

# 检查日志文件
tail -f ~/.vx/logs/vx.log
```

### 网络诊断
```bash
# 测试网络连接
curl -I https://nodejs.org/dist/

# 检查代理设置
echo $HTTP_PROXY
echo $HTTPS_PROXY

# 测试下载
vx --verbose install node@18.17.0
```

### 路径诊断
```bash
# 显示工具路径
vx which node
vx which uv

# 检查PATH
echo $PATH

# 显示工具版本信息
vx version node
vx version --all
```

## 🛠️ 修复工具

### 重置VX
```bash
# 备份配置
cp -r ~/.vx ~/.vx.backup

# 清理所有数据
rm -rf ~/.vx

# 重新初始化
vx config init
```

### 修复安装
```bash
# 清理缓存
vx cleanup --cache-only

# 重新安装工具
vx install node@18.17.0 --force

# 验证安装
vx node --version
```

### 修复虚拟环境
```bash
# 重新创建环境
vx venv remove myproject --force
vx venv create myproject --from-config

# 验证环境
vx venv list
vx venv use myproject
```

## 📊 性能问题

### 安装速度慢
```bash
# 使用镜像源
vx config set mirrors.node "https://npmmirror.com/mirrors/node/"
vx config set mirrors.python "https://npmmirror.com/mirrors/python/"

# 启用并行下载
vx config set install.parallel_downloads 4

# 增加超时时间
vx config set install.timeout 600
```

### 磁盘使用过多
```bash
# 检查磁盘使用
vx stats --detailed

# 清理未使用的工具
vx global cleanup

# 清理缓存
vx cleanup --cache-only

# 清理孤立文件
vx cleanup --orphaned-only
```

### 内存使用过多
```bash
# 减少并行下载数
vx config set install.parallel_downloads 2

# 禁用缓存
vx config set cache.enabled false

# 使用系统PATH
vx --use-system-path node --version
```

## 🆘 获取帮助

### 社区支持
- GitHub Issues: https://github.com/loonghao/vx/issues
- 讨论区: https://github.com/loonghao/vx/discussions
- 文档: https://vx.dev/docs

### 报告问题
```bash
# 收集诊断信息
vx --version
vx config show
vx list --status

# 生成诊断报告
vx diagnose --output vx-report.txt
```

### 调试模式
```bash
# 启用调试模式
export VX_DEBUG=true
export VX_VERBOSE=true

# 运行问题命令
vx install node@18.17.0

# 检查调试日志
cat ~/.vx/logs/debug.log
```

## 🔄 恢复策略

### 从备份恢复
```bash
# 恢复配置
cp ~/.vx.backup/config/global.toml ~/.vx/config/

# 恢复工具
cp -r ~/.vx.backup/tools/* ~/.vx/tools/

# 验证恢复
vx list --status
```

### 重新安装
```bash
# 完全重新安装
rm -rf ~/.vx
vx config init
vx sync

# 验证安装
vx --version
vx list
```
