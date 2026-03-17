import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Kanban',
  description: 'AI Agent Orchestration Board',
  base: '/kanban/',
  head: [
    ['meta', { name: 'theme-color', content: '#6366f1' }],
  ],
  themeConfig: {
    logo: undefined,
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'CLI Reference', link: '/cli/' },
      { text: 'MCP', link: '/mcp/' },
      { text: 'Agent Protocol', link: '/agents/' },
      { text: 'GitHub', link: 'https://github.com/akassharjun/kanban' },
    ],
    sidebar: {
      '/guide/': [
        {
          text: 'Introduction',
          items: [
            { text: 'What is Kanban?', link: '/guide/' },
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Core Concepts', link: '/guide/concepts' },
          ],
        },
        {
          text: 'Features',
          items: [
            { text: 'Projects', link: '/guide/projects' },
            { text: 'Issues', link: '/guide/issues' },
            { text: 'Statuses & Workflow', link: '/guide/statuses' },
            { text: 'Labels & Filters', link: '/guide/labels' },
            { text: 'Members & Assignment', link: '/guide/members' },
            { text: 'Task Contracts', link: '/guide/task-contracts' },
            { text: 'Agent Routing', link: '/guide/agent-routing' },
            { text: 'Validation Pipeline', link: '/guide/validation' },
            { text: 'Execution Replay', link: '/guide/execution-replay' },
            { text: 'Import & Export', link: '/guide/import-export' },
          ],
        },
      ],
      '/cli/': [
        {
          text: 'CLI Reference',
          items: [
            { text: 'Overview', link: '/cli/' },
            { text: 'Projects', link: '/cli/projects' },
            { text: 'Issues', link: '/cli/issues' },
            { text: 'Members', link: '/cli/members' },
            { text: 'Labels', link: '/cli/labels' },
            { text: 'Agents', link: '/cli/agents' },
            { text: 'Tasks', link: '/cli/tasks' },
            { text: 'Metrics', link: '/cli/metrics' },
          ],
        },
      ],
      '/mcp/': [
        {
          text: 'MCP Server',
          items: [
            { text: 'Overview', link: '/mcp/' },
            { text: 'Configuration', link: '/mcp/configuration' },
            { text: 'Tools Reference', link: '/mcp/tools' },
          ],
        },
      ],
      '/agents/': [
        {
          text: 'Agent Protocol',
          items: [
            { text: 'Overview', link: '/agents/' },
            { text: 'Lifecycle', link: '/agents/lifecycle' },
            { text: 'Task Contracts', link: '/agents/task-contracts' },
            { text: 'Examples', link: '/agents/examples' },
          ],
        },
      ],
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/akassharjun/kanban' },
    ],
    search: {
      provider: 'local',
    },
    footer: {
      message: 'Released under the MIT License.',
    },
  },
})
