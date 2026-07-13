import { defineCollection } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';
import { z } from 'astro:schema';

// Extend Starlight's frontmatter with Academy-specific teaching metadata:
// difficulty and estimated time drive the difficulty indicator badges,
// and `languages` powers the language-comparison filters.
export const collections = {
  docs: defineCollection({
    loader: docsLoader(),
    schema: docsSchema({
      extend: z.object({
        difficulty: z
          .enum(['beginner', 'intermediate', 'advanced', 'expert'])
          .optional(),
        estimatedMinutes: z.number().int().positive().optional(),
        languages: z.array(z.enum(['python', 'go', 'rust'])).optional(),
      }),
    }),
  }),
};
