'use client';

import { Github, Twitter, Linkedin } from 'lucide-react';

const Footer = () => {
  return (
    <footer className="py-20 px-6 bg-[#faf9f6] border-t border-gray-200">
      <div className="max-w-6xl mx-auto">
        <div className="grid md:grid-cols-4 gap-12 mb-16">
          <div className="col-span-2">
            <div className="flex items-center mb-6">
              <div className="w-30 h-30 flex items-center justify-center shrink-0">
                <img src="/astra-logo-2.png" alt="Astra Logo" className="w-full h-full object-contain" />
              </div>
              {/* <span className="text-xl font-bold text-gray-900 tracking-tight" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
                Astra
              </span> */}
            </div>
            <p className="text-gray-600 max-w-sm mb-8 leading-relaxed">
              AI-powered CLI that understands your entire codebase. Cross-language migrations, time travel debugging, and semantic refactoring.
            </p>
            <div className="flex gap-4">
              <a href="#" className="w-10 h-10 border border-gray-300 flex items-center justify-center text-gray-600 hover:border-gray-900 hover:text-gray-900 transition-colors">
                <Github className="w-4 h-4" />
              </a>
              <a href="#" className="w-10 h-10 border border-gray-300 flex items-center justify-center text-gray-600 hover:border-gray-900 hover:text-gray-900 transition-colors">
                <Twitter className="w-4 h-4" />
              </a>
              <a href="#" className="w-10 h-10 border border-gray-300 flex items-center justify-center text-gray-600 hover:border-gray-900 hover:text-gray-900 transition-colors">
                <Linkedin className="w-4 h-4" />
              </a>
            </div>
          </div>
          
          <div>
            <h4 className="text-xs font-bold text-gray-500 uppercase tracking-widest mb-6">Product</h4>
            <ul className="space-y-3">
              <li>
                <a href="/#features" className="text-sm text-gray-700 hover:text-gray-900 transition-colors relative group inline-block">
                  Features
                  <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
                </a>
              </li>
              <li>
                <a href="/integrations" className="text-sm text-gray-700 hover:text-gray-900 transition-colors relative group inline-block">
                  Integrations
                  <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
                </a>
              </li>
              <li>
                <a href="/pricing" className="text-sm text-gray-700 hover:text-gray-900 transition-colors relative group inline-block">
                  Pricing
                  <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
                </a>
              </li>
              <li>
                <a href="/docs" className="text-sm text-gray-700 hover:text-gray-900 transition-colors relative group inline-block">
                  Documentation
                  <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
                </a>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="text-xs font-bold text-gray-500 uppercase tracking-widest mb-6">Company</h4>
            <ul className="space-y-3">
              <li>
                <a href="/about" className="text-sm text-gray-700 hover:text-gray-900 transition-colors relative group inline-block">
                  About
                  <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
                </a>
              </li>
              <li>
                <a href="/blog" className="text-sm text-gray-700 hover:text-gray-900 transition-colors relative group inline-block">
                  Blog
                  <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
                </a>
              </li>
              <li>
                <a href="/careers" className="text-sm text-gray-700 hover:text-gray-900 transition-colors relative group inline-block">
                  Careers
                  <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
                </a>
              </li>
              <li>
                <a href="/contact" className="text-sm text-gray-700 hover:text-gray-900 transition-colors relative group inline-block">
                  Contact
                  <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
                </a>
              </li>
            </ul>
          </div>
        </div>
        
        <div className="pt-8 border-t border-gray-200 flex flex-col md:flex-row items-center justify-between gap-4">
          <div className="text-xs text-gray-600">
            © 2026 Astra. All rights reserved.
          </div>
          <div className="flex gap-6 text-xs">
            <a href="/privacy" className="text-gray-600 hover:text-gray-900 transition-colors relative group">
              Privacy Policy
              <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
            </a>
            <a href="/terms" className="text-gray-600 hover:text-gray-900 transition-colors relative group">
              Terms of Service
              <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
