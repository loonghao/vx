#!/usr/bin/env rust-script

//! # VX 自动安装功能演示
//!
//! 这个示例展示了 vx 的自动安装功能如何工作。
//!
//! ## 运行方式
//! ```bash
//! cargo run --example auto_install_demo
//! ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 VX 自动安装功能演示");
    println!("{}", "=".repeat(50));

    // 演示场景1：首次使用未安装的工具
    println!("\n📦 场景1: 首次使用未安装的工具");
    println!("命令: vx node --version");
    println!("预期行为:");
    println!("  1. 检测到 node 未安装");
    println!("  2. 自动获取最新版本信息");
    println!("  3. 下载并安装 Node.js");
    println!("  4. 执行 node --version");

    // 模拟自动安装过程
    simulate_auto_install("node", "20.10.0")?;

    // 演示场景2：项目特定版本
    println!("\n📋 场景2: 项目特定版本自动安装");
    println!("项目配置 (.vx.toml):");
    println!("  [tools]");
    println!("  node = \"18.17.0\"");
    println!("  python = \"3.11.5\"");
    println!("\n命令: vx python --version");
    println!("预期行为:");
    println!("  1. 读取项目配置");
    println!("  2. 检测到需要 Python 3.11.5");
    println!("  3. 自动安装指定版本");
    println!("  4. 执行 python --version");

    simulate_auto_install("python", "3.11.5")?;

    // 演示场景3：配置控制
    println!("\n⚙️  场景3: 自动安装配置控制");
    println!("全局配置 (~/.vx/config.toml):");
    println!("  [auto_install]");
    println!("  enabled = false");
    println!("\n命令: vx go version");
    println!("预期行为:");
    println!("  1. 检测到 go 未安装");
    println!("  2. 发现自动安装已禁用");
    println!("  3. 显示手动安装提示");

    simulate_disabled_auto_install("go")?;

    // 演示场景4：错误处理
    println!("\n❌ 场景4: 自动安装错误处理");
    println!("命令: vx nonexistent-tool --version");
    println!("预期行为:");
    println!("  1. 检测到工具不存在");
    println!("  2. 在插件注册表中查找");
    println!("  3. 未找到支持的插件");
    println!("  4. 显示友好的错误信息");

    simulate_tool_not_found("nonexistent-tool")?;

    println!("\n✅ 演示完成！");
    println!("\n💡 关键特性总结:");
    println!("  • 透明的自动安装体验");
    println!("  • 智能版本选择（最新稳定版）");
    println!("  • 项目特定版本支持");
    println!("  • 可配置的安装行为");
    println!("  • 友好的错误处理和提示");

    Ok(())
}

/// 模拟自动安装过程
fn simulate_auto_install(tool: &str, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 检测到工具 '{}' 未安装", tool);
    println!("📦 正在获取最新版本信息...");

    // 模拟网络请求延迟
    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("⬇️  正在下载 {} v{}...", tool, version);

    // 模拟下载进度
    for i in 1..=5 {
        print!("   [{}{}] {}%\r", "=".repeat(i), " ".repeat(5 - i), i * 20);
        std::io::Write::flush(&mut std::io::stdout())?;
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    println!();

    let install_path = format!("~/.vx/tools/{}/{}/", tool, version);
    println!("📁 正在安装到 {}...", install_path);

    // 模拟安装过程
    std::thread::sleep(std::time::Duration::from_millis(300));

    println!("✅ 安装完成！");
    println!("🚀 执行: {} --version", tool);
    println!("📤 输出: v{}", version);

    Ok(())
}

/// 模拟禁用自动安装的情况
fn simulate_disabled_auto_install(tool: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 检测到工具 '{}' 未安装", tool);
    println!("⚠️  自动安装已禁用");
    println!("💡 提示: 请手动安装工具:");
    println!("   vx install {}", tool);
    println!("   或启用自动安装:");
    println!("   vx config set auto_install.enabled true");

    Ok(())
}

/// 模拟工具未找到的情况
fn simulate_tool_not_found(tool: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 检测到工具 '{}' 未安装", tool);
    println!("🔌 在插件注册表中查找...");

    std::thread::sleep(std::time::Duration::from_millis(300));

    println!("❌ 未找到支持 '{}' 的插件", tool);
    println!("💡 建议:");
    println!("   • 检查工具名称是否正确");
    println!("   • 查看支持的工具: vx list");
    println!("   • 搜索可用插件: vx plugin search {}", tool);
    println!(
        "   • 或使用系统PATH: vx --use-system-path {} --version",
        tool
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_functions() {
        // 测试演示函数不会panic
        assert!(simulate_auto_install("test-tool", "1.0.0").is_ok());
        assert!(simulate_disabled_auto_install("test-tool").is_ok());
        assert!(simulate_tool_not_found("test-tool").is_ok());
    }
}
