'use client';

import { motion } from 'framer-motion';
import { Copy, Check } from 'lucide-react';
import { useState } from 'react';

export default function Installation() {
  const [copied, setCopied] = useState(false);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section className="py-32 px-6 bg-white">
      <div className="max-w-4xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
        >
          <h2 className="text-4xl font-medium text-gray-900 mb-8 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Installation
          </h2>

          <div className="space-y-8">
            <div>
              <h3 className="text-xl font-medium text-gray-900 mb-4">Prerequisites</h3>
              <ul className="space-y-2 text-gray-600">
                <li className="flex items-start gap-2">
                  <span className="w-1.5 h-1.5 rounded-full bg-gray-900 mt-2 shrink-0" />
                  <span>Rust 1.70+ — <a href="https://rustup.rs/" className="text-gray-900 hover:underline">Install Rust</a></span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="w-1.5 h-1.5 rounded-full bg-gray-900 mt-2 shrink-0" />
                  <span>Git — for repository analysis</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="w-1.5 h-1.5 rounded-full bg-gray-900 mt-2 shrink-0" />
                  <span>Optional: Ollama for local AI, or API keys for Groq/OpenAI</span>
                </li>
              </ul>
            </div>

            <div>
              <h3 className="text-xl font-medium text-gray-900 mb-4">Install from Source</h3>
              <div className="relative">
                <pre className="bg-gray-900 text-gray-100 p-6 overflow-x-auto text-sm font-mono">
{`# Clone the repository
git clone https://github.com/yourusername/astra.git
cd astra

# Build release binary
cargo build --release

# Add to PATH (optional)
cp target/release/astra /usr/local/bin/`}
                </pre>
                <button
                  onClick={() => copyToClipboard('git clone https://github.com/yourusername/astra.git\ncd astra\ncargo build --release\ncp target/release/astra /usr/local/bin/')}
                  className="absolute top-4 right-4 p-2 bg-gray-800 hover:bg-gray-700 transition-colors"
                >
                  {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                </button>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
