'use client';

import { motion } from 'framer-motion';
import { ArrowRight } from 'lucide-react';

export default function CodeShowcase() {
  return (
    <section className="py-12 md:py-24 px-4 md:px-6 bg-[#f5f3ef]">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="text-center mb-8 md:mb-16"
        >
          <h2 className="text-2xl md:text-4xl font-semibold text-gray-900 mb-3 md:mb-4 px-4">
            Migration made simple
          </h2>
          <p className="text-base md:text-lg text-gray-600 max-w-2xl mx-auto px-4">
            Watch Astra transform your TypeScript code into idiomatic Rust, preserving logic and improving performance.
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="grid md:grid-cols-2 gap-4 md:gap-6"
        >
          {/* TypeScript Code */}
          <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
            <div className="px-3 md:px-4 py-2 md:py-3 bg-gray-50 border-b border-gray-200 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 md:w-3 md:h-3 rounded-full bg-gray-300" />
                <span className="text-[10px] md:text-xs font-medium text-gray-600">TypeScript</span>
              </div>
              <span className="text-[10px] md:text-xs text-gray-500">user.ts</span>
            </div>
            <div className="p-3 md:p-6 font-mono text-[10px] md:text-sm overflow-x-auto">
              <pre className="text-gray-800 leading-relaxed">
{`interface User {
  id: string;
  name: string;
  email: string;
}

async function getUser(
  id: string
): Promise<User> {
  const response = await fetch(
    \`/api/users/\${id}\`
  );
  return response.json();
}

export { getUser };`}
              </pre>
            </div>
          </div>

          {/* Rust Code */}
          <div className="bg-white rounded-lg border border-gray-200 overflow-hidden relative">
            <div className="absolute top-2 md:top-3 right-2 md:right-3 z-10">
              <div className="px-1.5 md:px-2 py-0.5 md:py-1 bg-emerald-100 border border-emerald-200 rounded text-[10px] md:text-xs font-medium text-emerald-700">
                Migrated
              </div>
            </div>
            <div className="px-3 md:px-4 py-2 md:py-3 bg-gray-50 border-b border-gray-200 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 md:w-3 md:h-3 rounded-full bg-gray-900" />
                <span className="text-[10px] md:text-xs font-medium text-gray-600">Rust</span>
              </div>
              <span className="text-[10px] md:text-xs text-gray-500">user.rs</span>
            </div>
            <div className="p-3 md:p-6 font-mono text-[10px] md:text-sm overflow-x-auto">
              <pre className="text-gray-800 leading-relaxed">
{`use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

pub async fn get_user(
    id: &str
) -> Result<User, Error> {
    let url = format!("/api/users/{}", id);
    let response = reqwest::get(&url).await?;
    response.json().await
}`}
              </pre>
            </div>
          </div>
        </motion.div>

        {/* Arrow indicator */}
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.3 }}
          className="flex justify-center mt-6 md:mt-8"
        >
          <div className="flex items-center gap-2 md:gap-3 px-3 md:px-4 py-1.5 md:py-2 bg-white border border-gray-200 rounded-full text-xs md:text-sm text-gray-600">
            <span>One command</span>
            <ArrowRight className="w-3 h-3 md:w-4 md:h-4" />
            <span className="font-mono text-[10px] md:text-xs">astra migrate</span>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
