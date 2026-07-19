// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// The Ultimate Systems Programming Academy
// Author: Walter Angolar

// Rehype plugin: make every external link (http/https) open in a new tab,
// so reading references never navigate the reader away from the lesson.
function rehypeExternalLinksNewTab() {
  return (/** @type {any} */ tree) => {
    const visit = (/** @type {any} */ node) => {
      if (
        node.type === 'element' &&
        node.tagName === 'a' &&
        typeof node.properties?.href === 'string' &&
        /^https?:\/\//i.test(node.properties.href)
      ) {
        node.properties.target = '_blank';
        node.properties.rel = ['noopener', 'noreferrer'];
      }
      if (Array.isArray(node.children)) node.children.forEach(visit);
    };
    visit(tree);
  };
}

export default defineConfig({
  site: 'https://angolardevops.github.io',
  base: '/systems-programming-academy',
  markdown: {
    rehypePlugins: [rehypeExternalLinksNewTab],
  },
  integrations: [
    starlight({
      title: 'Systems Programming Academy',
      description:
        'Learn production software engineering from absolute beginner to advanced with Python, Go, and Rust.',
      tagline: 'Python · Go · Rust — from first principles to production.',
      defaultLocale: 'root',
      locales: {
        root: { label: 'English', lang: 'en' },
        pt: { label: 'Português', lang: 'pt-BR' },
      },
      // Show estimated reading time on every lesson.
      // Enables git-blame-free last-updated + edit links can be added later.
      customCss: ['./src/styles/academy.css'],
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/angolardevops' },
      ],
      sidebar: [
        {
          label: 'Start Here',
          translations: { 'pt-BR': 'Comece Aqui' },
          items: [
            {
              label: 'About the Author',
              translations: { 'pt-BR': 'Sobre o Autor' },
              slug: 'about-the-author',
            },
            { label: 'Welcome', translations: { 'pt-BR': 'Bem-vindo' }, slug: 'welcome' },
            {
              label: 'How to use this Academy',
              translations: { 'pt-BR': 'Como usar esta Academia' },
              slug: 'how-to-use',
            },
          ],
        },
        {
          label: 'Toolchains & Contributing',
          translations: { 'pt-BR': 'Ferramentas & Contribuir' },
          items: [
            {
              label: 'The Toolchains: cargo · uv · go',
              translations: { 'pt-BR': 'As Toolchains: cargo · uv · go' },
              slug: 'toolchains/cli-tools',
            },
            {
              label: 'Git & GitHub for Contributors',
              translations: { 'pt-BR': 'Git & GitHub para Contribuidores' },
              slug: 'toolchains/git-github',
            },
            {
              label: 'Where to Contribute',
              translations: { 'pt-BR': 'Onde Contribuir' },
              slug: 'toolchains/contributing',
            },
          ],
        },
        {
          label: 'Part 1 · Foundations',
          translations: { 'pt-BR': 'Parte 1 · Fundamentos' },
          items: [
            {
              label: 'Rust',
              collapsed: true,
              items: [
                {
                  label: 'Ownership & the Borrow Checker',
                  translations: { 'pt-BR': 'Ownership e o Borrow Checker' },
                  slug: 'part-1-foundations/rust/ownership',
                },
                {
                  label: 'Error Handling (Result, Option, ?)',
                  translations: { 'pt-BR': 'Tratamento de Erros (Result, Option, ?)' },
                  slug: 'part-1-foundations/rust/error-handling',
                },
                {
                  label: 'Types & Traits',
                  translations: { 'pt-BR': 'Tipos e Traits' },
                  slug: 'part-1-foundations/rust/traits',
                },
                {
                  label: 'Collections & Iterators',
                  translations: { 'pt-BR': 'Coleções e Iteradores' },
                  slug: 'part-1-foundations/rust/collections',
                },
                {
                  label: 'Testing & Documentation',
                  translations: { 'pt-BR': 'Testes e Documentação' },
                  slug: 'part-1-foundations/rust/testing',
                },
                {
                  label: 'Generics & Lifetimes',
                  translations: { 'pt-BR': 'Generics e Lifetimes' },
                  slug: 'part-1-foundations/rust/generics',
                },
                {
                  label: 'Smart Pointers & Interior Mutability',
                  translations: { 'pt-BR': 'Smart Pointers e Mutabilidade Interior' },
                  slug: 'part-1-foundations/rust/smart-pointers',
                },
                {
                  label: 'Concurrency Basics',
                  translations: { 'pt-BR': 'Concorrência Básica' },
                  slug: 'part-1-foundations/rust/concurrency',
                },
                {
                  label: 'Unsafe & FFI',
                  translations: { 'pt-BR': 'Unsafe e FFI' },
                  slug: 'part-1-foundations/rust/unsafe-ffi',
                },
              ],
            },
            {
              label: 'Go',
              collapsed: true,
              items: [
                {
                  label: 'Structs, Methods & Interfaces',
                  translations: { 'pt-BR': 'Structs, Métodos e Interfaces' },
                  slug: 'part-1-foundations/go/interfaces',
                },
                {
                  label: 'Error Handling',
                  translations: { 'pt-BR': 'Tratamento de Erros' },
                  slug: 'part-1-foundations/go/error-handling',
                },
                {
                  label: 'Slices, Maps & Strings',
                  translations: { 'pt-BR': 'Slices, Maps e Strings' },
                  slug: 'part-1-foundations/go/collections',
                },
                {
                  label: 'Testing & Documentation',
                  translations: { 'pt-BR': 'Testes e Documentação' },
                  slug: 'part-1-foundations/go/testing',
                },
                {
                  label: 'Generics',
                  translations: { 'pt-BR': 'Generics' },
                  slug: 'part-1-foundations/go/generics',
                },
                {
                  label: 'Goroutines & Channels',
                  translations: { 'pt-BR': 'Goroutines e Channels' },
                  slug: 'part-1-foundations/go/concurrency',
                },
              ],
            },
            {
              label: 'Python',
              collapsed: true,
              items: [
                {
                  label: 'The Data Model',
                  translations: { 'pt-BR': 'O Modelo de Dados' },
                  slug: 'part-1-foundations/python/data-model',
                },
                {
                  label: 'Type Hints',
                  translations: { 'pt-BR': 'Type Hints' },
                  slug: 'part-1-foundations/python/type-hints',
                },
                {
                  label: 'Protocols & Duck Typing',
                  translations: { 'pt-BR': 'Protocols e Duck Typing' },
                  slug: 'part-1-foundations/python/protocols',
                },
                {
                  label: 'Iterators & Generators',
                  translations: { 'pt-BR': 'Iteradores e Geradores' },
                  slug: 'part-1-foundations/python/iterators',
                },
                {
                  label: 'Testing',
                  translations: { 'pt-BR': 'Testes' },
                  slug: 'part-1-foundations/python/testing',
                },
              ],
            },
          ],
        },
        {
          label: 'Part 2 · Real-World Engineering',
          translations: { 'pt-BR': 'Parte 2 · Engenharia do Mundo Real' },
          items: [
            {
              label: 'Repository Pattern & Dependency Injection',
              translations: { 'pt-BR': 'Padrão Repository e Injeção de Dependências' },
              slug: 'part-2-engineering/repository-pattern',
            },
            {
              label: 'Configuration & Secrets',
              translations: { 'pt-BR': 'Configuração e Segredos' },
              slug: 'part-2-engineering/configuration',
            },
            {
              label: 'Logging & Observability',
              translations: { 'pt-BR': 'Logging e Observabilidade' },
              slug: 'part-2-engineering/logging',
            },
            {
              label: 'Caching',
              translations: { 'pt-BR': 'Caching' },
              slug: 'part-2-engineering/caching',
            },
            {
              label: 'The Testing Pyramid',
              translations: { 'pt-BR': 'A Pirâmide de Testes' },
              slug: 'part-2-engineering/testing-pyramid',
            },
            {
              label: 'Clean & Hexagonal Architecture',
              translations: { 'pt-BR': 'Arquitetura Clean e Hexagonal' },
              slug: 'part-2-engineering/hexagonal-architecture',
            },
            {
              label: 'CI/CD',
              translations: { 'pt-BR': 'CI/CD' },
              slug: 'part-2-engineering/ci-cd',
            },
          ],
        },
        {
          label: 'Part 3 · DevOps Automation',
          translations: { 'pt-BR': 'Parte 3 · Automação DevOps' },
          items: [
            {
              label: 'Project: Log Analyzer CLI',
              translations: { 'pt-BR': 'Projeto: CLI Analisador de Logs' },
              slug: 'part-3-devops/log-analyzer',
            },
            {
              label: 'Project: Prometheus-Style Exporter',
              translations: { 'pt-BR': 'Projeto: Exporter Estilo Prometheus' },
              slug: 'part-3-devops/prometheus-exporter',
            },
            {
              label: 'Project: Config Generator',
              translations: { 'pt-BR': 'Projeto: Gerador de Configuração' },
              slug: 'part-3-devops/config-generator',
            },
            {
              label: 'Project: Health-Check Agent',
              translations: { 'pt-BR': 'Projeto: Agente de Health-Check' },
              slug: 'part-3-devops/health-check',
            },
          ],
        },
        {
          label: 'Part 4 · Concurrency & Parallelism',
          translations: { 'pt-BR': 'Parte 4 · Concorrência & Paralelismo' },
          items: [
            {
              label: 'Threads & Shared State',
              translations: { 'pt-BR': 'Threads & Estado Partilhado' },
              slug: 'part-4-concurrency/threads-shared-state',
            },
            {
              label: 'Message Passing & Channels',
              translations: { 'pt-BR': 'Message Passing & Channels' },
              slug: 'part-4-concurrency/message-passing',
            },
            {
              label: 'Async & Event-Driven I/O',
              translations: { 'pt-BR': 'Async & I/O Orientado a Eventos' },
              slug: 'part-4-concurrency/async-io',
            },
            {
              label: 'Project: mini-NGINX',
              translations: { 'pt-BR': 'Projeto: mini-NGINX' },
              slug: 'part-4-concurrency/mini-nginx',
              badge: { text: 'Capstone', variant: 'tip' },
            },
          ],
        },
        {
          label: 'Part 5 · Building Frameworks',
          translations: { 'pt-BR': 'Parte 5 · Construir Frameworks' },
          items: [
            {
              label: 'Routing & Middleware',
              translations: { 'pt-BR': 'Routing & Middleware' },
              slug: 'part-5-frameworks/routing-middleware',
            },
            {
              label: 'Query Builder & SQL Injection',
              translations: { 'pt-BR': 'Query Builder & SQL Injection' },
              slug: 'part-5-frameworks/query-builder',
            },
            {
              label: 'DI Container',
              translations: { 'pt-BR': 'DI Container' },
              slug: 'part-5-frameworks/di-container',
            },
            {
              label: 'Validation Framework',
              translations: { 'pt-BR': 'Framework de Validação' },
              slug: 'part-5-frameworks/validation',
            },
            {
              label: 'Template Engine & XSS',
              translations: { 'pt-BR': 'Template Engine & XSS' },
              slug: 'part-5-frameworks/template-engine',
            },
            {
              label: 'Serialization: JSON Encoder & Parser',
              translations: { 'pt-BR': 'Serialization: Encoder & Parser JSON' },
              slug: 'part-5-frameworks/serialization',
            },
            {
              label: 'Test Framework',
              translations: { 'pt-BR': 'Test Framework' },
              slug: 'part-5-frameworks/test-framework',
            },
          ],
        },
        {
          label: 'Part 6 · Command-Line Tools & Dashboards',
          translations: { 'pt-BR': 'Parte 6 · Ferramentas CLI & Dashboards' },
          items: [
            {
              label: 'Project: Performance Dashboard',
              translations: { 'pt-BR': 'Projeto: Dashboard de Performance' },
              slug: 'part-6-cli-tools/performance-dashboard',
            },
            {
              label: 'Project: Port Scanner',
              translations: { 'pt-BR': 'Projeto: Scanner de Portas' },
              slug: 'part-6-cli-tools/port-scanner',
            },
            {
              label: 'Project: ping (ICMP)',
              translations: { 'pt-BR': 'Projeto: ping (ICMP)' },
              slug: 'part-6-cli-tools/ping',
            },
            {
              label: 'Project: traceroute',
              translations: { 'pt-BR': 'Projeto: traceroute' },
              slug: 'part-6-cli-tools/traceroute',
            },
            {
              label: 'Project: eBPF Observability',
              translations: { 'pt-BR': 'Projeto: Observabilidade eBPF' },
              slug: 'part-6-cli-tools/ebpf-observability',
            },
            {
              label: 'Capstone: netdiag',
              translations: { 'pt-BR': 'Capstone: netdiag' },
              slug: 'part-6-cli-tools/netdiag',
            },
            {
              label: 'Part 6 in Review',
              translations: { 'pt-BR': 'Parte 6 em Revisão' },
              slug: 'part-6-cli-tools/synthesis',
            },
          ],
        },
        {
          label: 'Capstone',
          translations: { 'pt-BR': 'Capstone' },
          items: [
            {
              label: 'Secure Guestbook, End to End',
              translations: { 'pt-BR': 'Guestbook Seguro, End to End' },
              slug: 'capstone/guestbook',
              badge: { text: 'Capstone', variant: 'tip' },
            },
            {
              label: 'Running It for Real: HTTP + SQLite',
              translations: { 'pt-BR': 'A Correr a Sério: HTTP + SQLite' },
              slug: 'capstone/running-app',
              badge: { text: 'Live app', variant: 'tip' },
            },
            {
              label: 'The Three Languages, Compared',
              translations: { 'pt-BR': 'As Três Linguagens, Comparadas' },
              slug: 'capstone/three-languages',
              badge: { text: 'Synthesis', variant: 'note' },
            },
          ],
        },
      ],
      components: {
        // Custom footer with the required authorship block.
        Footer: './src/components/AcademyFooter.astro',
      },
    }),
  ],
});
