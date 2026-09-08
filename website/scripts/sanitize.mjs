import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const prohibited = [
  /\/Users\/[A-Za-z0-9._-]+\//,
  /BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY/,
  /(?:api[_-]?key|client[_-]?secret|password)\s*[:=]\s*["'][^"']+/i,
  /\.prometheus\/(?:events\.jsonl|knowledge\/private)/,
];

function walk(dir) {
  for (const entry of fs.readdirSync(dir, {withFileTypes: true})) {
    const item = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(item);
    else if (/\.(?:md|mdx|js|jsx|ts|tsx|json|ya?ml|css)$/.test(entry.name)) {
      const text = fs.readFileSync(item, 'utf8');
      for (const rule of prohibited) if (rule.test(text)) throw new Error(`prohibited public content in ${item}: ${rule}`);
    }
  }
}

for (const dir of ['docs', 'src', 'static']) if (fs.existsSync(path.join(root, dir))) walk(path.join(root, dir));
console.log('public-content sanitization passed');
