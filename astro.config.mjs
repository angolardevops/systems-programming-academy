// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// The Ultimate Systems Programming Academy
// Author: Walter Angolar — Technical Review: Claude Code
export default defineConfig({
  site: 'https://angolardevops.github.io',
  base: '/systems-programming-academy',
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
            { label: 'Welcome', translations: { 'pt-BR': 'Bem-vindo' }, slug: 'welcome' },
            {
              label: 'How to use this Academy',
              translations: { 'pt-BR': 'Como usar esta Academia' },
              slug: 'how-to-use',
            },
          ],
        },
        {
          label: 'Part 1 · Foundations',
          translations: { 'pt-BR': 'Parte 1 · Fundamentos' },
          items: [
            {
              label: 'Rust',
              badge: { text: '9 · complete', variant: 'success' },
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
              badge: { text: '6 · complete', variant: 'success' },
              collapsed: true,
              items: [
                {
                  label: 'Structs, Methods & Interfaces',
                  translations: { 'pt-BR': 'Structs, Métodos e Interfaces' },
                  slug: 'part-1-foundations/go/interfaces',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Error Handling',
                  translations: { 'pt-BR': 'Tratamento de Erros' },
                  slug: 'part-1-foundations/go/error-handling',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Slices, Maps & Strings',
                  translations: { 'pt-BR': 'Slices, Maps e Strings' },
                  slug: 'part-1-foundations/go/collections',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Testing & Documentation',
                  translations: { 'pt-BR': 'Testes e Documentação' },
                  slug: 'part-1-foundations/go/testing',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Generics',
                  translations: { 'pt-BR': 'Generics' },
                  slug: 'part-1-foundations/go/generics',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Goroutines & Channels',
                  translations: { 'pt-BR': 'Goroutines e Channels' },
                  slug: 'part-1-foundations/go/concurrency',
                  badge: { text: 'Complete', variant: 'success' },
                },
              ],
            },
            {
              label: 'Python',
              badge: { text: '5 · complete', variant: 'success' },
              collapsed: true,
              items: [
                {
                  label: 'The Data Model',
                  translations: { 'pt-BR': 'O Modelo de Dados' },
                  slug: 'part-1-foundations/python/data-model',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Type Hints',
                  translations: { 'pt-BR': 'Type Hints' },
                  slug: 'part-1-foundations/python/type-hints',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Protocols & Duck Typing',
                  translations: { 'pt-BR': 'Protocols e Duck Typing' },
                  slug: 'part-1-foundations/python/protocols',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Iterators & Generators',
                  translations: { 'pt-BR': 'Iteradores e Geradores' },
                  slug: 'part-1-foundations/python/iterators',
                  badge: { text: 'Complete', variant: 'success' },
                },
                {
                  label: 'Testing',
                  translations: { 'pt-BR': 'Testes' },
                  slug: 'part-1-foundations/python/testing',
                  badge: { text: 'Complete', variant: 'success' },
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
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Configuration & Secrets',
              translations: { 'pt-BR': 'Configuração e Segredos' },
              slug: 'part-2-engineering/configuration',
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Logging & Observability',
              translations: { 'pt-BR': 'Logging e Observabilidade' },
              slug: 'part-2-engineering/logging',
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Caching',
              translations: { 'pt-BR': 'Caching' },
              slug: 'part-2-engineering/caching',
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'The Testing Pyramid',
              translations: { 'pt-BR': 'A Pirâmide de Testes' },
              slug: 'part-2-engineering/testing-pyramid',
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Clean & Hexagonal Architecture',
              translations: { 'pt-BR': 'Arquitetura Clean e Hexagonal' },
              slug: 'part-2-engineering/hexagonal-architecture',
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'CI/CD',
              translations: { 'pt-BR': 'CI/CD' },
              slug: 'part-2-engineering/ci-cd',
              badge: { text: 'Complete', variant: 'success' },
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
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Project: Prometheus-Style Exporter',
              translations: { 'pt-BR': 'Projeto: Exporter Estilo Prometheus' },
              slug: 'part-3-devops/prometheus-exporter',
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Project: Config Generator',
              translations: { 'pt-BR': 'Projeto: Gerador de Configuração' },
              slug: 'part-3-devops/config-generator',
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Project: Health-Check Agent',
              translations: { 'pt-BR': 'Projeto: Agente de Health-Check' },
              slug: 'part-3-devops/health-check',
              badge: { text: 'Complete', variant: 'success' },
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
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Message Passing & Channels',
              translations: { 'pt-BR': 'Message Passing & Channels' },
              slug: 'part-4-concurrency/message-passing',
              badge: { text: 'Complete', variant: 'success' },
            },
            {
              label: 'Async & Event-Driven I/O',
              translations: { 'pt-BR': 'Async & I/O Orientado a Eventos' },
              slug: 'part-4-concurrency/async-io',
              badge: { text: 'Complete', variant: 'success' },
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
