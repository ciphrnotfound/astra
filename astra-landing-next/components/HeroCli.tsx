"use client";

import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface Message {
  type: 'user' | 'astra' | 'system';
  content: string;
}

const CONVERSATION_SEQUENCES: Message[][] = [
  [
    { type: 'user', content: 'astra "why did we switch to GraphQL?"' },
    { type: 'system', content: 'Analyzing git history...' },
    { type: 'astra', content: 'Found in commit a3f2b1c by @sarah (Dec 2023)\n\n"Switch from REST to GraphQL for better data fetching"\n\n• Reduced over-fetching by 60%\n• Frontend teams wanted flexible queries\n• Mobile needed nested data in single request' },
  ],
  [
    { type: 'user', content: 'astra "who owns the auth module?"' },
    { type: 'system', content: 'Scanning ownership...' },
    { type: 'astra', content: '@alex (67% of commits)\n@jordan (23% of commits)\n\nLast change: 3 weeks ago\nFiles: src/auth/*.ts (12 files)' },
  ],
  [
    { type: 'user', content: 'astra :bisect "login timeout"' },
    { type: 'system', content: 'Running bisect...' },
    { type: 'astra', content: 'Bug in commit 7d4e9a2 by @morgan (5 days ago)\n\n"Increase API timeout to 30s"\n\n• Old: 10s timeout\n• New: 30s timeout\n• Issue: Login waits full 30s on failure\n\nFix: Separate timeout for auth endpoints' },
  ],
];

export default function HeroCli() {
  const [sequenceIndex, setSequenceIndex] = useState(0);
  const [messages, setMessages] = useState<Message[]>([]);
  const [messageIndex, setMessageIndex] = useState(0);
  const [isTyping, setIsTyping] = useState(false);

  useEffect(() => {
    const currentSequence = CONVERSATION_SEQUENCES[sequenceIndex];
    
    if (messageIndex >= currentSequence.length) {
      const timeout = setTimeout(() => {
        setMessages([]);
        setMessageIndex(0);
        setSequenceIndex((prev) => (prev + 1) % CONVERSATION_SEQUENCES.length);
      }, 4000);
      return () => clearTimeout(timeout);
    }

    const currentMessage = currentSequence[messageIndex];
    const delay = messageIndex === 0 ? 500 : currentMessage.type === 'system' ? 600 : 1000;

    const timeout = setTimeout(() => {
      if (currentMessage.type === 'astra') {
        setIsTyping(true);
        setTimeout(() => {
          setMessages((prev) => [...prev, currentMessage]);
          setIsTyping(false);
          setMessageIndex((prev) => prev + 1);
        }, 500);
      } else {
        setMessages((prev) => [...prev, currentMessage]);
        setMessageIndex((prev) => prev + 1);
      }
    }, delay);

    return () => clearTimeout(timeout);
  }, [sequenceIndex, messageIndex]);

  return (
    <div className="w-full">
      <div className="rounded-lg border border-gray-200 overflow-hidden bg-white shadow-sm">
        {/* Header with subtle gradient */}
        <div className="flex items-center justify-between px-4 py-2.5 border-b border-gray-100 bg-gradient-to-r from-white via-blue-50/30 to-white">
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-blue-500/20" />
            <div className="w-2 h-2 rounded-full bg-indigo-500/20" />
            <div className="w-2 h-2 rounded-full bg-violet-500/20" />
          </div>
          <span className="text-[10px] text-gray-400 font-mono tracking-wide">ASTRA</span>
          <div className="w-12" />
        </div>

        {/* Content */}
        <div className="p-6 font-mono text-[13px] leading-relaxed min-h-[380px] max-h-[380px] overflow-y-auto">
          <AnimatePresence mode="sync">
            {messages.map((message, index) => (
              <motion.div
                key={`${sequenceIndex}-${index}`}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.25, ease: [0.23, 1, 0.32, 1] }}
                className="mb-6 last:mb-0"
              >
                {message.type === 'user' && (
                  <div className="flex gap-3">
                    <span className="text-blue-400 select-none shrink-0">$</span>
                    <span className="text-gray-900">{message.content}</span>
                  </div>
                )}

                {message.type === 'system' && (
                  <div className="flex items-center gap-2.5 py-1">
                    <svg className="w-3 h-3 text-indigo-400 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                    <span className="text-gray-400 text-xs">{message.content}</span>
                  </div>
                )}

                {message.type === 'astra' && (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ duration: 0.3 }}
                    className="pl-6 border-l border-indigo-200 bg-gradient-to-r from-indigo-50/30 to-transparent py-2 -ml-2 pr-2"
                  >
                    <div className="text-gray-700 whitespace-pre-wrap">
                      {message.content}
                    </div>
                  </motion.div>
                )}
              </motion.div>
            ))}

            {isTyping && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="flex items-center gap-2.5 py-1"
              >
                <svg className="w-3 h-3 text-indigo-500 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                <span className="text-gray-400 text-xs">Thinking</span>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Cursor */}
          {messages.length > 0 && !isTyping && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.2 }}
              className="flex items-center gap-3 mt-6"
            >
              <span className="text-blue-400">$</span>
              <motion.div
                className="w-1.5 h-3.5 bg-indigo-500"
                animate={{ opacity: [1, 0, 1] }}
                transition={{ duration: 1.2, repeat: Infinity, ease: "easeInOut" }}
              />
            </motion.div>
          )}
        </div>
      </div>
    </div>
  );
}
