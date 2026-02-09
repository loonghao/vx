import { defineConfig } from 'vitepress'

// Shared config
const sharedConfig = {
  base: '/vx/',
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/vx/logo.svg' }],
  ],
}

// English sidebar
const enSidebar = {
  '/guide/': [
    {
      text: 'Getting Started',
      items: [
        { text: 'Introduction', link: '/guide/' },
        { text: 'Installation', link: '/guide/installation' },
        { text: 'Quick Start', link: '/guide/getting-started' },
        { text: 'Core Concepts', link: '/guide/concepts' }
      ]
    },
    {
      text: 'Usage',
      items: [
        { text: 'Direct Execution', link: '/guide/direct-execution' },
        { text: 'Version Management', link: '/guide/version-management' },
        { text: 'Configuration', link: '/guide/configuration' },
        { text: 'Enhanced Scripts', link: '/guide/enhanced-scripts' },
        { text: 'Shell Integration', link: '/guide/shell-integration' }
      ]
    },
    {
      text: 'Environments',
      items: [
        { text: 'Project Environments', link: '/guide/project-environments' },
        { text: 'Environment Management', link: '/guide/environment-management' },
        { text: 'Virtual Environment Isolation', link: '/guide/virtual-env-isolation' }
      ]
    },
    {
      text: 'Advanced Topics',
      items: [
        { text: 'Manifest-Driven Providers', link: '/guide/manifest-driven-providers' },
        { text: 'CDN Fallback', link: '/guide/cdn-fallback' },
        { text: 'Windows Long Path', link: '/guide/windows-long-path' },
        { text: 'Migration Guide', link: '/guide/migration' },
        { text: 'Best Practices', link: '/guide/best-practices' }
      ]
    }
  ],
  '/cli/': [
    {
      text: 'CLI Reference',
      items: [
        { text: 'Overview', link: '/cli/overview' },
        { text: 'All Commands', link: '/cli/commands' }
      ]
    },
    {
      text: 'Tool Management',
      items: [
        { text: 'install', link: '/cli/install' },
        { text: 'list', link: '/cli/list' },
        { text: 'test', link: '/cli/test' },
        { text: 'global', link: '/cli/global' },
        { text: 'info', link: '/cli/info' }
      ]
    },
    {
      text: 'Project & Scripts',
      items: [
        { text: 'run', link: '/cli/run' },
        { text: 'dev', link: '/cli/dev' },
        { text: 'setup', link: '/cli/setup' },
        { text: 'env', link: '/cli/env' }
      ]
    },
    {
      text: 'Configuration & Shell',
      items: [
        { text: 'config', link: '/cli/config' },
        { text: 'shell', link: '/cli/shell' },
        { text: 'metrics', link: '/cli/metrics' }
      ]
    },
    {
      text: 'Extensions',
      items: [
        { text: 'ext', link: '/cli/ext' },
        { text: 'plugin', link: '/cli/plugin' },
        { text: 'Implicit Package Execution', link: '/cli/implicit-package-execution' }
      ]
    }
  ],
  '/config/': [
    {
      text: 'Configuration Reference',
      items: [
        { text: 'vx.toml', link: '/config/vx-toml' },
        { text: 'Global Config', link: '/config/global' },
        { text: 'Environment Variables', link: '/config/env-vars' },
        { text: 'Provider Overrides', link: '/config/provider-overrides' }
      ]
    }
  ],
  '/tools/': [
    {
      text: 'Language Runtimes',
      items: [
        { text: 'Overview', link: '/tools/overview' },
        { text: 'Node.js', link: '/tools/nodejs' },
        { text: 'Python', link: '/tools/python' },
        { text: 'Go', link: '/tools/go' },
        { text: 'Rust', link: '/tools/rust' }
      ]
    },
    {
      text: 'DevOps & Cloud',
      items: [
        { text: 'DevOps Tools', link: '/tools/devops' },
        { text: 'Cloud CLI', link: '/tools/cloud' }
      ]
    },
    {
      text: 'Build & Quality',
      items: [
        { text: 'Build Tools', link: '/tools/build-tools' },
        { text: 'Code Quality', link: '/tools/quality' }
      ]
    },
    {
      text: 'Specialized',
      items: [
        { text: 'AI Tools', link: '/tools/ai' },
        { text: 'Scientific & HPC', link: '/tools/scientific' },
        { text: 'Media Processing', link: '/tools/media' },
        { text: 'Other Tools', link: '/tools/other' }
      ]
    }
  ],
  '/guides/': [
    {
      text: 'Tutorials & Guides',
      items: [
        { text: 'GitHub Action', link: '/guides/github-action' },
        { text: 'Use Cases', link: '/guides/use-cases' },
        { text: 'Runtime Lifecycle', link: '/guides/runtime-lifecycle' },
        { text: 'Platform Redirection', link: '/guides/platform-redirection' }
      ]
    }
  ],
  '/advanced/': [
    {
      text: 'Architecture',
      items: [
        { text: 'System Architecture', link: '/advanced/architecture' },
        { text: 'Provider Version Resolution', link: '/advanced/provider-version-resolution' },
        { text: 'Security', link: '/advanced/security' }
      ]
    },
    {
      text: 'Developer Guide',
      items: [
        { text: 'Contributing', link: '/advanced/contributing' },
        { text: 'Provider Development', link: '/advanced/plugin-development' },
        { text: 'CLI Command Development', link: '/advanced/cli-development' },
        { text: 'Extension Development', link: '/advanced/extension-development' },
        { text: 'Release Process', link: '/advanced/release-process' }
      ]
    }
  ],
  '/appendix/': [
    {
      text: 'Appendix',
      items: [
        { text: 'FAQ', link: '/appendix/faq' },
        { text: 'Troubleshooting', link: '/appendix/troubleshooting' }
      ]
    }
  ]
}

