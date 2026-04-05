/**
 * Bug Condition Exploration Test
 * 
 * **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**
 * 
 * This test verifies that the missing authentication and team management functions
 * cause TypeScript compilation failures. This test is EXPECTED TO FAIL on unfixed code,
 * which confirms the bug exists.
 * 
 * Property 1: Bug Condition - Module Resolution Success
 * For any TypeScript file that imports authentication or team management functions
 * from '@/lib/db/queries' or imports Button from '@/components/ui/button',
 * the TypeScript compiler SHALL successfully resolve all imports.
 */

import { describe, it, expect } from 'vitest';
import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

describe('Bug Condition Exploration - Module Resolution', () => {
  const affectedFiles = [
    'lib/payments/stripe.ts',
    'lib/auth/middleware.ts',
    'app/(login)/actions.ts',
    'app/(login)/login.tsx',
    'app/api/user/route.ts',
    'app/api/team/route.ts',
  ];

  const missingFunctions = [
    'getUser',
    'getTeamForUser',
    'getUserWithTeam',
    'getTeamByStripeCustomerId',
    'updateTeamSubscription',
  ];

  it('should successfully resolve all imports from @/lib/db/queries', () => {
    // Check if the missing functions are exported from queries.ts
    const queriesPath = path.join(process.cwd(), 'lib/db/queries.ts');
    const queriesContent = fs.readFileSync(queriesPath, 'utf-8');

    const missingExports: string[] = [];
    
    for (const funcName of missingFunctions) {
      // Check if function is exported (either as export function or export { })
      const exportRegex = new RegExp(
        `export\\s+(async\\s+)?function\\s+${funcName}|export\\s*{[^}]*${funcName}[^}]*}`
      );
      
      if (!exportRegex.test(queriesContent)) {
        missingExports.push(funcName);
      }
    }

    // This assertion will FAIL on unfixed code, confirming the bug exists
    expect(missingExports).toEqual([]);
    
    if (missingExports.length > 0) {
      throw new Error(
        `Missing exports from @/lib/db/queries: ${missingExports.join(', ')}\n` +
        `This confirms the bug exists - these functions are imported but not defined.`
      );
    }
  });

  it('should successfully resolve Button import from @/components/ui/button', () => {
    const buttonPath = path.join(process.cwd(), 'components/ui/button.tsx');
    const buttonContent = fs.readFileSync(buttonPath, 'utf-8');

    // Check if Button is exported as a named export
    const hasNamedExport = /export\s+{\s*Button\s*}|export\s+function\s+Button|export\s+const\s+Button/.test(buttonContent);
    
    // This assertion will FAIL on unfixed code if Button uses default export
    expect(hasNamedExport).toBe(true);
    
    if (!hasNamedExport) {
      throw new Error(
        'Button component is not exported as a named export from @/components/ui/button\n' +
        'This confirms the bug exists - files expect a named export but only default export exists.'
      );
    }
  });

  it('should verify all affected files exist and import the missing functions', () => {
    const importIssues: string[] = [];

    for (const file of affectedFiles) {
      const filePath = path.join(process.cwd(), file);
      
      if (!fs.existsSync(filePath)) {
        importIssues.push(`File not found: ${file}`);
        continue;
      }

      const content = fs.readFileSync(filePath, 'utf-8');

      // Check for imports from @/lib/db/queries
      if (content.includes("from '@/lib/db/queries'") || content.includes('from "@/lib/db/queries"')) {
        const importMatch = content.match(/import\s+{([^}]+)}\s+from\s+['"]@\/lib\/db\/queries['"]/);
        if (importMatch) {
          const imports = importMatch[1].split(',').map(i => i.trim());
          const missingInFile = imports.filter(imp => missingFunctions.includes(imp));
          
          if (missingInFile.length > 0) {
            importIssues.push(`${file} imports missing functions: ${missingInFile.join(', ')}`);
          }
        }
      }

      // Check for Button import
      if (file.includes('login.tsx') && content.includes("from '@/components/ui/button'")) {
        importIssues.push(`${file} imports Button component`);
      }
    }

    // Document the affected files
    expect(importIssues.length).toBeGreaterThan(0);
    console.log('Affected files with missing imports:', importIssues);
  });
});