// Chinese sidebar
const zhSidebar = {
  '/zh/guide/': [
    {
      text: '快速开始',
      items: [
        { text: '简介', link: '/zh/guide/' },
        { text: '安装', link: '/zh/guide/installation' },
        { text: '快速上手', link: '/zh/guide/getting-started' },
        { text: '核心概念', link: '/zh/guide/concepts' }
      ]
    },
    {
      text: '使用指南',
      items: [
        { text: '直接执行', link: '/zh/guide/direct-execution' },
        { text: '版本管理', link: '/zh/guide/version-management' },
        { text: '配置', link: '/zh/guide/configuration' },
        { text: '增强脚本系统', link: '/zh/guide/enhanced-scripts' },
        { text: 'Shell 集成', link: '/zh/guide/shell-integration' }
      ]
    },
    {
      text: '环境管理',
      items: [
        { text: '项目环境', link: '/zh/guide/project-environments' },
        { text: '环境管理', link: '/zh/guide/environment-management' },
        { text: '虚拟环境隔离', link: '/zh/guide/virtual-env-isolation' }
      ]
    },
    {
      text: '进阶主题',
      items: [
        { text: '声明式 Provider', link: '/zh/guide/manifest-driven-providers' },
        { text: 'CDN 回退', link: '/zh/guide/cdn-fallback' },
        { text: 'Windows 长路径', link: '/zh/guide/windows-long-path' },
        { text: '迁移指南', link: '/zh/guide/migration' },
        { text: '最佳实践', link: '/zh/guide/best-practices' }
      ]
    }
  ],
  '/zh/cli/': [
    {
      text: 'CLI 参考',
      items: [
        { text: '概览', link: '/zh/cli/overview' },
        { text: '全部命令', link: '/zh/cli/commands' }
      ]
    },
    {
      text: '工具管理',
      items: [
        { text: 'install', link: '/zh/cli/install' },
        { text: 'list', link: '/zh/cli/list' },
        { text: 'test', link: '/zh/cli/test' },
        { text: 'global', link: '/zh/cli/global' },
        { text: 'info', link: '/zh/cli/info' }
      ]
    },
    {
      text: '项目与脚本',
      items: [
        { text: 'run', link: '/zh/cli/run' },
        { text: 'dev', link: '/zh/cli/dev' },
        { text: 'setup', link: '/zh/cli/setup' },
        { text: 'env', link: '/zh/cli/env' }
      ]
    },
    {
      text: '配置与 Shell',
      items: [
        { text: 'config', link: '/zh/cli/config' },
        { text: 'shell', link: '/zh/cli/shell' },
        { text: 'metrics', link: '/zh/cli/metrics' }
      ]
    },
    {
      text: '扩展',
      items: [
        { text: 'ext', link: '/zh/cli/ext' },
        { text: 'plugin', link: '/zh/cli/plugin' },
        { text: '隐式包执行', link: '/zh/cli/implicit-package-execution' }
      ]
    }
  ],
  '/zh/config/': [
    {
      text: '配置参考',
      items: [
        { text: 'vx.toml', link: '/zh/config/vx-toml' },
        { text: '全局配置', link: '/zh/config/global' },
        { text: '环境变量', link: '/zh/config/env-vars' },
        { text: 'Provider 覆盖', link: '/zh/config/provider-overrides' }
      ]
    }
  ],
  '/zh/tools/': [
    {
      text: '语言运行时',
      items: [
        { text: '概览', link: '/zh/tools/overview' },
        { text: 'Node.js', link: '/zh/tools/nodejs' },
        { text: 'Python', link: '/zh/tools/python' },
        { text: 'Go', link: '/zh/tools/go' },
        { text: 'Rust', link: '/zh/tools/rust' }
      ]
    },
    {
      text: 'DevOps & 云',
      items: [
        { text: 'DevOps 工具', link: '/zh/tools/devops' },
        { text: '云 CLI', link: '/zh/tools/cloud' }
      ]
    },
    {
      text: '构建与质量',
      items: [
        { text: '构建工具', link: '/zh/tools/build-tools' },
        { text: '代码质量', link: '/zh/tools/quality' }
      ]
    },
    {
      text: '专业工具',
      items: [
        { text: 'AI 工具', link: '/zh/tools/ai' },
        { text: '科学计算 & HPC', link: '/zh/tools/scientific' },
        { text: '媒体处理', link: '/zh/tools/media' },
        { text: '其他工具', link: '/zh/tools/other' }
      ]
    }
  ],
  '/zh/guides/': [
    {
      text: '教程与指南',
      items: [
        { text: 'GitHub Action', link: '/zh/guides/github-action' },
        { text: '使用案例', link: '/zh/guides/use-cases' },
        { text: '运行时生命周期', link: '/zh/guides/runtime-lifecycle' },
        { text: '平台重定向', link: '/zh/guides/platform-redirection' }
      ]
    }
  ],
  '/zh/advanced/': [
    {
      text: '架构',
      items: [
        { text: '系统架构', link: '/zh/advanced/architecture' },
        { text: 'Provider 版本解析', link: '/zh/advanced/provider-version-resolution' },
        { text: '安全', link: '/zh/advanced/security' }
      ]
    },
    {
      text: '开发者指南',
      items: [
        { text: '贡献指南', link: '/zh/advanced/contributing' },
        { text: 'Provider 开发', link: '/zh/advanced/plugin-development' },
        { text: 'CLI 命令开发', link: '/zh/advanced/cli-development' },
        { text: 'Extension 开发', link: '/zh/advanced/extension-development' },
        { text: '发布流程', link: '/zh/advanced/release-process' }
      ]
    }
  ],
  '/zh/appendix/': [
    {
      text: '附录',
      items: [
        { text: '常见问题', link: '/zh/appendix/faq' },
        { text: '故障排除', link: '/zh/appendix/troubleshooting' }
      ]
    }
  ]
}

export default defineConfig({
  ...sharedConfig,
  title: 'vx',
  description: 'Universal Development Tool Manager with Zero Learning Curve',

  // Ignore dead links to local-only RFC documents and source code
  ignoreDeadLinks: [
    /\/rfcs\//,
    /\/crates\//
  ],

  locales: {
    root: {
      label: 'English',
      lang: 'en'
    },
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh/',
      themeConfig: {
        nav: [
          { text: '指南', link: '/zh/guide/getting-started' },
          { text: 'CLI', link: '/zh/cli/overview' },
          { text: '配置', link: '/zh/config/vx-toml' },
          { text: '工具', link: '/zh/tools/overview' },
          { text: '教程', link: '/zh/guides/github-action' },
          {
            text: '更多',
            items: [
              { text: '常见问题', link: '/zh/appendix/faq' },
              { text: '故障排除', link: '/zh/appendix/troubleshooting' },
              { text: '贡献指南', link: '/zh/advanced/contributing' },
              { text: '更新日志', link: 'https://github.com/loonghao/vx/releases' }
            ]
          }
        ],
        sidebar: zhSidebar,
        editLink: {
          pattern: 'https://github.com/loonghao/vx/edit/main/docs/:path',
          text: '在 GitHub 上编辑此页'
        },
        footer: {
          message: '基于 MIT 许可证发布',
          copyright: 'Copyright © 2024-present loonghao'
        },
        docFooter: {
          prev: '上一页',
          next: '下一页'
        },
        outline: {
          label: '页面导航'
        },
        lastUpdated: {
          text: '最后更新于'
        },
        returnToTopLabel: '回到顶部',
        sidebarMenuLabel: '菜单',
        darkModeSwitchLabel: '主题',
        lightModeSwitchTitle: '切换到浅色模式',
        darkModeSwitchTitle: '切换到深色模式'
      }
    }
  },

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'CLI', link: '/cli/overview' },
      { text: 'Config', link: '/config/vx-toml' },
      { text: 'Tools', link: '/tools/overview' },
      { text: 'Tutorials', link: '/guides/github-action' },
      {
        text: 'More',
        items: [
          { text: 'FAQ', link: '/appendix/faq' },
          { text: 'Troubleshooting', link: '/appendix/troubleshooting' },
          { text: 'Contributing', link: '/advanced/contributing' },
          { text: 'Changelog', link: 'https://github.com/loonghao/vx/releases' }
        ]
      }
    ],

    sidebar: enSidebar,

    socialLinks: [
      { icon: 'github', link: 'https://github.com/loonghao/vx' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present loonghao'
    },

    search: {
      provider: 'local'
    },

    editLink: {
      pattern: 'https://github.com/loonghao/vx/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    }
  }
})
